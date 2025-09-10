use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use log::{info, warn, error};

mod cli;
mod database;
mod extractor;
mod pdf;
mod ocr;
mod progress;

use cli::Args;
use database::Database;
use extractor::PdfExtractor;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    let args = Args::parse();
    
    info!("🦀 PDF OCR Extractor (Rust Edition) starting...");
    info!("Threads: {}", args.threads);
    info!("Database: {}", args.database.display());
    
    // Initialize database
    let mut db = Database::new(&args.database).await?;
    db.init_schema().await?;
    
    // Create extractor
    let extractor = PdfExtractor::new(args.clone());
    
    // Find PDF files
    let pdf_files = extractor.find_pdf_files(&args.input_dir).await?;
    
    if pdf_files.is_empty() {
        warn!("No PDF files found in {}", args.input_dir.display());
        return Ok(());
    }
    
    info!("Found {} PDF files", pdf_files.len());
    
    // Process files
    let results = extractor.process_files(pdf_files, &db).await?;
    
    // Print summary
    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.len() - successful;
    
    println!("\n🎉 Extraction complete!");
    println!("Processed: {}/{} PDFs", results.len(), results.len());
    println!("Successful: {}", successful);
    println!("Failed: {}", failed);
    println!("Database: {}", args.database.display());
    
    // Export if requested
    if let Some(export_path) = args.export_txt {
        db.export_to_text(&export_path).await?;
        println!("Results exported to: {}", export_path.display());
    }
    
    Ok(())
}