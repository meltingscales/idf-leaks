use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod database;

#[derive(Parser, Debug)]
#[command(name = "pdf-query")]
#[command(about = "Query PDF extraction database")]
pub struct QueryArgs {
    /// SQLite database file
    #[arg(short, long, default_value = "pdf_extractions.db")]
    pub database: PathBuf,
    
    #[command(subcommand)]
    pub command: QueryCommand,
}

#[derive(Parser, Debug)]
pub enum QueryCommand {
    /// Show extraction statistics
    Stats,
    
    /// Search extracted text
    Search {
        /// Text to search for
        query: String,
        
        /// Maximum number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    
    /// List processed files
    List {
        /// Filter by extraction method
        #[arg(short, long)]
        method: Option<String>,
        
        /// Include failed extractions
        #[arg(long)]
        include_failed: bool,
    },
    
    /// Export to JSON
    Export {
        /// Output JSON file
        output: PathBuf,
        
        /// Include extracted text in export
        #[arg(long)]
        include_text: bool,
    },
}

pub async fn run_query(args: QueryArgs) -> Result<()> {
    let db = database::Database::new(&args.database).await?;
    
    match args.command {
        QueryCommand::Stats => {
            let stats = db.get_stats().await?;
            
            println!("PDF Extraction Database Statistics");
            println!("{}", "=".repeat(40));
            println!("Total files processed: {}", stats.total);
            println!("Successful extractions: {}", stats.successful);
            println!("Failed extractions: {}", stats.failed);
            if stats.total > 0 {
                println!("Success rate: {:.1}%", (stats.successful as f64 / stats.total as f64) * 100.0);
            }
            println!("Average processing time: {:.2}s", stats.avg_processing_time);
        }
        
        QueryCommand::Search { query, limit } => {
            let results = db.search_text(&query, limit).await?;
            
            println!("Search results for '{}' (showing {} of max {}):", query, results.len(), limit);
            println!("{}", "=".repeat(60));
            
            for result in results {
                println!("File: {}", result.file_path);
                println!("Method: {}", result.extraction_method);
                println!("Timestamp: {}", result.timestamp);
                println!("Preview: {}...", result.preview);
                println!("{}", "-".repeat(40));
            }
        }
        
        QueryCommand::List { method: _, include_failed: _ } => {
            // TODO: Implement list functionality
            println!("List functionality not yet implemented");
        }
        
        QueryCommand::Export { output: _, include_text: _ } => {
            // TODO: Implement export functionality
            println!("Export functionality not yet implemented");
        }
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let args = QueryArgs::parse();
    run_query(args).await
}