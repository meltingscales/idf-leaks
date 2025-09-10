use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone)]
pub struct ProgressTracker {
    bar: ProgressBar,
    completed: Arc<AtomicUsize>,
}

impl ProgressTracker {
    pub fn new(total: usize) -> Self {
        let bar = ProgressBar::new(total as u64);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
                .expect("Failed to set progress bar template")
                .progress_chars("#>-")
        );
        
        bar.set_message("Processing PDFs...");
        
        Self {
            bar,
            completed: Arc::new(AtomicUsize::new(0)),
        }
    }
    
    pub fn increment(&self) {
        let count = self.completed.fetch_add(1, Ordering::SeqCst) + 1;
        self.bar.set_position(count as u64);
        self.bar.set_message(format!("Processed {} files", count));
    }
    
    pub fn finish(&self) {
        self.bar.finish_with_message("✅ All files processed!");
    }
}