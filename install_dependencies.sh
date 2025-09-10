#!/usr/bin/env bash
# Install system dependencies for PDF OCR extraction (Rust Edition)

echo "🦀 PDF OCR Extractor Setup Script (Rust Edition)"
echo "==============================================="

echo "Installing system dependencies..."

# Check package manager and install system deps
if command -v apt-get > /dev/null; then
    echo "Detected apt (Ubuntu/Debian)"
    sudo apt-get update
    sudo apt-get install -y tesseract-ocr poppler-utils build-essential
elif command -v dnf > /dev/null; then
    echo "Detected dnf (Fedora/RHEL)"
    sudo dnf install -y tesseract poppler-utils gcc
elif command -v pacman > /dev/null; then
    echo "Detected pacman (Arch Linux)"
    sudo pacman -S tesseract poppler base-devel
elif command -v brew > /dev/null; then
    echo "Detected brew (macOS)"
    brew install tesseract poppler
else
    echo "Unknown package manager. Please install tesseract-ocr and poppler-utils manually."
    exit 1
fi

# Check if Rust is installed
if ! command -v cargo > /dev/null; then
    echo ""
    echo "Rust not found. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    echo "Rust installed successfully!"
else
    echo "Rust already installed: $(rustc --version)"
fi

echo ""
echo "Setup complete!"
echo "Next steps:"
echo "  1. Build the project: make build"
echo "  2. Run extraction: make run"
echo "  3. Show help: make help"