# Cross-Platform Setup Guide

This application works on both **Windows** and **Linux**. The PDFium library is required for PDF processing.

## Library Files

The project includes PDFium libraries for both platforms in the `lib/` directory:

- **Windows**: `pdfium.dll` (also in root directory for convenience)
- **Linux**: `libpdfium.so`

## Running on Windows

### Option 1: Using the batch file
```cmd
run.bat
```

### Option 2: Direct execution
```cmd
cargo run --release
```

The Windows DLL is automatically found because:
1. `pdfium.dll` is in the project root
2. Windows searches the current directory for DLLs

## Running on Linux

### Option 1: Using the shell script (Recommended)
```bash
./run.sh
```

This script automatically sets the `LD_LIBRARY_PATH` to include the `lib/` directory.

### Option 2: Manual library path setup
```bash
export LD_LIBRARY_PATH=$PWD/lib:$LD_LIBRARY_PATH
cargo run --release
```

### Option 3: System-wide installation (Advanced)
```bash
sudo cp lib/libpdfium.so /usr/local/lib/
sudo ldconfig
```

## Building

The `build.rs` script automatically configures the library search path during compilation.

### Build for release:
```bash
cargo build --release
```

### Build for debug:
```bash
cargo build
```

## Troubleshooting

### Linux: "libpdfium.so: cannot open shared object file"

**Solution 1**: Use the `run.sh` script
```bash
./run.sh
```

**Solution 2**: Set library path before running
```bash
export LD_LIBRARY_PATH=$PWD/lib:$LD_LIBRARY_PATH
./target/release/receipt_extractor
```

**Solution 3**: Check if the library exists
```bash
ls -la lib/libpdfium.so
```

### Windows: "pdfium.dll not found"

**Solution 1**: Ensure `pdfium.dll` is in the project root
```cmd
dir pdfium.dll
```

**Solution 2**: Copy from lib directory
```cmd
copy lib\pdfium.dll .
```

## Library Versions

**IMPORTANT**: The PDFium library version must match the `pdfium-render` crate version!

- **pdfium-render crate**: v0.8.37 (requires Pdfium 7543)
- **Windows**: `pdfium-win-x64.tgz` from Chromium 7543
- **Linux**: `pdfium-linux-x64.tgz` from Chromium 7543

Download from: https://github.com/bblanchon/pdfium-binaries/releases/tag/chromium%2F7543

### Version Mismatch Errors

If you see errors like:
```
undefined symbol: FPDFFormObj_RemoveObject
```

This means your PDFium library is too old. Download the correct version (7543) for your platform.

## Development Notes

- The `build.rs` file handles compile-time library path configuration
- On Linux, the binary is built with rpath to find libraries in `../lib` relative to the executable
- On Windows, DLLs are searched in the current directory automatically
- Both platforms use the same `pdfium-render` Rust crate (version 0.8.37)

