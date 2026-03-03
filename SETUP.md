# Quick Setup Guide

## Step 1: Install Rust & Build Tools

### A. Install Visual Studio Build Tools (Required for Windows)

**Before installing Rust**, you need the C++ build tools:

1. Visit **https://visualstudio.microsoft.com/downloads/**
2. Scroll to "**Tools for Visual Studio**"
3. Download "**Build Tools for Visual Studio 2022**"
4. Run the installer
5. Select: ✅ **Desktop development with C++**
6. Click Install (~6-7 GB, takes 10-15 minutes)
7. **Restart your computer**

**Alternative**: If you don't want Visual Studio tools, see "Option 2: GNU Toolchain" below.

### B. Install Rust

1. Visit **https://rustup.rs/**
2. Download and run the installer for Windows
3. Follow the installation prompts (default options are fine)
4. **Restart your terminal** after installation completes

To verify installation, run:
```bash
cargo --version
```

### Option 2: GNU Toolchain (No Visual Studio needed)

If you prefer not to install Visual Studio Build Tools:

1. Install Rust first (from https://rustup.rs/)
2. Run the included `switch_to_gnu.bat` file
3. This switches to the GNU toolchain which doesn't need Visual Studio

**Note**: The GNU toolchain may have compatibility issues with some libraries. MSVC (Visual Studio) is recommended.

## Step 2: Get OpenAI API Key

1. Go to **https://platform.openai.com/api-keys**
2. Sign in or create an account
3. Click "Create new secret key"
4. Copy the key (you'll need it when running the app)

**Note**: You'll need to add credits to your OpenAI account. Processing receipts is very cheap (typically < $0.01 per receipt).

## Step 3: Build the Application

Open a terminal in this project directory and run:

```bash
cargo build --release
```

This will:
- Download all dependencies
- Compile the application
- Create an optimized executable

**First build takes 5-10 minutes** as it compiles all dependencies. Subsequent builds are much faster.

## Step 4: Run the Application

```bash
cargo run --release
```

Or run the executable directly:
```bash
.\target\release\receipt_extractor.exe
```

## Step 5: Use the Application

1. **Enter API Key**: Paste your OpenAI API key and click "Save"
2. **Select Files**: Click "📁 Select Documents" and choose your receipt files
3. **Configure & Extract**: Click "⚙ Configure Fields & Extract"
   - Review/modify the fields to extract
   - Click "Start Extraction"
4. **Wait**: The app will process all documents (progress bar shows status)
5. **Get Results**: Find `extracted_receipts.csv` in the project folder

## Troubleshooting

### "cargo: command not found"
- Rust is not installed or terminal wasn't restarted
- Install from https://rustup.rs/ and restart terminal

### "linker `link.exe` not found" or "MSVC not found"
**This is the most common error on Windows!**

You need Visual Studio Build Tools:
1. Download from: https://visualstudio.microsoft.com/downloads/
2. Install "Build Tools for Visual Studio 2022"
3. Select "Desktop development with C++"
4. Restart your computer
5. Try building again

**OR** use the GNU toolchain:
- Run `switch_to_gnu.bat` in the project folder
- Then try building again

### Build errors
- Make sure you have internet connection (needs to download dependencies)
- Try: `cargo clean` then `cargo build --release` again

### "OpenAI API error: 401"
- Invalid API key - check you copied it correctly
- API key might be expired - generate a new one

### "OpenAI API error: 429"
- Rate limit exceeded - wait a moment and try again
- Or you've run out of credits - add more at https://platform.openai.com/account/billing

### Processing is slow
- This is normal! Each document takes 2-10 seconds depending on size
- Images take longer than text/PDF files
- The progress bar shows current status

## File Support

- ✅ **PDF**: `.pdf` files
- ✅ **Images**: `.png`, `.jpg`, `.jpeg` files  
- ✅ **Text**: `.txt` files

## Tips

- **Batch processing**: Select all your receipts at once for efficiency
- **Custom fields**: Add any fields you need (e.g., "Category", "Notes", "Invoice Number")
- **Image quality**: Higher quality images = better extraction accuracy
- **File naming**: The CSV includes the original filename for each row

## Cost Estimate

Using GPT-4o-mini:
- Text/PDF receipt: ~$0.001 - $0.005 per document
- Image receipt: ~$0.005 - $0.01 per document

Processing 100 receipts typically costs less than $1.

