# PDF OCR Text Extractor

Extract text from PDF files using OCR (Optical Character Recognition). Supports both text-based and image-based PDFs.

## Quick Start

```bash
# Install dependencies and run
make install && make run
```

## Setup Options

### Option 1: Traditional Package Managers
```bash
./install_dependencies.sh
```

### Option 2: NixOS
```bash
nix-shell
uv pip install -r requirements.txt
```

## Usage

### Using Makefile (Recommended)
```bash
make help          # Show available commands
make install       # Install all dependencies  
make run           # Extract text from all PDFs (4 threads)
make run-fast      # Extract with maximum CPU threads
make run-threads   # Extract with custom thread count
make test          # Test with a single PDF
make clean         # Remove generated files
```

### Direct Execution
```bash
# Default (4 threads, SQLite database)
uv run pdf_ocr_extractor.py

# Custom thread count and database
uv run pdf_ocr_extractor.py --threads 8 --database my_results.db

# Export to text file after processing
uv run pdf_ocr_extractor.py --export-txt results.txt

# Show help
uv run pdf_ocr_extractor.py --help
```

### Database Queries
```bash
# Show statistics
uv run query_database.py stats

# Search extracted text
uv run query_database.py search "your search term"

# List all processed files
uv run query_database.py list

# Export to JSON
uv run query_database.py export results.json
```

## Dependencies

### System Requirements
- `tesseract-ocr` - OCR engine
- `poppler-utils` - PDF processing tools

### Python Packages
- PyPDF2 - PDF text extraction
- pytesseract - Tesseract OCR wrapper
- pdf2image - Convert PDF pages to images
- Pillow - Image processing

## Output

The script stores results in a SQLite database (`pdf_extractions.db`) with the following information:
- File path and metadata (size, hash)
- Extraction method used (direct text or OCR)
- Extracted text content
- Processing time and success status
- Timestamps and error messages

You can export results to a text file or query the database directly.

## How It Works

1. **Direct Text Extraction**: First attempts to extract text directly from PDFs using PyPDF2
2. **OCR Fallback**: If direct extraction yields minimal text, converts PDF pages to images and uses Tesseract OCR
3. **Batch Processing**: Processes all PDFs in directory tree and saves results to a single output file

## Supported Environments

- Ubuntu/Debian (apt)
- Fedora/RHEL (dnf) 
- Arch Linux (pacman)
- macOS (brew)
- NixOS (nix-shell)