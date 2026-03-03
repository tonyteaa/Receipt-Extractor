use anyhow::Result;
use csv::Writer;
use rust_xlsxwriter::{Workbook, Format};
use std::path::PathBuf;

use crate::document_processor::ExtractedData;

/// Export format options
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub enum ExportFormat {
    #[default]
    Csv,
    Excel,
    Json,
}

impl ExportFormat {
    #[allow(dead_code)]  // May be used for file extension handling in the future
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Csv => "csv",
            ExportFormat::Excel => "xlsx",
            ExportFormat::Json => "json",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ExportFormat::Csv => "CSV",
            ExportFormat::Excel => "Excel",
            ExportFormat::Json => "JSON",
        }
    }
}

/// Result of dual export
pub struct DualExportResult {
    pub summary_path: Option<PathBuf>,
    pub items_path: Option<PathBuf>,
}

pub struct CSVExporter;

impl CSVExporter {
    /// Export summary CSV (one row per receipt)
    pub fn export_summary(results: &[ExtractedData], fields: &[String], output_path: PathBuf) -> Result<PathBuf> {
        let mut writer = Writer::from_path(&output_path)?;

        // Write header
        let mut header = vec!["File Name".to_string()];
        header.extend_from_slice(fields);
        writer.write_record(&header)?;

        // Write data rows
        for result in results {
            let mut row = vec![result.file_name.clone()];
            for field in fields {
                let value = result.fields.get(field).cloned().unwrap_or_default();
                row.push(value);
            }
            writer.write_record(&row)?;
        }

        writer.flush()?;
        Ok(output_path)
    }

    /// Export items CSV (one row per item, with receipt context)
    pub fn export_items(
        results: &[ExtractedData],
        item_fields: &[String],
        summary_fields: &[String],
        output_path: PathBuf,
    ) -> Result<PathBuf> {
        let mut writer = Writer::from_path(&output_path)?;

        // Build header: File Name + context fields (Date, Vendor) + item fields
        let mut header = vec!["File Name".to_string()];

        // Add receipt context fields (Date and Vendor if they exist in summary fields)
        let context_fields: Vec<String> = summary_fields.iter()
            .filter(|f| {
                let lower = f.to_lowercase();
                lower.contains("date") || lower.contains("vendor") || lower.contains("store")
            })
            .cloned()
            .collect();
        header.extend_from_slice(&context_fields);

        // Add item fields
        header.extend_from_slice(item_fields);
        writer.write_record(&header)?;

        // Write data rows - one row per item
        for result in results {
            // Get context values from summary fields
            let context_values: Vec<String> = context_fields.iter()
                .map(|f| result.fields.get(f).cloned().unwrap_or_default())
                .collect();

            // Write each item as a separate row
            for item in &result.items {
                let mut row = vec![result.file_name.clone()];

                // Add context values
                row.extend(context_values.clone());

                // Add item field values
                for field in item_fields {
                    let value = item.fields.get(field).cloned().unwrap_or_default();
                    row.push(value);
                }
                writer.write_record(&row)?;
            }
        }

        writer.flush()?;
        Ok(output_path)
    }

    /// Export dual CSV files based on extraction options
    pub fn export_dual(
        results: &[ExtractedData],
        summary_fields: &[String],
        item_fields: &[String],
        enable_general: bool,
        enable_detailed: bool,
        base_path: PathBuf,
    ) -> Result<DualExportResult> {
        let mut result = DualExportResult {
            summary_path: None,
            items_path: None,
        };

        // Get the base name and directory
        let raw_stem = base_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        // Strip any existing _summary / _items suffix to avoid duplication
        let base_stem = raw_stem
            .strip_suffix("_summary")
            .or_else(|| raw_stem.strip_suffix("_items"))
            .unwrap_or(raw_stem);
        let parent = base_path.parent().unwrap_or(std::path::Path::new("."));

        // Export summary CSV if general extraction is enabled
        if enable_general && !summary_fields.is_empty() {
            let summary_path = parent.join(format!("{}_summary.csv", base_stem));
            Self::export_summary(results, summary_fields, summary_path.clone())?;
            result.summary_path = Some(summary_path);
        }

        // Export items CSV if detailed extraction is enabled
        if enable_detailed && !item_fields.is_empty() {
            // Only export items for results that have items
            let results_with_items: Vec<&ExtractedData> = results.iter()
                .filter(|r| !r.items.is_empty())
                .collect();

            if !results_with_items.is_empty() {
                let items_path = parent.join(format!("{}_items.csv", base_stem));
                let items_refs: Vec<ExtractedData> = results_with_items.into_iter().cloned().collect();
                Self::export_items(&items_refs, item_fields, summary_fields, items_path.clone())?;
                result.items_path = Some(items_path);
            }
        }

        Ok(result)
    }

    /// Export with specified format
    pub fn export_with_format(
        results: &[ExtractedData],
        summary_fields: &[String],
        item_fields: &[String],
        enable_general: bool,
        enable_detailed: bool,
        base_path: PathBuf,
        format: ExportFormat,
    ) -> Result<DualExportResult> {
        match format {
            ExportFormat::Csv => Self::export_dual(results, summary_fields, item_fields, enable_general, enable_detailed, base_path),
            ExportFormat::Excel => Self::export_excel(results, summary_fields, item_fields, enable_general, enable_detailed, base_path),
            ExportFormat::Json => Self::export_json(results, summary_fields, item_fields, enable_general, enable_detailed, base_path),
        }
    }

    /// Export to Excel format
    fn export_excel(
        results: &[ExtractedData],
        summary_fields: &[String],
        item_fields: &[String],
        enable_general: bool,
        enable_detailed: bool,
        base_path: PathBuf,
    ) -> Result<DualExportResult> {
        let mut result = DualExportResult {
            summary_path: None,
            items_path: None,
        };

        let base_stem = base_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let parent = base_path.parent().unwrap_or(std::path::Path::new("."));

        // Create a single workbook with multiple sheets
        let excel_path = parent.join(format!("{}.xlsx", base_stem));
        let mut workbook = Workbook::new();

        // Header format
        let header_format = Format::new().set_bold();

        // Export summary sheet if general extraction is enabled
        if enable_general && !summary_fields.is_empty() {
            let sheet = workbook.add_worksheet().set_name("Summary")?;

            // Write header
            sheet.write_with_format(0, 0, "File Name", &header_format)?;
            for (col, field) in summary_fields.iter().enumerate() {
                sheet.write_with_format(0, (col + 1) as u16, field.as_str(), &header_format)?;
            }

            // Write data
            for (row, data) in results.iter().enumerate() {
                let row_num = (row + 1) as u32;
                sheet.write_string(row_num, 0, &data.file_name)?;
                for (col, field) in summary_fields.iter().enumerate() {
                    let value = data.fields.get(field).cloned().unwrap_or_default();
                    sheet.write_string(row_num, (col + 1) as u16, &value)?;
                }
            }

            result.summary_path = Some(excel_path.clone());
        }

        // Export items sheet if detailed extraction is enabled
        if enable_detailed && !item_fields.is_empty() {
            let results_with_items: Vec<&ExtractedData> = results.iter()
                .filter(|r| !r.items.is_empty())
                .collect();

            if !results_with_items.is_empty() {
                let sheet = workbook.add_worksheet().set_name("Items")?;

                // Build context fields
                let context_fields: Vec<String> = summary_fields.iter()
                    .filter(|f| {
                        let lower = f.to_lowercase();
                        lower.contains("date") || lower.contains("vendor") || lower.contains("store")
                    })
                    .cloned()
                    .collect();

                // Write header
                sheet.write_with_format(0, 0, "File Name", &header_format)?;
                let mut col = 1;
                for field in &context_fields {
                    sheet.write_with_format(0, col, field.as_str(), &header_format)?;
                    col += 1;
                }
                for field in item_fields {
                    sheet.write_with_format(0, col, field.as_str(), &header_format)?;
                    col += 1;
                }

                // Write data
                let mut row_num: u32 = 1;
                for data in &results_with_items {
                    let context_values: Vec<String> = context_fields.iter()
                        .map(|f| data.fields.get(f).cloned().unwrap_or_default())
                        .collect();

                    for item in &data.items {
                        sheet.write_string(row_num, 0, &data.file_name)?;
                        let mut col: u16 = 1;
                        for value in &context_values {
                            sheet.write_string(row_num, col, value)?;
                            col += 1;
                        }
                        for field in item_fields {
                            let value = item.fields.get(field).cloned().unwrap_or_default();
                            sheet.write_string(row_num, col, &value)?;
                            col += 1;
                        }
                        row_num += 1;
                    }
                }

                result.items_path = Some(excel_path.clone());
            }
        }

        workbook.save(&excel_path)?;
        Ok(result)
    }

    /// Export to JSON format
    fn export_json(
        results: &[ExtractedData],
        summary_fields: &[String],
        item_fields: &[String],
        enable_general: bool,
        enable_detailed: bool,
        base_path: PathBuf,
    ) -> Result<DualExportResult> {
        use std::collections::HashMap;

        let mut result = DualExportResult {
            summary_path: None,
            items_path: None,
        };

        let raw_stem = base_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        // Strip any existing _summary / _items suffix to avoid duplication
        let base_stem = raw_stem
            .strip_suffix("_summary")
            .or_else(|| raw_stem.strip_suffix("_items"))
            .unwrap_or(raw_stem);
        let parent = base_path.parent().unwrap_or(std::path::Path::new("."));

        // Export summary JSON if general extraction is enabled
        if enable_general && !summary_fields.is_empty() {
            let summary_path = parent.join(format!("{}_summary.json", base_stem));

            let json_data: Vec<HashMap<String, String>> = results.iter().map(|r| {
                let mut map = HashMap::new();
                map.insert("file_name".to_string(), r.file_name.clone());
                for field in summary_fields {
                    let value = r.fields.get(field).cloned().unwrap_or_default();
                    map.insert(field.clone(), value);
                }
                map
            }).collect();

            let json_str = serde_json::to_string_pretty(&json_data)?;
            std::fs::write(&summary_path, json_str)?;
            result.summary_path = Some(summary_path);
        }

        // Export items JSON if detailed extraction is enabled
        if enable_detailed && !item_fields.is_empty() {
            let results_with_items: Vec<&ExtractedData> = results.iter()
                .filter(|r| !r.items.is_empty())
                .collect();

            if !results_with_items.is_empty() {
                let items_path = parent.join(format!("{}_items.json", base_stem));

                // Build context fields
                let context_fields: Vec<String> = summary_fields.iter()
                    .filter(|f| {
                        let lower = f.to_lowercase();
                        lower.contains("date") || lower.contains("vendor") || lower.contains("store")
                    })
                    .cloned()
                    .collect();

                // Build items JSON data - using explicit loop to avoid borrow issues
                let mut json_data: Vec<HashMap<String, String>> = Vec::new();
                for r in &results_with_items {
                    for item in &r.items {
                        let mut map = HashMap::new();
                        map.insert("file_name".to_string(), r.file_name.clone());
                        for field in &context_fields {
                            let value = r.fields.get(field).cloned().unwrap_or_default();
                            map.insert(field.clone(), value);
                        }
                        for field in item_fields {
                            let value = item.fields.get(field).cloned().unwrap_or_default();
                            map.insert(field.clone(), value);
                        }
                        json_data.push(map);
                    }
                }

                let json_str = serde_json::to_string_pretty(&json_data)?;
                std::fs::write(&items_path, json_str)?;
                result.items_path = Some(items_path);
            }
        }

        Ok(result)
    }
}
