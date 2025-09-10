.PHONY: help build install run clean test stats search export-txt bench

# Default target
help:
	@echo "🦀 PDF OCR Extractor (Rust Edition) - Available targets:"
	@echo ""
	@echo "  build        - Build the Rust binaries"
	@echo "  install      - Install system dependencies"
	@echo "  run          - Extract text from all PDFs (4 threads)"
	@echo "  run-fast     - Extract with maximum threads (CPU count)"
	@echo "  stats        - Show database statistics"
	@echo "  search       - Search extracted text"
	@echo "  export-txt   - Export database to text file"
	@echo "  clean        - Remove generated files and build artifacts"
	@echo "  test         - Run tests"
	@echo "  bench        - Run benchmarks"
	@echo ""
	@echo "Quick start:"
	@echo "  make install && make build && make run"
	@echo ""
	@echo "For faster processing:"
	@echo "  make run-fast"

# Install system dependencies
install:
	@echo "Installing system dependencies..."
	./install_dependencies.sh

# Build Rust binaries
build:
	@echo "Building Rust binaries..."
	cargo build --release

# Run the OCR extractor (default 4 threads)
run: build
	@echo "Running PDF OCR extraction with 4 threads..."
	./target/release/pdf-ocr-extractor --threads 4

# Run with maximum threads
run-fast: build
	@echo "Running PDF OCR extraction with maximum threads..."
	./target/release/pdf-ocr-extractor --threads $$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)

# Show database statistics
stats: build
	@echo "Showing database statistics..."
	./target/release/pdf-query stats

# Search extracted text
search: build
	@echo "Searching extracted text..."
	@if [ -z "$(QUERY)" ]; then \
		echo "Usage: make search QUERY='your search term'"; \
	else \
		./target/release/pdf-query search "$(QUERY)"; \
	fi

# Export database to text file
export-txt: build
	@echo "Exporting database to text file..."
	./target/release/pdf-ocr-extractor --export-txt extracted_text_all_pdfs.txt

# Clean up generated files and build artifacts
clean:
	@echo "Cleaning up..."
	rm -f extracted_text_all_pdfs.txt
	rm -f pdf_extractions.db
	cargo clean

# Run tests
test:
	@echo "Running tests..."
	cargo test

# Run benchmarks
bench:
	@echo "Running benchmarks..."
	cargo bench

# Development build (faster compile)
dev:
	@echo "Building in development mode..."
	cargo build

# Run in development mode
dev-run: dev
	@echo "Running in development mode..."
	./target/debug/pdf-ocr-extractor --threads 2 --verbose

# Check code quality
check:
	@echo "Checking code..."
	cargo check
	cargo clippy -- -D warnings
	cargo fmt --check

# Format code
fmt:
	@echo "Formatting code..."
	cargo fmt

# Update dependencies
update:
	@echo "Updating dependencies..."
	cargo update