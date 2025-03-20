pub mod raw;
pub mod standard;
pub mod detector;
#[cfg(test)]
mod tests;

use std::path::Path;
use anyhow::Result;
use image::DynamicImage;
use tracing::{debug, error};

/// Trait for image processors
pub trait ImageProcessor {
    /// Check if this processor can handle the given file
    fn can_handle(&self, path: &Path) -> bool;
    
    /// Load and process the image
    fn load_image(&self, path: &Path) -> Result<DynamicImage>;
}

/// Factory for creating appropriate image processors based on file type detection
pub fn get_processor(path: &Path) -> Box<dyn ImageProcessor> {
    match detector::detect_image_type(path) {
        Ok(image_type) => {
            debug!("Detected image type: {:?}", image_type);
            if image_type.is_raw() {
                Box::new(raw::RawProcessor::new())
            } else {
                Box::new(standard::StandardProcessor::new())
            }
        },
        Err(e) => {
            error!("Failed to detect image type: {}", e);
            // Fall back to standard processor if detection fails
            Box::new(standard::StandardProcessor::new())
        }
    }
}
