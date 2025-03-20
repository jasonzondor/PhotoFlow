use std::path::Path;
use anyhow::{Context, Result};
use image::DynamicImage;
use rawloader::{decode_file, RawImageData};
use tracing::{info, debug, error};

use super::{ImageProcessor, detector::{self, ImageType}};

pub struct RawProcessor;

impl RawProcessor {
    pub fn new() -> Self {
        RawProcessor
    }
}

impl ImageProcessor for RawProcessor {
    fn can_handle(&self, path: &Path) -> bool {
        match detector::detect_image_type(path) {
            Ok(image_type) => image_type.is_raw(),
            Err(_) => false,
        }
    }
    
    fn load_image(&self, path: &Path) -> Result<DynamicImage> {
        info!("Loading RAW image: {}", path.display());
        
        if !path.exists() {
            error!("RAW file does not exist: {}", path.display());
            return Err(anyhow::anyhow!("RAW file does not exist"));
        }
        
        // Get the specific RAW format
        let image_type = detector::detect_image_type(path)?;
        info!("Detected RAW format: {:?}", image_type);
        
        // Configure rawloader based on the RAW format
        debug!("Decoding RAW file with format-specific settings...");
        let raw_image = decode_file(path)
            .context("Failed to decode RAW file")?;
            
        info!("RAW image decoded successfully:");
        info!("  - Dimensions: {}x{}", raw_image.width, raw_image.height);
        info!("  - Make: {}", raw_image.make);
        info!("  - Model: {}", raw_image.model);
        
        let width = raw_image.width as u32;
        let height = raw_image.height as u32;
        
        // Convert raw image data to RGB with format-specific adjustments
        info!("Converting RAW data to RGB...");
        let rgb_data = match raw_image.data {
            RawImageData::Integer(data) => {
                debug!("Converting integer RAW data");
                let mut rgb = Vec::with_capacity(width as usize * height as usize * 3);
                let max_value = data.iter().copied().max().unwrap_or(65535) as f32;
                
                // Apply gamma correction with format-specific adjustments
                let gamma = match image_type {
                    ImageType::RawFuji => 2.4,  // Fuji typically needs slightly higher gamma
                    ImageType::RawCanon => 2.1, // Canon typically needs slightly lower gamma
                    _ => 2.2,  // Standard gamma for other formats
                };
                
                for value in data {
                    // Convert to 8-bit with gamma correction
                    let normalized = (value as f32 / max_value).powf(1.0 / gamma);
                    let v = (normalized * 255.0) as u8;
                    rgb.extend_from_slice(&[v, v, v]);
                }
                rgb
            },
            RawImageData::Float(data) => {
                debug!("Converting float RAW data");
                let mut rgb = Vec::with_capacity(width as usize * height as usize * 3);
                let max_value = data.iter().copied().fold(0.0, f32::max);
                
                // Apply gamma correction with format-specific adjustments
                let gamma = match image_type {
                    ImageType::RawFuji => 2.4,
                    ImageType::RawCanon => 2.1,
                    _ => 2.2,
                };
                
                for value in data {
                    // Convert to 8-bit with gamma correction
                    let normalized = (value / max_value).powf(1.0 / gamma);
                    let v = (normalized * 255.0) as u8;
                    rgb.extend_from_slice(&[v, v, v]);
                }
                rgb
            },
        };
        
        debug!("Creating RGB image from RAW data");
        let rgb_image = image::RgbImage::from_raw(width, height, rgb_data)
            .context("Failed to create image from raw data")?;
            
        debug!("Successfully created RGB image: {}x{}", width, height);
        Ok(DynamicImage::ImageRgb8(rgb_image))
    }
}
