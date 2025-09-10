use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::sync::Mutex;
use log::{info, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub id: Option<i64>,
    pub file_path: String,
    pub file_hash: Option<String>,
    pub file_size: i64,
    pub extraction_method: String,
    pub extracted_text: String,
    pub page_count: i32,
    pub processing_time_seconds: f64,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub error_message: Option<String>,
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub async fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        
        // Enable WAL mode for better concurrency
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA cache_size=10000;
             PRAGMA temp_store=memory;"
        )?;
        
        Ok(Database {
            conn: Mutex::new(conn),
        })
    }
    
    pub async fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS pdf_extractions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT NOT NULL,
                file_hash TEXT,
                file_size INTEGER,
                extraction_method TEXT NOT NULL,
                extracted_text TEXT,
                page_count INTEGER,
                processing_time_seconds REAL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                success BOOLEAN NOT NULL,
                error_message TEXT,
                UNIQUE(file_path, file_hash)
            )
            "#,
            [],
        )?;
        
        // Create indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_file_path ON pdf_extractions(file_path)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_extraction_method ON pdf_extractions(extraction_method)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON pdf_extractions(timestamp)",
            [],
        )?;
        
        info!("Database schema initialized");
        Ok(())
    }
    
    pub async fn insert_result(&self, result: &ExtractionResult) -> Result<()> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            r#"
            INSERT OR REPLACE INTO pdf_extractions 
            (file_path, file_hash, file_size, extraction_method, extracted_text, 
             page_count, processing_time_seconds, success, error_message)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                result.file_path,
                result.file_hash,
                result.file_size,
                result.extraction_method,
                result.extracted_text,
                result.page_count,
                result.processing_time_seconds,
                result.success,
                result.error_message,
            ],
        )?;
        
        Ok(())
    }
    
    pub async fn batch_insert(&self, results: &[ExtractionResult]) -> Result<()> {
        let conn = self.conn.lock().await;
        let tx = conn.unchecked_transaction()?;
        
        {
            let mut stmt = tx.prepare(
                r#"
                INSERT OR REPLACE INTO pdf_extractions 
                (file_path, file_hash, file_size, extraction_method, extracted_text, 
                 page_count, processing_time_seconds, success, error_message)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
            )?;
            
            for result in results {
                stmt.execute(params![
                    result.file_path,
                    result.file_hash,
                    result.file_size,
                    result.extraction_method,
                    result.extracted_text,
                    result.page_count,
                    result.processing_time_seconds,
                    result.success,
                    result.error_message,
                ])?;
            }
        }
        
        tx.commit()?;
        Ok(())
    }
    
    pub async fn file_exists(&self, file_path: &str, file_hash: &str) -> Result<bool> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM pdf_extractions WHERE file_path = ?1 AND file_hash = ?2"
        )?;
        
        let count: i64 = stmt.query_row(params![file_path, file_hash], |row| {
            row.get(0)
        })?;
        
        Ok(count > 0)
    }
    
    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        let conn = self.conn.lock().await;
        
        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM pdf_extractions",
            [],
            |row| row.get(0)
        )?;
        
        let successful: i64 = conn.query_row(
            "SELECT COUNT(*) FROM pdf_extractions WHERE success = 1",
            [],
            |row| row.get(0)
        )?;
        
        let failed = total - successful;
        
        let avg_time: f64 = conn.query_row(
            "SELECT AVG(processing_time_seconds) FROM pdf_extractions WHERE success = 1",
            [],
            |row| row.get(0)
        ).unwrap_or(0.0);
        
        Ok(DatabaseStats {
            total,
            successful,
            failed,
            avg_processing_time: avg_time,
        })
    }
    
    pub async fn search_text(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare(
            r#"
            SELECT file_path, extraction_method, 
                   substr(extracted_text, 1, 200) as preview,
                   timestamp
            FROM pdf_extractions 
            WHERE extracted_text LIKE ?1 AND success = 1
            ORDER BY timestamp DESC
            LIMIT ?2
            "#,
        )?;
        
        let query_param = format!("%{}%", query);
        let results = stmt.query_map(params![query_param, limit], |row| {
            Ok(SearchResult {
                file_path: row.get(0)?,
                extraction_method: row.get(1)?,
                preview: row.get(2)?,
                timestamp: row.get(3)?,
            })
        })?;
        
        let mut search_results = Vec::new();
        for result in results {
            search_results.push(result?);
        }
        
        Ok(search_results)
    }
    
    pub async fn export_to_text(&self, output_path: &Path) -> Result<()> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare(
            r#"
            SELECT file_path, extraction_method, extracted_text, success, error_message, timestamp
            FROM pdf_extractions 
            ORDER BY file_path
            "#,
        )?;
        
        let results = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,  // file_path
                row.get::<_, String>(1)?,  // extraction_method
                row.get::<_, String>(2)?,  // extracted_text
                row.get::<_, bool>(3)?,    // success
                row.get::<_, Option<String>>(4)?, // error_message
                row.get::<_, String>(5)?,  // timestamp
            ))
        })?;
        
        let mut content = String::new();
        content.push_str("PDF Text Extraction Results (Rust Edition)\n");
        content.push_str(&"=".repeat(80));
        content.push_str("\n\n");
        
        for result in results {
            let (file_path, method, text, success, error, timestamp) = result?;
            
            content.push_str(&format!("FILE: {}\n", file_path));
            content.push_str(&format!("METHOD: {}\n", method));
            content.push_str(&format!("SUCCESS: {}\n", success));
            content.push_str(&format!("TIMESTAMP: {}\n", timestamp));
            content.push_str(&"=".repeat(80));
            content.push_str("\n");
            
            if success && !text.is_empty() {
                content.push_str(&text);
            } else if let Some(error_msg) = error {
                content.push_str(&format!("ERROR: {}\n", error_msg));
            }
            
            content.push_str("\n");
            content.push_str(&"=".repeat(80));
            content.push_str("\n\n");
        }
        
        tokio::fs::write(output_path, content).await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct DatabaseStats {
    pub total: i64,
    pub successful: i64,
    pub failed: i64,
    pub avg_processing_time: f64,
}

#[derive(Debug)]
pub struct SearchResult {
    pub file_path: String,
    pub extraction_method: String,
    pub preview: String,
    pub timestamp: String,
}