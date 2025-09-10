use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;
use log::{info, warn, error};
use walkdir::WalkDir;

use crate::cli::Args;
use crate::database::{Database, ExtractionResult};
use crate::pdf::PdfProcessor;
use crate::ocr::OcrProcessor;
use crate::progress::ProgressTracker;

pub struct PdfExtractor {
    args: Args,
    pdf_processor: PdfProcessor,
    ocr_processor: OcrProcessor,
}

impl PdfExtractor {
    pub fn new(args: Args) -> Self {
        Self {
            pdf_processor: PdfProcessor::new(),
            ocr_processor: OcrProcessor::new(args.text_only, args.use_gpu),
            args,
        }
    }
    
    /// Find all PDF files in directory tree
    pub async fn find_pdf_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut pdf_files = Vec::new();
        
        for entry in WalkDir::new(dir) {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && 
               path.extension().and_then(|s| s.to_str()) == Some("pdf") {
                pdf_files.push(path.to_path_buf());
            }
        }
        
        pdf_files.sort();
        Ok(pdf_files)
    }
    
    /// Process multiple PDF files concurrently
    pub async fn process_files(
        &self, 
        pdf_files: Vec<PathBuf>, 
        db: &Database
    ) -> Result<Vec<ExtractionResult>> {
        // Check OCR availability
        if !self.ocr_processor.check_ocr_availability().await && !self.args.text_only {
            warn!("OCR tools not available - falling back to text-only mode");
        }
        
        let semaphore = Arc::new(Semaphore::new(self.args.threads));
        let progress = ProgressTracker::new(pdf_files.len());
        
        let tasks: Vec<_> = pdf_files
            .into_iter()
            .enumerate()
            .map(|(index, pdf_path)| {
                let semaphore = semaphore.clone();
                let progress = progress.clone();
                let extractor = self.clone_for_task();
                
                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    let result = extractor.process_single_file(&pdf_path, index + 1).await;
                    progress.increment();
                    result
                })
            })
            .collect();
        
        // Wait for all tasks and collect results
        let mut results = Vec::new();
        for task in tasks {
            match task.await? {
                Ok(result) => results.push(result),
                Err(e) => error!("Task failed: {}", e),
            }
        }
        
        // Batch insert into database
        if !results.is_empty() {
            info!("Inserting {} results into database", results.len());
            db.batch_insert(&results).await?;
        }
        
        progress.finish();
        Ok(results)
    }
    
    /// Process a single PDF file
    async fn process_single_file(&self, pdf_path: &Path, file_index: usize) -> Result<ExtractionResult> {
        let start_time = Instant::now();
        let relative_path = pdf_path.to_string_lossy().to_string();
        
        info!("Processing ({}): {}", file_index, relative_path);
        
        // Get file metadata
        let metadata = tokio::fs::metadata(pdf_path).await?;
        let file_size = metadata.len() as i64;
        let file_hash = self.calculate_file_hash(pdf_path).await?;
        
        // Check if already processed (unless force flag is set)
        if !self.args.force {
            // This would require database access - simplified for now
        }
        
        // Choose extraction method based on flags
        let (text, page_count, method) = if self.args.ocr_only {
            // Skip direct extraction, use OCR only
            match self.ocr_processor.extract_text_ocr(pdf_path).await {
                Ok((ocr_text, ocr_page_count)) => {
                    info!("Used OCR only for {}", relative_path);
                    (ocr_text, ocr_page_count, "ocr".to_string())
                }
                Err(e) => {
                    error!("OCR extraction failed for {}: {}", relative_path, e);
                    return Ok(ExtractionResult {
                        id: None,
                        file_path: relative_path,
                        file_hash: Some(file_hash),
                        file_size,
                        extraction_method: "error".to_string(),
                        extracted_text: String::new(),
                        page_count: 0,
                        processing_time_seconds: start_time.elapsed().as_secs_f64(),
                        timestamp: chrono::Utc::now(),
                        success: false,
                        error_message: Some(format!("OCR failed: {}", e)),
                    });
                }
            }
        } else {
            // Try direct text extraction first (default behavior)
            match self.pdf_processor.extract_text_direct(pdf_path) {
                Ok((text, page_count)) => {
                    if self.pdf_processor.has_extractable_text(&text) {
                        info!("Extracted text directly from {}", relative_path);
                        (text, page_count, "direct".to_string())
                    } else {
                        // Fall back to OCR
                        match self.ocr_processor.extract_text_ocr(pdf_path).await {
                            Ok((ocr_text, ocr_page_count)) => {
                                info!("Used OCR for {}", relative_path);
                                (ocr_text, ocr_page_count, "ocr".to_string())
                            }
                            Err(e) => {
                                warn!("OCR failed for {}: {}", relative_path, e);
                                (text, page_count, "direct_partial".to_string())
                            }
                        }
                    }
                }
                Err(e) => {
                    // Try OCR as last resort
                    match self.ocr_processor.extract_text_ocr(pdf_path).await {
                        Ok((ocr_text, ocr_page_count)) => {
                            info!("Used OCR after direct extraction failed for {}", relative_path);
                            (ocr_text, ocr_page_count, "ocr".to_string())
                        }
                        Err(ocr_e) => {
                            error!("Both direct and OCR extraction failed for {}: direct={}, ocr={}", 
                                   relative_path, e, ocr_e);
                            return Ok(ExtractionResult {
                                id: None,
                                file_path: relative_path,
                                file_hash: Some(file_hash),
                                file_size,
                                extraction_method: "error".to_string(),
                                extracted_text: String::new(),
                                page_count: 0,
                                processing_time_seconds: start_time.elapsed().as_secs_f64(),
                                timestamp: chrono::Utc::now(),
                                success: false,
                                error_message: Some(format!("Direct: {}, OCR: {}", e, ocr_e)),
                            });
                        }
                    }
                }
            }
        };
        
        let processing_time = start_time.elapsed().as_secs_f64();
        
        Ok(ExtractionResult {
            id: None,
            file_path: relative_path,
            file_hash: Some(file_hash),
            file_size,
            extraction_method: method,
            extracted_text: text,
            page_count: page_count as i32,
            processing_time_seconds: processing_time,
            timestamp: chrono::Utc::now(),
            success: true,
            error_message: None,
        })
    }
    
    /// Calculate file hash (fast or full based on settings)
    async fn calculate_file_hash(&self, file_path: &Path) -> Result<String> {
        if self.args.full_hash {
            self.calculate_full_hash(file_path).await
        } else {
            self.calculate_fast_hash(file_path).await
        }
    }
    
    /// Calculate fast hash (metadata + first/last chunks)
    async fn calculate_fast_hash(&self, file_path: &Path) -> Result<String> {
        use sha2::{Sha256, Digest};
        
        let metadata = tokio::fs::metadata(file_path).await?;
        let mut hasher = Sha256::new();
        
        // Hash metadata
        hasher.update(metadata.len().to_be_bytes());
        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                hasher.update(duration.as_secs().to_be_bytes());
            }
        }
        
        // Hash first and last 1KB
        let file_content = tokio::fs::read(file_path).await?;
        let file_len = file_content.len();
        
        if file_len > 0 {
            let first_chunk_end = std::cmp::min(1024, file_len);
            hasher.update(&file_content[0..first_chunk_end]);
            
            if file_len > 1024 {
                let last_chunk_start = std::cmp::max(1024, file_len - 1024);
                hasher.update(&file_content[last_chunk_start..]);
            }
        }
        
        Ok(format!("{:x}", hasher.finalize()))
    }
    
    /// Calculate full file hash
    async fn calculate_full_hash(&self, file_path: &Path) -> Result<String> {
        use sha2::{Sha256, Digest};
        
        let content = tokio::fs::read(file_path).await?;
        let hash = Sha256::digest(&content);
        Ok(format!("{:x}", hash))
    }
    
    /// Create a clone suitable for async tasks
    fn clone_for_task(&self) -> Self {
        Self {
            args: self.args.clone(),
            pdf_processor: PdfProcessor::new(),
            ocr_processor: OcrProcessor::new(self.args.text_only, self.args.use_gpu),
        }
    }
}