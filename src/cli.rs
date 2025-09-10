use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(name = "pdf-ocr-extractor")]
#[command(about = "High-performance PDF OCR text extraction tool")]
#[command(version)]
pub struct Args {
    /// Number of worker threads
    #[arg(short, long, default_value = "4")]
    pub threads: usize,
    
    /// SQLite database file
    #[arg(short, long, default_value = "pdf_extractions.db")]
    pub database: PathBuf,
    
    /// Input directory to search for PDFs
    #[arg(short, long, default_value = ".")]
    pub input_dir: PathBuf,
    
    /// Export results to text file after processing
    #[arg(long)]
    pub export_txt: Option<PathBuf>,
    
    /// Use full file hashing (slower but more accurate)
    #[arg(long)]
    pub full_hash: bool,
    
    /// Skip OCR and only do direct text extraction
    #[arg(long)]
    pub text_only: bool,
    
    /// Skip direct text extraction and only use OCR
    #[arg(long)]
    pub ocr_only: bool,
    
    /// Force re-processing of already processed files
    #[arg(long)]
    pub force: bool,
    
    /// Show verbose output
    #[arg(short, long)]
    pub verbose: bool,
}