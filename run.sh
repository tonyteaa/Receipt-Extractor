#!/bin/bash
# Run script for Linux - sets library path for PDFium

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Set library path to include the lib directory
export LD_LIBRARY_PATH="$SCRIPT_DIR/lib:$LD_LIBRARY_PATH"

# Run the application
if [ -f "$SCRIPT_DIR/target/release/receipt_extractor" ]; then
    echo "Running release build..."
    "$SCRIPT_DIR/target/release/receipt_extractor"
elif [ -f "$SCRIPT_DIR/target/debug/receipt_extractor" ]; then
    echo "Running debug build..."
    "$SCRIPT_DIR/target/debug/receipt_extractor"
else
    echo "No build found. Building release version..."
    cargo build --release
    "$SCRIPT_DIR/target/release/receipt_extractor"
fi

