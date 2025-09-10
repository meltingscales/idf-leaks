#!/bin/bash
# Install system dependencies and Python packages for PDF OCR extraction

echo "PDF OCR Extractor Setup Script"
echo "=============================="

# Check if we're in a Nix environment
if [ -n "$NIX_STORE" ] || command -v nix-shell > /dev/null; then
    echo "Detected Nix environment!"
    echo ""
    echo "For NixOS users:"
    echo "  1. Run: nix-shell"
    echo "  2. Install Python deps: uv pip install -r requirements.txt"
    echo "  3. Run extractor: python pdf_ocr_extractor.py"
    echo ""
    echo "Or run directly: nix-shell --run 'uv pip install -r requirements.txt && python pdf_ocr_extractor.py'"
    exit 0
fi

echo "Installing system dependencies..."

# Check package manager and install system deps
if command -v apt-get > /dev/null; then
    echo "Detected apt (Ubuntu/Debian)"
    sudo apt-get update
    sudo apt-get install -y tesseract-ocr poppler-utils
elif command -v dnf > /dev/null; then
    echo "Detected dnf (Fedora/RHEL)"
    sudo dnf install -y tesseract poppler-utils
elif command -v pacman > /dev/null; then
    echo "Detected pacman (Arch Linux)"
    sudo pacman -S tesseract poppler
elif command -v brew > /dev/null; then
    echo "Detected brew (macOS)"
    brew install tesseract poppler
else
    echo "Unknown package manager. Please install tesseract-ocr and poppler-utils manually."
    echo "Or use NixOS: nix-shell"
    exit 1
fi

echo "Installing Python dependencies..."

# Prefer uv if available, fallback to pip
if command -v uv > /dev/null; then
    echo "Using uv for Python package installation"
    uv pip install -r requirements.txt
else
    echo "Using pip for Python package installation"
    pip install -r requirements.txt
fi

echo ""
echo "Setup complete!"
echo "You can now run: python pdf_ocr_extractor.py"