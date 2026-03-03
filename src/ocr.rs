use anyhow::{Result, Context};
use std::path::Path;
use tesseract::Tesseract;

use crate::debug_log::log as log_to_file;

/// Extract text from an image using Tesseract OCR
pub fn extract_text_from_image(image_path: &Path) -> Result<String> {
    let tesseract = Tesseract::new(None, Some("eng"))
        .context("Failed to initialize Tesseract")?;

    let text = tesseract
        .set_image(image_path.to_str().unwrap())
        .context("Failed to set image for OCR")?
        .get_text()
        .context("Failed to extract text from image")?;

    Ok(text)
}

/// Parse OCR text to find specific tax lines (GST, PST, etc.)
pub fn find_tax_lines(ocr_text: &str) -> Vec<(String, f64)> {
    let mut taxes = Vec::new();
    let mut found_labels = std::collections::HashSet::new();

    // Common tax labels to look for (in order of priority)
    let tax_patterns = [
        ("GST", vec!["GST", "G.S.T", "G S T", "G. S. T"]),
        ("PST", vec!["PST", "P.S.T", "P S T", "P. S. T"]),
        ("HST", vec!["HST", "H.S.T", "H S T", "H. S. T"]),
        ("QST", vec!["QST", "Q.S.T", "Q S T"]),
        ("TPS", vec!["TPS", "T.P.S"]),
        ("TVQ", vec!["TVQ", "T.V.Q"]),
        ("TVH", vec!["TVH", "T.V.H"]),
        ("VAT", vec!["VAT", "V.A.T"]),
    ];

    for line in ocr_text.lines() {
        let line_upper = line.to_uppercase();
        let line_clean = line_upper.replace(".", "").replace(" ", "");

        // Check if this line contains a tax label
        for (tax_name, patterns) in &tax_patterns {
            // Skip if we already found this tax type
            if found_labels.contains(*tax_name) {
                continue;
            }

            let mut found = false;
            for pattern in patterns {
                let pattern_clean = pattern.replace(".", "").replace(" ", "");
                if line_clean.contains(&pattern_clean) {
                    found = true;
                    break;
                }
            }

            if found {
                // Try to extract the amount from this line
                if let Some(amount) = extract_amount_from_line(line) {
                    taxes.push(((*tax_name).to_string(), amount));
                    found_labels.insert((*tax_name).to_string());
                    break; // Found a tax on this line, move to next line
                }
            }
        }
    }

    taxes
}

/// Extract a numeric amount from a line of text
fn extract_amount_from_line(line: &str) -> Option<f64> {
    // Remove common currency symbols
    let cleaned = line.replace("$", "").replace("€", "").replace("£", "");

    // Find all potential numbers (with decimals and commas)
    let mut numbers = Vec::new();
    for word in cleaned.split_whitespace() {
        // Remove commas and try to parse
        let num_str = word.replace(",", "");

        // Check if it looks like a number (contains digits and maybe a decimal point)
        if num_str.chars().any(|c| c.is_numeric()) {
            // Try to extract just the numeric part
            let numeric_part: String = num_str
                .chars()
                .filter(|c| c.is_numeric() || *c == '.')
                .collect();

            if let Ok(amount) = numeric_part.parse::<f64>() {
                // Sanity check: amounts are usually between 0.01 and 100000
                if amount > 0.0 && amount < 100000.0 {
                    numbers.push(amount);
                }
            }
        }
    }

    // Usually the amount is the last number on the line
    numbers.last().copied()
}

/// Find the subtotal (amount before tax) from OCR text
pub fn find_subtotal(ocr_text: &str) -> Option<f64> {
    log_to_file("🔍 OCR: Searching for subtotal (amount before tax)...");

    // IMPORTANT: Order matters! Check for "ITEM TOTAL" BEFORE "SUBTOTAL"
    // because "Sub Total" often appears AFTER taxes (final amount)
    // while "Item Total" appears BEFORE taxes
    let priority_patterns = [
        "ITEM TOTAL", "ITEMS TOTAL", "ITEM-TOTAL",
        "MERCHANDISE TOTAL", "MERCH TOTAL",
        "AMOUNT BEFORE TAX", "BEFORE TAX",
    ];

    let fallback_patterns = [
        "SUBTOTAL", "SUB TOTAL", "SUB-TOTAL",
    ];

    // First, try priority patterns (these are more likely to be before tax)
    for line in ocr_text.lines() {
        let line_upper = line.to_uppercase();

        for pattern in &priority_patterns {
            if line_upper.contains(pattern) {
                if let Some(amount) = extract_amount_from_line(line) {
                    log_to_file(&format!("✅ OCR: Found subtotal via '{}': {:.2} (line: {})", pattern, amount, line.trim()));
                    return Some(amount);
                }
            }
        }
    }

    // If no priority pattern found, try fallback patterns
    for line in ocr_text.lines() {
        let line_upper = line.to_uppercase();

        for pattern in &fallback_patterns {
            if line_upper.contains(pattern) {
                if let Some(amount) = extract_amount_from_line(line) {
                    log_to_file(&format!("⚠️ OCR: Found subtotal via fallback '{}': {:.2} (line: {})", pattern, amount, line.trim()));
                    return Some(amount);
                }
            }
        }
    }

    log_to_file("❌ OCR: No subtotal found");
    None
}

/// Find the total amount from OCR text
pub fn find_total(ocr_text: &str) -> Option<f64> {
    log_to_file("🔍 OCR: Searching for total (final amount after tax)...");

    // Priority patterns (most specific first)
    let priority_patterns = [
        "SUB TOTAL", "SUBTOTAL", "SUB-TOTAL",  // After taxes
        "GRAND TOTAL",
        "FINAL TOTAL",
        "AMOUNT DUE", "BALANCE DUE",
        "AMOUNT PAID",
    ];

    // Fallback pattern (very generic, use last)
    let fallback_pattern = "TOTAL";

    // Search from bottom to top (total is usually at the end)
    let lines: Vec<&str> = ocr_text.lines().collect();

    // First try priority patterns
    for line in lines.iter().rev() {
        let line_upper = line.to_uppercase();

        for pattern in &priority_patterns {
            if line_upper.contains(pattern) {
                if let Some(amount) = extract_amount_from_line(line) {
                    log_to_file(&format!("✅ OCR: Found total via '{}': {:.2} (line: {})", pattern, amount, line.trim()));
                    return Some(amount);
                }
            }
        }
    }

    // If no priority pattern found, try generic "TOTAL" but skip "ITEM TOTAL"
    for line in lines.iter().rev() {
        let line_upper = line.to_uppercase();

        if line_upper.contains(fallback_pattern) &&
           !line_upper.contains("ITEM") &&
           !line_upper.contains("ITEMS") {
            if let Some(amount) = extract_amount_from_line(line) {
                log_to_file(&format!("⚠️ OCR: Found total via fallback 'TOTAL': {:.2} (line: {})", amount, line.trim()));
                return Some(amount);
            }
        }
    }

    log_to_file("❌ OCR: No total found");
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_amount() {
        assert_eq!(extract_amount_from_line("GST $58.54"), Some(58.54));
        assert_eq!(extract_amount_from_line("P.S.T 81.96"), Some(81.96));
        assert_eq!(extract_amount_from_line("Total: 1,311.31"), Some(1311.31));
    }

    #[test]
    fn test_find_tax_lines() {
        let text = "Item Total 1,170.81\nG.S.T 58.54\nP.S.T 81.96\nSub Total 1,311.31";
        let taxes = find_tax_lines(text);
        assert_eq!(taxes.len(), 2);
        assert_eq!(taxes[0].1, 58.54);
        assert_eq!(taxes[1].1, 81.96);
    }
}

