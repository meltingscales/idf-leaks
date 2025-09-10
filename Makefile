.PHONY: help install run clean test

# Default target
help:
	@echo "PDF OCR Extractor - Available targets:"
	@echo ""
	@echo "  install      - Install dependencies (system + Python)"
	@echo "  run          - Extract text from all PDFs (4 threads)"
	@echo "  run-threads  - Extract with custom thread count"
	@echo "  run-fast     - Extract with maximum threads (CPU count)"
	@echo "  stats        - Show database statistics"
	@echo "  search       - Search extracted text"
	@echo "  export-txt   - Export database to text file"
	@echo "  clean        - Remove generated output files"
	@echo "  test         - Test OCR on a single PDF"
	@echo "  nix          - Enter Nix shell environment"
	@echo ""
	@echo "Quick start:"
	@echo "  make install && make run"
	@echo ""
	@echo "Database queries:"
	@echo "  make stats"
	@echo "  make search QUERY='your search term'"

# Install all dependencies
install:
	@echo "Installing dependencies..."
	./install_dependencies.sh

# Run the OCR extractor (default 4 threads)
run:
	@echo "Running PDF OCR extraction with 4 threads..."
	uv run pdf_ocr_extractor.py --threads 4

# Run with custom thread count
run-threads:
	@echo "Running PDF OCR extraction with custom thread count..."
	@read -p "Enter number of threads (default 4): " threads; \
	threads=$${threads:-4}; \
	echo "Using $$threads threads..."; \
	uv run pdf_ocr_extractor.py --threads $$threads

# Run with maximum threads (CPU count)
run-fast:
	@echo "Running PDF OCR extraction with maximum threads..."
	uv run pdf_ocr_extractor.py --threads $$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)

# Show database statistics
stats:
	@echo "Showing database statistics..."
	uv run query_database.py stats

# Search extracted text
search:
	@echo "Searching extracted text..."
	@if [ -z "$(QUERY)" ]; then \
		echo "Usage: make search QUERY='your search term'"; \
	else \
		uv run query_database.py search "$(QUERY)"; \
	fi

# Export database to text file
export-txt:
	@echo "Exporting database to text file..."
	uv run pdf_ocr_extractor.py --export-txt extracted_text_all_pdfs.txt --database pdf_extractions.db

# Clean up generated files
clean:
	@echo "Cleaning up generated files..."
	rm -f extracted_text_all_pdfs.txt
	rm -f test_output.txt
	rm -f pdf_extractions.db
	rm -rf __pycache__/
	rm -rf .pytest_cache/

# Test with a single PDF (if available)
test:
	@echo "Testing OCR on first available PDF..."
	@if [ -f "IDF Papers/*/document.pdf" ]; then \
		echo "Testing with first PDF found..."; \
		uv run -c "import sys; sys.path.append('.'); from pdf_ocr_extractor import extract_text_from_pdf; pdf_path = next(iter(__import__('glob').glob('IDF Papers/*/document.pdf')), None); print(f'Testing: {pdf_path}') if pdf_path else print('No PDFs found'); result = extract_text_from_pdf(pdf_path)[:500] + '...' if pdf_path else 'No test performed'; print(result)" > test_output.txt; \
		echo "Test output saved to test_output.txt"; \
	else \
		echo "No PDF files found for testing"; \
	fi

# Enter Nix shell (for NixOS users)
nix:
	@echo "Entering Nix shell environment..."
	nix-shell