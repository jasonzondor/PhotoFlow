use std::path::Path;
use anyhow::{Context, Result};
use image::DynamicImage;
use rawloader::{decode_file, RawImageData};
use tracing::{info, debug, error};

use crate::photo::ExifData;
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
        // Update EXIF data from rawloader metadata
        let _exif = ExifData {
            make: Some(raw_image.make.clone()),
            model: Some(raw_image.model.clone()),
            exposure_time: None, // TODO: Add these from rawloader
            f_number: None,
            iso: None,
            focal_length: None,
            datetime: None,
        };
        
        let rgb_data = match raw_image.data {
            RawImageData::Integer(data) => {
                debug!("Converting integer RAW data");
                let mut rgb = Vec::with_capacity(width as usize * height as usize * 3);
                
                // Get black and white levels
                let black_level = raw_image.blacklevels[0] as f32;
                let white_level = raw_image.whitelevels[0] as f32;
                let range = white_level - black_level;
                debug!("Black level: {}, White level: {}, Range: {}", black_level, white_level, range);
                
                // Get white balance coefficients
                let wb_coeffs = raw_image.wb_coeffs;
                debug!("WB coeffs: R={}, G={}, B={}", wb_coeffs[0], wb_coeffs[1], wb_coeffs[2]);
                
                // Get CFA pattern info
                let cfa = raw_image.cfa.clone();
                debug!("CFA pattern: width={}, height={}", cfa.width, cfa.height);
                debug!("CFA pattern string: {}", raw_image.cfa.name);
                
                // Sample some raw values
                debug!("Raw value samples:");
                for y in [0, height as usize / 2, height as usize - 1] {
                    for x in [0, width as usize / 2, width as usize - 1] {
                        let pixel_idx = y * width as usize + x;
                        let raw_value = data[pixel_idx];
                        debug!("  ({}, {}): {}", x, y, raw_value);
                    }
                }
                
                // Create buffers for each color channel
                let mut red = vec![0.0f32; (width * height) as usize];
                let mut green = vec![0.0f32; (width * height) as usize];
                let mut blue = vec![0.0f32; (width * height) as usize];
                
                // First pass: Fill in known values
                for y in 0..height as usize {
                    for x in 0..width as usize {
                        let pixel_idx = y * width as usize + x;
                        let raw_value = data[pixel_idx] as f32;
                        
                        // Normalize value using black and white levels
                        let mut normalized = (raw_value - black_level) / range;
                        normalized = normalized.clamp(0.0, 1.0);
                        
                        // For X-Trans sensors, the pattern repeats every 6x6 pixels
                        let cfa_x = x % 6;
                        let cfa_y = y % 6;
                        
                        // X-Trans pattern (0=R, 1=G, 2=B)
                        let color = match (cfa_x, cfa_y) {
                            // Row 0
                            (0, 0) => 2, (1, 0) => 1, (2, 0) => 1, (3, 0) => 2, (4, 0) => 1, (5, 0) => 1,
                            // Row 1
                            (0, 1) => 1, (1, 1) => 2, (2, 1) => 0, (3, 1) => 1, (4, 1) => 0, (5, 1) => 2,
                            // Row 2
                            (0, 2) => 1, (1, 2) => 0, (2, 2) => 1, (3, 2) => 2, (4, 2) => 1, (5, 2) => 0,
                            // Row 3
                            (0, 3) => 2, (1, 3) => 1, (2, 3) => 1, (3, 3) => 2, (4, 3) => 1, (5, 3) => 1,
                            // Row 4
                            (0, 4) => 1, (1, 4) => 2, (2, 4) => 0, (3, 4) => 1, (4, 4) => 0, (5, 4) => 2,
                            // Row 5
                            (0, 5) => 1, (1, 5) => 0, (2, 5) => 1, (3, 5) => 2, (4, 5) => 1, (5, 5) => 0,
                            _ => unreachable!()
                        };
                        
                        // Apply white balance
                        let wb_coeff = match color {
                            0 => wb_coeffs[0], // Red
                            1 => wb_coeffs[1], // Green
                            2 => wb_coeffs[2], // Blue
                            _ => 1.0,
                        };
                        
                        let color_value = normalized * wb_coeff;
                        
                        // Store in appropriate channel
                        match color {
                            0 => red[pixel_idx] = color_value,
                            1 => green[pixel_idx] = color_value,
                            2 => blue[pixel_idx] = color_value,
                            _ => {},
                        }
                    }
                }
                
                // Sample some normalized values
                debug!("Normalized value samples after first pass:");
                for y in [0, height as usize / 2, height as usize - 1] {
                    for x in [0, width as usize / 2, width as usize - 1] {
                        let pixel_idx = y * width as usize + x;
                        debug!("  ({}, {}): R={:.3}, G={:.3}, B={:.3}", 
                            x, y, red[pixel_idx], green[pixel_idx], blue[pixel_idx]);
                    }
                }
                
                // Second pass: Simple bilinear interpolation for missing colors
                for y in 1..(height as usize - 1) {
                    for x in 1..(width as usize - 1) {
                        let pixel_idx = y * width as usize + x;
                        let cfa_x = x % cfa.width;
                        let cfa_y = y % cfa.height;
                        let color = cfa.color_at(cfa_x, cfa_y);
                        
                        // For each missing color at this pixel, average the neighbors
                        match color {
                            0 => { // Red pixel - interpolate G and B
                                if green[pixel_idx] == 0.0 {
                                    let neighbors = [
                                        green[pixel_idx - 1],
                                        green[pixel_idx + 1],
                                        green[pixel_idx - width as usize],
                                        green[pixel_idx + width as usize],
                                    ];
                                    let valid_count = neighbors.iter().filter(|&&v| v > 0.0).count();
                                    if valid_count > 0 {
                                        green[pixel_idx] = neighbors.iter().filter(|&&v| v > 0.0).sum::<f32>() / valid_count as f32;
                                    }
                                }
                                if blue[pixel_idx] == 0.0 {
                                    let neighbors = [
                                        blue[pixel_idx - 1 - width as usize],
                                        blue[pixel_idx - 1 + width as usize],
                                        blue[pixel_idx + 1 - width as usize],
                                        blue[pixel_idx + 1 + width as usize],
                                    ];
                                    let valid_count = neighbors.iter().filter(|&&v| v > 0.0).count();
                                    if valid_count > 0 {
                                        blue[pixel_idx] = neighbors.iter().filter(|&&v| v > 0.0).sum::<f32>() / valid_count as f32;
                                    }
                                }
                            },
                            1 => { // Green pixel - interpolate R and B
                                if red[pixel_idx] == 0.0 {
                                    let neighbors = [
                                        red[pixel_idx - 1],
                                        red[pixel_idx + 1],
                                        red[pixel_idx - width as usize],
                                        red[pixel_idx + width as usize],
                                    ];
                                    let valid_count = neighbors.iter().filter(|&&v| v > 0.0).count();
                                    if valid_count > 0 {
                                        red[pixel_idx] = neighbors.iter().filter(|&&v| v > 0.0).sum::<f32>() / valid_count as f32;
                                    }
                                }
                                if blue[pixel_idx] == 0.0 {
                                    let neighbors = [
                                        blue[pixel_idx - 1],
                                        blue[pixel_idx + 1],
                                        blue[pixel_idx - width as usize],
                                        blue[pixel_idx + width as usize],
                                    ];
                                    let valid_count = neighbors.iter().filter(|&&v| v > 0.0).count();
                                    if valid_count > 0 {
                                        blue[pixel_idx] = neighbors.iter().filter(|&&v| v > 0.0).sum::<f32>() / valid_count as f32;
                                    }
                                }
                            },
                            2 => { // Blue pixel - interpolate R and G
                                if red[pixel_idx] == 0.0 {
                                    let neighbors = [
                                        red[pixel_idx - 1 - width as usize],
                                        red[pixel_idx - 1 + width as usize],
                                        red[pixel_idx + 1 - width as usize],
                                        red[pixel_idx + 1 + width as usize],
                                    ];
                                    let valid_count = neighbors.iter().filter(|&&v| v > 0.0).count();
                                    if valid_count > 0 {
                                        red[pixel_idx] = neighbors.iter().filter(|&&v| v > 0.0).sum::<f32>() / valid_count as f32;
                                    }
                                }
                                if green[pixel_idx] == 0.0 {
                                    let neighbors = [
                                        green[pixel_idx - 1],
                                        green[pixel_idx + 1],
                                        green[pixel_idx - width as usize],
                                        green[pixel_idx + width as usize],
                                    ];
                                    let valid_count = neighbors.iter().filter(|&&v| v > 0.0).count();
                                    if valid_count > 0 {
                                        green[pixel_idx] = neighbors.iter().filter(|&&v| v > 0.0).sum::<f32>() / valid_count as f32;
                                    }
                                }
                            },
                            _ => {},
                        }
                    }
                }
                
                // Sample some normalized values after interpolation
                debug!("Normalized value samples after interpolation:");
                for y in [0, height as usize / 2, height as usize - 1] {
                    for x in [0, width as usize / 2, width as usize - 1] {
                        let pixel_idx = y * width as usize + x;
                        debug!("  ({}, {}): R={:.3}, G={:.3}, B={:.3}", 
                            x, y, red[pixel_idx], green[pixel_idx], blue[pixel_idx]);
                    }
                }
                
                // Final pass: Convert to RGB bytes with gamma correction
                let gamma = 2.2;
                for i in 0..(width * height) as usize {
                    let r = (red[i].powf(1.0 / gamma) * 255.0) as u8;
                    let g = (green[i].powf(1.0 / gamma) * 255.0) as u8;
                    let b = (blue[i].powf(1.0 / gamma) * 255.0) as u8;
                    rgb.extend_from_slice(&[r, g, b]);
                }
                rgb
            },
            RawImageData::Float(_data) => {
                // Similar process for float data
                vec![0; (width * height * 3) as usize] // TODO: Implement float handling
            },
        };
        
        debug!("Creating RGB image from RAW data");
        let rgb_image = image::RgbImage::from_raw(width, height, rgb_data)
            .context("Failed to create image from raw data")?;
            
        debug!("Successfully created RGB image: {}x{}", width, height);
        Ok(DynamicImage::ImageRgb8(rgb_image))
    }
}
