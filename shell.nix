{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # System dependencies
    tesseract
    poppler_utils
    
    # Python and uv
    python3
    uv
    
    # Additional utilities
    file
  ];

  shellHook = ''
    echo "PDF OCR extraction environment loaded!"
    echo ""
    echo "Available tools:"
    echo "  - tesseract: $(tesseract --version | head -1)"
    echo "  - poppler: $(pdftoppm -h 2>&1 | head -1 || echo 'poppler installed')"
    echo "  - python: $(python --version)"
    echo "  - uv: $(uv --version)"
    echo ""
    echo "To install Python dependencies: uv pip install -r requirements.txt"
    echo "To run the extractor: python pdf_ocr_extractor.py"
  '';
}