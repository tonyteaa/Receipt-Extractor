use std::path::Path;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use base64::Engine;
use std::fs::OpenOptions;
use std::io::Write;
use pdfium_render::prelude::*;
use scraper::{Html, Selector};

use crate::ai_extractor::AIExtractor;
use crate::ocr;

fn log_to_file(msg: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("receipt_extractor_debug.log")
    {
        let _ = writeln!(file, "{}", msg);
    }
}

/// Represents a single item extracted from a receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedItem {
    pub fields: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedData {
    pub file_name: String,
    /// Summary fields for general extraction (Date, Vendor, Total, etc.)
    pub fields: HashMap<String, String>,
    /// Individual items for detailed extraction
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ExtractedItem>,
    /// Optional error message explaining why extraction failed or was incomplete
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extraction_error: Option<String>,
}

/// Options for configuring what to extract from documents
#[derive(Debug, Clone)]
pub struct ExtractionOptions {
    /// Enable general/summary extraction (one row per receipt)
    pub enable_general: bool,
    /// Enable detailed/item extraction (one row per item)
    pub enable_detailed: bool,
    /// Summary fields to extract (Date, Vendor, Total, etc.)
    pub summary_fields: Vec<String>,
    /// Item fields to extract (Item Name, Quantity, Unit Price, etc.)
    pub item_fields: Vec<String>,
    /// Custom extraction prompts for specific fields (field_name -> prompt)
    /// Used for AI-enhanced custom fields
    pub custom_prompts: HashMap<String, String>,
}

pub struct DocumentProcessor {
    results: Vec<ExtractedData>,
}

impl DocumentProcessor {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    /// Process a document with extraction options (supports dual extraction)
    pub async fn process_document_with_options(
        &mut self,
        file_path: &Path,
        options: &ExtractionOptions,
        ai: &mut AIExtractor,
    ) -> Result<()> {
        // Set custom prompts on the AI extractor for enhanced field extraction
        ai.set_custom_prompts(options.custom_prompts.clone());

        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Determine which fields to extract for summary
        // If detailed extraction is enabled, we also need context fields (Date, Vendor) for the items CSV
        let fields_to_extract = if options.enable_general && !options.summary_fields.is_empty() {
            options.summary_fields.clone()
        } else if options.enable_detailed {
            // Extract basic context fields for items CSV even if general extraction is disabled
            let context_fields: Vec<String> = options.summary_fields.iter()
                .filter(|f| {
                    let lower = f.to_lowercase();
                    lower.contains("date") || lower.contains("vendor") || lower.contains("store")
                })
                .cloned()
                .collect();
            if context_fields.is_empty() {
                // Default context fields if none found in summary_fields
                vec!["Date".to_string(), "Vendor/Store Name".to_string()]
            } else {
                context_fields
            }
        } else {
            Vec::new()
        };

        // Extract summary/context fields
        let mut extracted_fields = if !fields_to_extract.is_empty() {
            match extension.as_str() {
                "pdf" => self.process_pdf(file_path, &fields_to_extract, ai).await?,
                "png" | "jpg" | "jpeg" => self.process_image(file_path, &fields_to_extract, ai).await?,
                "txt" => self.process_text(file_path, &fields_to_extract, ai).await?,
                "html" | "htm" => self.process_html(file_path, &fields_to_extract, ai).await?,
                _ => {
                    eprintln!("Unsupported file type: {}", extension);
                    let mut fields_map = HashMap::new();
                    fields_map.insert("__extraction_error__".to_string(), format!("Unsupported file type: {}", extension));
                    fields_map
                }
            }
        } else {
            HashMap::new()
        };

        // Extract items if detailed extraction is enabled
        let items = if options.enable_detailed && !options.item_fields.is_empty() {
            match extension.as_str() {
                "pdf" => self.process_pdf_items(file_path, &options.item_fields, ai).await.unwrap_or_default(),
                "png" | "jpg" | "jpeg" => self.process_image_items(file_path, &options.item_fields, ai).await.unwrap_or_default(),
                "txt" => self.process_text_items(file_path, &options.item_fields, ai).await.unwrap_or_default(),
                "html" | "htm" => self.process_html_items(file_path, &options.item_fields, ai).await.unwrap_or_default(),
                _ => Vec::new(),
            }
        } else {
            Vec::new()
        };

        // Extract error reason if present (set by AI extractor when parsing fails)
        let extraction_error = extracted_fields.remove("__extraction_error__");

        self.results.push(ExtractedData {
            file_name,
            fields: extracted_fields,
            items,
            extraction_error,
        });

        Ok(())
    }

    async fn process_pdf(
        &self,
        file_path: &Path,
        fields: &[String],
        ai: &mut AIExtractor,
    ) -> Result<HashMap<String, String>> {
        // Try to extract text from PDF first
        let msg = format!("Extracting text from PDF: {:?}", file_path);
        log_to_file(&msg);

        let text = pdf_extract::extract_text(file_path)?;

        let msg = format!("Extracted {} characters from PDF", text.len());
        log_to_file(&msg);

        // If PDF has no text (image-based PDF), use vision API instead
        if text.trim().is_empty() {
            log_to_file("PDF contains no text - treating as image-based PDF, using vision API");
            return self.process_pdf_as_image(file_path, fields, ai).await;
        }

        let msg = format!("First 500 chars: {}", &text.chars().take(500).collect::<String>());
        log_to_file(&msg);

        // Use AI to extract structured data from text
        log_to_file("Sending to AI for extraction...");
        let result = ai.extract_from_text(&text, fields).await?;
        let msg = format!("AI extraction result: {:?}", result);
        log_to_file(&msg);
        Ok(result)
    }

    async fn process_pdf_as_image(
        &self,
        file_path: &Path,
        fields: &[String],
        ai: &mut AIExtractor,
    ) -> Result<HashMap<String, String>> {
        // Convert PDF pages to images and encode as base64
        log_to_file("Converting PDF to image for vision processing...");

        // Initialize Pdfium
        let pdfium = Pdfium::new(
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
                .or_else(|_| Pdfium::bind_to_system_library())?,
        );

        // Load the PDF
        let document = pdfium.load_pdf_from_file(file_path, None)?;
        let page_count = document.pages().len();
        log_to_file(&format!("PDF loaded, {} pages", page_count));

        // Render config - use smaller resolution for multi-page PDFs to reduce payload size
        let (target_width, max_height) = if page_count > 1 {
            // Smaller resolution for multi-page to avoid request size limits
            (1200, 1800)
        } else {
            // Higher resolution for single page
            (2000, 3000)
        };

        let render_config = PdfRenderConfig::new()
            .set_target_width(target_width)
            .set_maximum_height(max_height);

        // Process up to 4 pages (to avoid token limits)
        let max_pages = std::cmp::min(page_count, 4);
        let mut base64_images = Vec::new();

        for page_index in 0..max_pages {
            log_to_file(&format!("Rendering page {} of {}", page_index + 1, page_count));
            let page = document.pages().get(page_index)?;
            let bitmap = page.render_with_config(&render_config)?;
            let image = bitmap.as_image();

            // Convert image to JPEG bytes (smaller than PNG for multi-page)
            let mut jpeg_bytes = Vec::new();
            image.write_to(
                &mut std::io::Cursor::new(&mut jpeg_bytes),
                image::ImageFormat::Jpeg
            )?;

            // Encode as base64
            let base64_image = base64::engine::general_purpose::STANDARD.encode(&jpeg_bytes);
            base64_images.push(base64_image);
            log_to_file(&format!("Page {} encoded, size: {} KB", page_index + 1, jpeg_bytes.len() / 1024));
        }

        log_to_file(&format!("Sending {} page(s) to vision API...", base64_images.len()));
        let result = ai.extract_from_images(&base64_images, fields).await?;
        let msg = format!("Vision API extraction result: {:?}", result);
        log_to_file(&msg);
        Ok(result)
    }

    async fn process_image(
        &self,
        file_path: &Path,
        fields: &[String],
        ai: &mut AIExtractor,
    ) -> Result<HashMap<String, String>> {
        log_to_file(&format!("Processing image: {:?}", file_path));

        // Step 1: Run OCR to extract text from the image
        log_to_file("Running OCR on image...");
        let ocr_result = ocr::extract_text_from_image(file_path);

        let mut result = HashMap::new();

        match ocr_result {
            Ok(ocr_text) => {
                log_to_file(&format!("OCR extracted {} characters", ocr_text.len()));
                log_to_file("========== OCR TEXT ==========");
                log_to_file(&ocr_text);
                log_to_file("==============================");

                // Step 2: Parse OCR text to find tax lines
                let tax_lines = ocr::find_tax_lines(&ocr_text);
                if !tax_lines.is_empty() {
                    log_to_file(&format!("Found {} tax line(s) in OCR:", tax_lines.len()));
                    let mut total_tax = 0.0;
                    for (label, amount) in &tax_lines {
                        log_to_file(&format!("  - {}: {:.2}", label, amount));
                        total_tax += amount;
                    }
                    log_to_file(&format!("  TOTAL TAX: {:.2}", total_tax));

                    // Pre-fill the tax amount from OCR
                    result.insert("Tax Amount".to_string(), format!("{:.2}", total_tax));
                }

                // Step 3: Find subtotal and total from OCR
                if let Some(subtotal) = ocr::find_subtotal(&ocr_text) {
                    log_to_file(&format!("OCR found subtotal: {:.2}", subtotal));
                    result.insert("amount befor taxe".to_string(), format!("{:.2}", subtotal));
                }

                if let Some(total) = ocr::find_total(&ocr_text) {
                    log_to_file(&format!("OCR found total: {:.2}", total));
                    result.insert("Total Amount".to_string(), format!("{:.2}", total));
                }
            }
            Err(e) => {
                log_to_file(&format!("OCR failed: {} - falling back to vision-only", e));
            }
        }

        // Step 4: Use AI vision to extract remaining fields
        log_to_file("Sending image to AI vision API...");
        let image_data = std::fs::read(file_path)?;
        let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_data);

        let ai_result = ai.extract_from_image(&base64_image, fields).await?;

        // Step 5: Merge results - OCR values take precedence for tax/amounts
        for (key, value) in ai_result {
            // Only use AI value if OCR didn't find it
            if !result.contains_key(&key) || result.get(&key).map(|v| v.is_empty()).unwrap_or(true) {
                result.insert(key, value);
            } else {
                log_to_file(&format!("Keeping OCR value for '{}': {} (AI suggested: {})",
                    key, result.get(&key).unwrap(), value));
            }
        }

        Ok(result)
    }

    async fn process_text(
        &self,
        file_path: &Path,
        fields: &[String],
        ai: &AIExtractor,
    ) -> Result<HashMap<String, String>> {
        // Read text file
        let text = std::fs::read_to_string(file_path)?;

        // Use AI to extract structured data
        ai.extract_from_text(&text, fields).await
    }

    async fn process_html(
        &self,
        file_path: &Path,
        fields: &[String],
        ai: &AIExtractor,
    ) -> Result<HashMap<String, String>> {
        log_to_file(&format!("Processing HTML file: {:?}", file_path));

        // Read HTML file
        let html_content = std::fs::read_to_string(file_path)?;

        // Parse HTML and extract text
        let document = Html::parse_document(&html_content);

        let mut text_parts = Vec::new();

        // Extract text from body, or entire document if no body
        let body_selector = Selector::parse("body").unwrap();
        if let Some(body) = document.select(&body_selector).next() {
            // Get text from body, excluding scripts and styles
            for node in body.descendants() {
                if let Some(text) = node.value().as_text() {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        text_parts.push(trimmed.to_string());
                    }
                }
            }
        } else {
            // No body tag, extract all text
            for node in document.root_element().descendants() {
                if let Some(text) = node.value().as_text() {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        text_parts.push(trimmed.to_string());
                    }
                }
            }
        }

        let extracted_text = text_parts.join("\n");

        log_to_file(&format!("Extracted {} characters from HTML", extracted_text.len()));
        log_to_file(&format!("First 500 chars: {}", &extracted_text.chars().take(500).collect::<String>()));

        // Use AI to extract structured data from text
        ai.extract_from_text(&extracted_text, fields).await
    }

    // ========================================================================
    // ITEM EXTRACTION METHODS (for detailed extraction mode)
    // ========================================================================

    async fn process_pdf_items(
        &self,
        file_path: &Path,
        item_fields: &[String],
        ai: &mut AIExtractor,
    ) -> Result<Vec<ExtractedItem>> {
        log_to_file(&format!("Extracting items from PDF: {:?}", file_path));

        // Try to extract text from PDF
        let text_result = pdf_extract::extract_text(file_path);
        if let Ok(text) = text_result {
            if !text.trim().is_empty() {
                return ai.extract_items_from_text(&text, item_fields).await;
            }
        }

        // Fall back to image-based extraction
        let bindings = Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
            .or_else(|_| Pdfium::bind_to_system_library())?;
        let pdfium = Pdfium::new(bindings);

        let document = pdfium.load_pdf_from_file(file_path, None)?;
        let mut all_items = Vec::new();

        for page in document.pages().iter() {
            let render_config = PdfRenderConfig::new()
                .set_target_width(2000)
                .set_maximum_height(3000)
                .rotate_if_landscape(PdfPageRenderRotation::Degrees90, true);

            let image = page.render_with_config(&render_config)?
                .as_image()
                .into_rgb8();

            let mut png_bytes = Vec::new();
            image.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png)?;
            let base64_image = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

            let page_items = ai.extract_items_from_images(&[base64_image], item_fields).await?;
            all_items.extend(page_items);
        }

        Ok(all_items)
    }

    async fn process_image_items(
        &self,
        file_path: &Path,
        item_fields: &[String],
        ai: &mut AIExtractor,
    ) -> Result<Vec<ExtractedItem>> {
        log_to_file(&format!("Extracting items from image: {:?}", file_path));

        let image_bytes = std::fs::read(file_path)?;
        let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_bytes);

        ai.extract_items_from_images(&[base64_image], item_fields).await
    }

    async fn process_text_items(
        &self,
        file_path: &Path,
        item_fields: &[String],
        ai: &AIExtractor,
    ) -> Result<Vec<ExtractedItem>> {
        log_to_file(&format!("Extracting items from text file: {:?}", file_path));

        let text = std::fs::read_to_string(file_path)?;
        ai.extract_items_from_text(&text, item_fields).await
    }

    async fn process_html_items(
        &self,
        file_path: &Path,
        item_fields: &[String],
        ai: &AIExtractor,
    ) -> Result<Vec<ExtractedItem>> {
        log_to_file(&format!("Extracting items from HTML: {:?}", file_path));

        let html_content = std::fs::read_to_string(file_path)?;
        let document = Html::parse_document(&html_content);

        let mut text_parts = Vec::new();
        let body_selector = Selector::parse("body").unwrap();

        if let Some(body) = document.select(&body_selector).next() {
            for node in body.descendants() {
                if let Some(text) = node.value().as_text() {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        text_parts.push(trimmed.to_string());
                    }
                }
            }
        } else {
            for node in document.root_element().descendants() {
                if let Some(text) = node.value().as_text() {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        text_parts.push(trimmed.to_string());
                    }
                }
            }
        }

        let extracted_text = text_parts.join("\n");
        ai.extract_items_from_text(&extracted_text, item_fields).await
    }

    pub fn get_results(&self) -> &[ExtractedData] {
        &self.results
    }
}

