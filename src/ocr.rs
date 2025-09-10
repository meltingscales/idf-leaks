use anyhow::{Result, Context};
use std::path::Path;
use log::{info, warn, error};

pub struct OcrProcessor {
    text_only: bool,
}

impl OcrProcessor {
    pub fn new(text_only: bool) -> Self {
        Self { text_only }
    }
    
    /// Perform OCR on PDF file
    pub async fn extract_text_ocr(&self, pdf_path: &Path) -> Result<(String, usize)> {
        if self.text_only {
            return Err(anyhow::anyhow!("OCR disabled (text-only mode)"));
        }
        
        info!("Starting OCR for {}", pdf_path.display());
        
        // For now, we'll use a simplified approach since tesseract-rs binding can be complex
        // In a real implementation, you'd:
        // 1. Convert PDF pages to images using pdf2image equivalent
        // 2. Run Tesseract OCR on each image
        // 3. Combine results
        
        // Placeholder implementation - would need proper PDF to image conversion
        // and Tesseract integration
        
        self.extract_with_system_tesseract(pdf_path).await
    }
    
    /// Use system tesseract command as fallback
    async fn extract_with_system_tesseract(&self, pdf_path: &Path) -> Result<(String, usize)> {
        use tokio::process::Command;
        
        // First convert PDF to images using pdftoppm
        let temp_dir = tempfile::tempdir()?;
        let image_prefix = temp_dir.path().join("page");
        
        let output = Command::new("pdftoppm")
            .args([
                "-png",
                pdf_path.to_str().unwrap(),
                image_prefix.to_str().unwrap(),
            ])
            .output()
            .await
            .context("Failed to run pdftoppm")?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "pdftoppm failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        // Find generated image files
        let mut image_files = Vec::new();
        let mut entries = tokio::fs::read_dir(temp_dir.path()).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("png") {
                image_files.push(path);
            }
        }
        
        image_files.sort();
        let page_count = image_files.len();
        
        if image_files.is_empty() {
            return Err(anyhow::anyhow!("No images generated from PDF"));
        }
        
        // Run OCR on each image
        let mut full_text = String::new();
        
        for (page_num, image_path) in image_files.iter().enumerate() {
            match self.ocr_image(image_path).await {
                Ok(text) => {
                    if !text.trim().is_empty() {
                        full_text.push_str(&format!("--- Page {} (OCR) ---\n", page_num + 1));
                        full_text.push_str(&text);
                        full_text.push_str("\n\n");
                    }
                }
                Err(e) => {
                    warn!("OCR failed for page {}: {}", page_num + 1, e);
                }
            }
        }
        
        Ok((full_text, page_count))
    }
    
    /// OCR a single image file
    async fn ocr_image(&self, image_path: &Path) -> Result<String> {
        use tokio::process::Command;
        
        let output = Command::new("tesseract")
            .args([
                image_path.to_str().unwrap(),
                "stdout",
                "-l", "eng",
            ])
            .output()
            .await
            .context("Failed to run tesseract")?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow::anyhow!(
                "Tesseract failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
    
    /// Check if OCR tools are available
    pub async fn check_ocr_availability(&self) -> bool {
        if self.text_only {
            return false;
        }
        
        let tesseract_available = tokio::process::Command::new("tesseract")
            .arg("--version")
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false);
        
        let pdftoppm_available = tokio::process::Command::new("pdftoppm")
            .arg("-h")
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false);
        
        if !tesseract_available {
            error!("Tesseract not found. Install with: apt install tesseract-ocr");
        }
        
        if !pdftoppm_available {
            error!("pdftoppm not found. Install with: apt install poppler-utils");
        }
        
        tesseract_available && pdftoppm_available
    }
}