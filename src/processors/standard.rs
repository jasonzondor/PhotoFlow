use std::path::Path;
use anyhow::{Context, Result};
use image::DynamicImage;
use tracing::{info, debug};
use std::fs::File;
use memmap2::Mmap;
use std::io::BufReader;

// Size threshold for using memory mapping (32MB)
const MMAP_THRESHOLD: u64 = 32 * 1024 * 1024;

use super::{ImageProcessor, detector::{self, ImageType}};

pub struct StandardProcessor;

impl StandardProcessor {
    pub fn new() -> Self {
        StandardProcessor
    }
}

impl ImageProcessor for StandardProcessor {
    fn can_handle(&self, path: &Path) -> bool {
        match detector::detect_image_type(path) {
            Ok(image_type) => matches!(
                image_type,
                ImageType::Jpeg
                    | ImageType::Png
                    | ImageType::Gif
                    | ImageType::WebP
                    | ImageType::Tiff
            ),
            Err(_) => false,
        }
    }
    
    fn load_image(&self, path: &Path) -> Result<DynamicImage> {
        info!("Loading standard image: {}", path.display());
        
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        
        // Use memory mapping for large files
        if metadata.len() > MMAP_THRESHOLD {
            debug!("Using memory mapping for large image: {} bytes", metadata.len());
            let mmap = unsafe { Mmap::map(&file)? };
            image::load_from_memory(&mmap).context("Failed to load image from memory map")
        } else {
            // Use buffered reader for smaller files
            let reader = BufReader::new(file);
            image::load(reader, image::ImageFormat::from_path(path)?)
                .context("Failed to load image from file")
        }
    }
}
