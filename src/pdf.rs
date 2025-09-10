use anyhow::{Result, Context};
use std::path::Path;
use log::{info, warn};

pub struct PdfProcessor;

impl PdfProcessor {
    pub fn new() -> Self {
        Self
    }
    
    /// Extract text directly from PDF using lopdf
    pub fn extract_text_direct(&self, pdf_path: &Path) -> Result<(String, usize)> {
        let document = lopdf::Document::load(pdf_path)
            .context("Failed to load PDF document")?;
        
        let mut text = String::new();
        let page_count = document.get_pages().len();
        
        for (page_num, _) in document.get_pages() {
            match document.extract_text(&[page_num]) {
                Ok(page_text) => {
                    if !page_text.trim().is_empty() {
                        text.push_str(&format!("--- Page {} ---\n", page_num));
                        text.push_str(&page_text);
                        text.push_str("\n\n");
                    }
                }
                Err(e) => {
                    warn!("Failed to extract text from page {}: {}", page_num, e);
                }
            }
        }
        
        // Try pdf-extract as fallback if lopdf didn't work well
        if text.trim().len() < 50 {
            match pdf_extract::extract_text(pdf_path) {
                Ok(extracted) => {
                    if extracted.trim().len() > text.trim().len() {
                        info!("Using pdf-extract fallback for {}", pdf_path.display());
                        return Ok((extracted, page_count));
                    }
                }
                Err(e) => {
                    warn!("pdf-extract fallback failed: {}", e);
                }
            }
        }
        
        Ok((text, page_count))
    }
    
    /// Check if PDF has substantial extractable text
    pub fn has_extractable_text(&self, text: &str) -> bool {
        text.trim().len() > 50
    }
    
    /// Get PDF page count
    pub fn get_page_count(&self, pdf_path: &Path) -> Result<usize> {
        let document = lopdf::Document::load(pdf_path)
            .context("Failed to load PDF for page count")?;
        
        Ok(document.get_pages().len())
    }
}