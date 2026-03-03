#!/bin/bash
# Test script to verify PDFium library is accessible

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
export LD_LIBRARY_PATH="$SCRIPT_DIR/lib:$LD_LIBRARY_PATH"

echo "Testing PDFium library setup..."
echo "LD_LIBRARY_PATH: $LD_LIBRARY_PATH"
echo ""

# Check if library file exists
if [ -f "$SCRIPT_DIR/lib/libpdfium.so" ]; then
    echo "✓ libpdfium.so found in lib/ directory"
    ls -lh "$SCRIPT_DIR/lib/libpdfium.so"
else
    echo "✗ libpdfium.so NOT found in lib/ directory"
    exit 1
fi

echo ""
echo "Checking library dependencies..."
ldd "$SCRIPT_DIR/lib/libpdfium.so" | head -10

echo ""
echo "Testing if the application can find the library..."
echo "Running: ldd target/release/receipt_extractor | grep pdfium"
ldd "$SCRIPT_DIR/target/release/receipt_extractor" | grep pdfium

if [ $? -eq 0 ]; then
    echo ""
    echo "✓ SUCCESS: PDFium library is properly linked!"
else
    echo ""
    echo "✗ WARNING: PDFium library may not be found at runtime"
fi

