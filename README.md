# 🦀 PDF OCR Text Extractor (Rust Edition)

High-performance PDF OCR text extraction tool written in Rust. Provides true parallelism, memory safety, and blazing fast performance.

## Features

- **True Parallelism**: No GIL limitations - all CPU cores utilized efficiently
- **Memory Safe**: Rust's ownership system prevents memory leaks and data races
- **Fast**: Optimized for processing thousands of PDFs concurrently
- **Database Storage**: SQLite with WAL mode for concurrent operations
- **Smart Extraction**: Direct text extraction with OCR fallback
- **Progress Tracking**: Real-time progress bars and statistics
- **CLI Tools**: Main extractor + query tool for database analysis

## Quick Start

```bash
# Install dependencies and build
make install && make build && make run
```

## Installation

### System Dependencies
```bash
# Ubuntu/Debian
sudo apt-get install tesseract-ocr poppler-utils

# Or use our installer
./install_dependencies.sh
```

### Build from Source
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the project
cargo build --release
```

## Usage

### Main Extractor

```bash
# Default (4 threads)
./target/release/pdf-ocr-extractor

# Custom thread count
./target/release/pdf-ocr-extractor --threads 8

# Text extraction only (no OCR)
./target/release/pdf-ocr-extractor --text-only

# OCR only (skip direct text extraction) - best for image-based PDFs
./target/release/pdf-ocr-extractor --ocr-only

# GPU-accelerated OCR (fastest for image-based PDFs)
./target/release/pdf-ocr-extractor --ocr-only --use-gpu

# Custom input directory
./target/release/pdf-ocr-extractor --input-dir /path/to/pdfs

# Export to text file after processing
./target/release/pdf-ocr-extractor --export-txt results.txt

# Force reprocessing
./target/release/pdf-ocr-extractor --force

# Show help
./target/release/pdf-ocr-extractor --help
```

### Database Queries

```bash
# Show statistics
./target/release/pdf-query stats

# Search extracted text
./target/release/pdf-query search "keyword"

# List files
./target/release/pdf-query list

# Export to JSON
./target/release/pdf-query export results.json

# Open database in GUI browser (NixOS)
make db
```

### GUI Database Browser

When using NixOS with the provided `shell.nix`, you get access to **SQLite Browser** - a popular GUI tool for exploring SQLite databases:

```bash
# Quick access via Makefile
make db

# Or run directly  
sqlitebrowser pdf_extractions.db
```

The GUI browser allows you to:
- **Browse tables** and view extraction results
- **Run custom SQL queries** on the data
- **Export data** in various formats (CSV, JSON, etc.)
- **Visualize extraction statistics** and patterns
- **Edit data** if needed

### Using Makefile

```bash
make help          # Show available commands
make build         # Build binaries
make run           # Extract with 4 threads
make run-fast      # Extract with all CPU cores
make run-gpu-ocr   # GPU-accelerated OCR
make stats         # Show database statistics
make search QUERY="keyword"  # Search text
make db            # Open database in GUI browser
make clean         # Remove build artifacts
make test          # Run tests
make bench         # Run benchmarks
```

## Performance

The Rust version provides significant performance improvements:

- **No GIL**: True parallel processing on all CPU cores
- **Memory Efficiency**: Zero-cost abstractions and minimal allocations
- **Async I/O**: Non-blocking file operations and database writes
- **SIMD**: Optimized hashing and text processing
- **Batch Operations**: Efficient database transactions

Expected performance gains over Python:
- **2-5x faster** overall processing
- **5-10x faster** file hashing
- **Better memory usage** (lower peak RAM)
- **Linear scaling** with CPU cores

## Architecture

```
src/
├── main.rs          # Main entry point and orchestration
├── cli.rs           # Command line argument parsing
├── database.rs      # SQLite database operations
├── extractor.rs     # Main extraction logic and coordination
├── pdf.rs           # PDF text extraction
├── ocr.rs           # OCR processing with Tesseract
├── progress.rs      # Progress tracking and reporting
└── query.rs         # Database query tool
```

## Dependencies

- **PDF Processing**: `lopdf`, `pdf-extract`
- **OCR**: System `tesseract` + `poppler-utils`
- **Database**: `rusqlite` with bundled SQLite
- **Async Runtime**: `tokio` for I/O, `rayon` for CPU parallelism
- **CLI**: `clap` for argument parsing
- **Progress**: `indicatif` for progress bars

## Database Schema

```sql
CREATE TABLE pdf_extractions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path TEXT NOT NULL,
    file_hash TEXT,
    file_size INTEGER,
    extraction_method TEXT NOT NULL,  -- 'direct', 'ocr', 'error'
    extracted_text TEXT,
    page_count INTEGER,
    processing_time_seconds REAL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    UNIQUE(file_path, file_hash)
);
```

## Development

```bash
# Development build (faster compilation)
make dev

# Run in development mode
make dev-run

# Code quality checks
make check

# Format code
make fmt

# Update dependencies
make update
```

## Benchmarks

```bash
# Run performance benchmarks
make bench
```


## Why Rust?

- **No GIL**: Python's Global Interpreter Lock limits true parallelism
- **Memory Safety**: Prevents segfaults and memory leaks
- **Performance**: Systems language with zero-cost abstractions
- **Concurrency**: Fearless concurrency with compile-time guarantees
- **Ecosystem**: Excellent crates for PDF processing and databases
- **Future-Proof**: Growing adoption in systems and CLI tools