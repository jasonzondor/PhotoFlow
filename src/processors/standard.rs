use std::path::Path;
use anyhow::{Context, Result};
use image::DynamicImage;
use tracing::info;

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
        image::open(path).context("Failed to open image file")
    }
}
