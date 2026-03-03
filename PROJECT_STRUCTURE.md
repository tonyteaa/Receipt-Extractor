# Project Structure

## Overview
This is a Rust desktop application using egui for the UI and OpenAI's API for AI-powered receipt data extraction.

## File Structure

```
receipt_extractor/
├── Cargo.toml                 # Rust project configuration & dependencies
├── .gitignore                 # Git ignore rules
├── README.md                  # Main documentation
├── SETUP.md                   # Quick setup guide
├── PROJECT_STRUCTURE.md       # This file
├── run.bat                    # Windows batch file to run the app
├── test_receipt.txt           # Sample receipt for testing
│
└── src/
    ├── main.rs                # Application entry point
    ├── app.rs                 # Main UI and application logic
    ├── document_processor.rs  # Document processing (PDF, images, text)
    ├── ai_extractor.rs        # OpenAI API integration
    └── csv_exporter.rs        # CSV file generation
```

## Module Descriptions

### `main.rs`
- Entry point of the application
- Initializes the egui window
- Sets up the native application options

### `app.rs` (ReceiptExtractorApp)
**Main UI and state management**

Key components:
- File selection dialog
- API key management
- Extraction fields configuration dialog
- Progress tracking with real-time updates
- Error handling and user feedback
- CSV export completion handling

State machine:
- `Idle`: Ready for user input
- `ConfiguringFields`: User is setting up extraction fields
- `Processing`: Documents are being processed
- `Completed`: Extraction finished successfully
- `Error`: An error occurred

### `document_processor.rs` (DocumentProcessor)
**Handles different document types**

Functions:
- `process_document()`: Main entry point for processing any document
- `process_pdf()`: Extracts text from PDF files using `pdf-extract`
- `process_image()`: Converts images to base64 for AI vision processing
- `process_text()`: Reads plain text files

Data structure:
- `ExtractedData`: Stores filename and extracted field values

### `ai_extractor.rs` (AIExtractor)
**OpenAI API integration**

Functions:
- `extract_from_text()`: Sends text to GPT-4o-mini for extraction
- `extract_from_image()`: Sends images to GPT-4o-mini Vision
- `create_extraction_prompt()`: Generates prompts for the AI
- `parse_response()`: Parses JSON responses from OpenAI

Features:
- Structured prompts for consistent extraction
- JSON parsing with error handling
- Support for both text and vision models

### `csv_exporter.rs` (CSVExporter)
**CSV file generation**

Functions:
- `export()`: Creates CSV file from extracted data

Output format:
- First column: Filename
- Subsequent columns: User-defined extraction fields
- One row per processed document

## Dependencies

### UI & System
- `eframe` / `egui`: Immediate mode GUI framework
- `rfd`: Native file dialogs
- `opener`: Open folders in file explorer

### Async & Networking
- `tokio`: Async runtime
- `reqwest`: HTTP client for API calls

### Data Processing
- `pdf-extract`: PDF text extraction
- `image`: Image processing
- `base64`: Base64 encoding for images
- `csv`: CSV file writing

### Serialization
- `serde` / `serde_json`: JSON serialization/deserialization

### Utilities
- `anyhow`: Error handling
- `walkdir`: Directory traversal (future use)

## Data Flow

1. **User Input** → Select files, configure fields
2. **Document Processing** → Extract text/images from files
3. **AI Processing** → Send to OpenAI API with extraction prompt
4. **Response Parsing** → Parse JSON response into structured data
5. **CSV Export** → Write all results to CSV file
6. **User Notification** → Show completion and open folder

## Threading Model

- **Main Thread**: UI rendering and user interaction (egui)
- **Worker Thread**: Document processing and API calls
- **Communication**: `std::sync::mpsc` channels for progress updates

Progress messages:
- `Progress(current, total)`: Update progress bar
- `Completed(path)`: Processing finished, CSV saved
- `Error(message)`: An error occurred

## Configuration

### Default Extraction Fields
1. Date
2. Vendor/Store Name
3. Total Amount
4. Tax Amount
5. Payment Method

Users can add/remove fields before extraction.

### AI Model Selection
- Text/PDF: `gpt-4o-mini` (fast, cost-effective)
- Images: `gpt-4o-mini` with vision (supports image input)

### Output
- Default filename: `extracted_receipts.csv`
- Location: Project root directory
- Format: UTF-8 CSV with headers

## Future Enhancements (Not Implemented)

Potential improvements:
- [ ] Batch folder processing
- [ ] Save/load field configurations
- [ ] Export to Excel/JSON formats
- [ ] Receipt preview before extraction
- [ ] Drag & drop file support
- [ ] Multi-language support
- [ ] Usage analytics dashboard

## Building & Running

**Development build:**
```bash
cargo build
cargo run
```

**Release build (optimized):**
```bash
cargo build --release
cargo run --release
```

**Executable location:**
```
target/release/receipt_extractor.exe
```

## Testing

Use the included `test_receipt.txt` to verify the application works:
1. Run the app
2. Enter your API key
3. Select `test_receipt.txt`
4. Configure fields and extract
5. Check `extracted_receipts.csv`

Expected output should include:
- Date: December 9, 2024
- Vendor: ACME GROCERY STORE
- Total: $65.28
- Tax: $5.11
- Payment Method: VISA

