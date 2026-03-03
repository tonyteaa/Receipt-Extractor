# Receipt Data Extractor

A free, open-source desktop application built with Rust and egui that extracts structured data from receipts and invoices using AI vision models. No license key required — just build and run.

## Features

- 📁 **Multi-file selection** — Select multiple documents at once (PDF, images, text files)
- 🤖 **Multiple AI providers** — Groq, ChatGPT (OpenAI), Claude (Anthropic), Google Gemini, and OpenRouter
- 🖥️ **Local LLM support** — Run fully offline using Ollama, LM Studio, or Jan (no API key needed)
- ⚡ **Fast extraction** — 2–5 seconds per receipt with cloud models
- 👁️ **Vision support** — Automatically handles image-based PDFs and scanned documents
- ⚙️ **Configurable fields** — Add any custom extraction field; it is included in the AI prompt automatically
- 📊 **Multiple export formats** — CSV, Excel (.xlsx), and JSON
- 💾 **Auto-save** — Results are saved after each successful extraction to prevent data loss
- 📂 **Quick-open buttons** — Open the output folder or the saved file directly from the app
- 🎯 **Accurate output** — Consistent date formatting (DD/MM/YYYY), amount detection, and line-item extraction

## Prerequisites

1. **Install Rust**: Visit [https://rustup.rs/](https://rustup.rs/) and follow the instructions
2. **An AI provider** — choose one:
   - **Cloud** (requires an API key): Groq, OpenAI, Anthropic, Google Gemini, or OpenRouter
   - **Local / free** (no API key): [Ollama](https://ollama.com) running on your machine

## Installation

```bash
git clone https://github.com/your-username/receipt-extractor.git
cd receipt-extractor
cargo build --release
```

## Usage

```bash
cargo run --release
```

Or run the compiled binary directly:

```bash
./target/release/receipts          # Linux / macOS
.\target\release\receipts.exe      # Windows
```

### Quick start

1. On first launch, select your **AI provider** and enter your API key (or configure a local LLM URL).
2. Click **📁 Select Documents** to choose receipt files (PDF, PNG, JPG, TXT).
3. Review or add extraction fields in the **Fields** panel.
4. Set an output path with the **Browse** button in the CSV Output section.
5. Click **▶ Extract Data** and wait for processing to finish.
6. Use **💾 Save Now** to export results, then:
   - **📂 Open Folder** — opens the folder containing the output file in your file manager
   - **📊 Open File** — opens the output file directly with your default program (e.g. LibreOffice Calc, Excel)

## AI Provider Setup

### Cloud providers
| Provider | Where to get an API key |
|---|---|
| **Groq** (recommended — fast & free tier) | https://console.groq.com/keys |
| **OpenAI (ChatGPT)** | https://platform.openai.com/api-keys |
| **Anthropic (Claude)** | https://console.anthropic.com/ |
| **Google Gemini** | https://aistudio.google.com/app/apikey |
| **OpenRouter** | https://openrouter.ai/keys |

### Local LLM (Ollama — free & private)
1. Install Ollama: `curl -fsSL https://ollama.com/install.sh | sh`
2. Pull a vision model: `ollama pull llava`
3. In the app, select **Local LLM** and enter:
   - **Server URL**: `http://localhost:11434` (default — no change needed)
   - **Model Name**: `llava`
4. Click **Test Connection** to verify.

> **Note:** You need a vision-capable model (e.g. `llava`, `moondream`) to process receipt images. Text-only models will not work for image receipts.

## Export Formats

| Format | Output files |
|---|---|
| **CSV** | `<name>_summary.csv` (one row per receipt) + `<name>_items.csv` (one row per line item) |
| **Excel** | Single `.xlsx` file with a Summary sheet and an Items sheet |
| **JSON** | `<name>_summary.json` + `<name>_items.json` |

The app automatically strips any `_summary` or `_items` suffix from the filename you enter, so you will never get double suffixes like `_summary_summary`.

## Default Extraction Fields

- **Date** — Purchase date (not the PDF creation date)
- **Vendor/Store Name** — Main store or marketplace
- **Seller** — Third-party seller if applicable
- **Total Amount** — Full transaction total
- **Tax Amount** — Tax charged
- **Payment Method** — Card, cash, etc.
- **Item** — Full product description
- **Item Summary** — Short item name (under 5 words)

You can add, remove, or rename any field before extraction.

## Supported File Types

- **PDF**: `.pdf`
- **Images**: `.png`, `.jpg`, `.jpeg`
- **Text**: `.txt`

## How It Works

1. **Document processing**
   - PDFs: text is extracted using the PDFium library; image-based PDFs are rendered and sent to a vision model
   - Images: sent directly to AI vision models for OCR and data extraction
   - Text files: read and processed directly

2. **AI extraction**
   - A structured prompt asks the model to return JSON with exactly the fields you configured
   - Supports both cloud APIs and locally hosted OpenAI-compatible servers (Ollama, LM Studio, Jan)

3. **Export**
   - Results are compiled into the selected format
   - Auto-save runs after every successful receipt so no data is lost if the app is closed early

## Privacy

- Your documents are sent to whichever AI provider you choose
- When using a **local LLM**, nothing leaves your machine
- API keys are stored locally in your OS config directory and never transmitted elsewhere
- Output files are saved locally only

## Troubleshooting

**"cargo: command not found"**
- Install Rust from https://rustup.rs/ and restart your terminal

**"AI API error" / rate limit**
- Wait a few seconds and try again, or switch to a different provider in Settings
- Process fewer files at once to stay within rate limits

**"Failed to parse AI response"**
- The model occasionally returns malformed JSON; the app logs the error and continues with the next file
- Try a more capable model if this happens frequently

**Local LLM not connecting**
- Confirm Ollama is running: open http://localhost:11434 in a browser — you should see "Ollama is running"
- Make sure you pulled a vision model: `ollama list`

## License

MIT License — free to use, modify, and distribute.

