{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # System dependencies for OCR
    tesseract
    poppler_utils
    
    # Rust toolchain
    cargo
    rustc
    rustfmt
    clippy
    
    # Build tools
    gcc
    pkg-config
    
    # SQLite GUI browser
    sqlitebrowser
    
    # Additional utilities
    file
  ];

  shellHook = ''
    echo "🦀 PDF OCR Extractor (Rust Edition) - Nix Environment"
    echo "=================================================="
    echo ""
    echo "Available tools:"
    echo "  - tesseract: $(tesseract --version | head -1)"
    echo "  - poppler: $(pdftoppm -h 2>&1 | head -1 || echo 'poppler installed')"
    echo "  - rust: $(rustc --version)"
    echo "  - cargo: $(cargo --version)"
    echo "  - sqlitebrowser: GUI database browser"
    echo ""
    echo "Quick start:"
    echo "  make build    # Build the Rust binaries"
    echo "  make run      # Extract text from PDFs"
    echo "  make db       # Open database in GUI browser"
    echo "  make help     # Show all commands"
  '';
}