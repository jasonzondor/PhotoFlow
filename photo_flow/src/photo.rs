use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use image::DynamicImage;
use rawloader::{decode_file, RawImageData};
use exif::{Reader, Tag, Value};
use std::fs::File;
use std::io::BufReader;
use tracing::{debug, error, info};

#[derive(Debug, Clone)]
pub struct Photo {
    path: PathBuf,
    exif_data: Option<ExifData>,
    pub image: Option<DynamicImage>,
}

#[derive(Debug, Clone)]
pub struct ExifData {
    pub make: Option<String>,
    pub model: Option<String>,
    pub exposure_time: Option<String>,
    pub f_number: Option<f32>,
    pub iso: Option<u32>,
    pub focal_length: Option<f32>,
    pub datetime: Option<String>,
}

impl Photo {
    pub fn new(path: PathBuf) -> Result<Self> {
        let mut photo = Self {
            path,
            exif_data: None,
            image: None,
        };
        
        if let Err(e) = photo.load_exif() {
            debug!("Failed to load EXIF data: {}", e);
            // Continue even if EXIF loading fails
        }
        Ok(photo)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn exif_data(&self) -> Option<&ExifData> {
        self.exif_data.as_ref()
    }

    pub fn set_image(&mut self, image: DynamicImage) {
        self.image = Some(image);
    }

    pub fn load_image(&self) -> Result<DynamicImage> {
        info!("Loading image: {}", self.path.display());
        let ext = self.path.extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());

        let ext = ext.as_deref().unwrap_or("").to_lowercase();
        info!("Loading image with extension: {}", ext);

        if ext.is_empty() {
            error!("No file extension found for: {}", self.path.display());
            return Err(anyhow::anyhow!("No file extension found"));
        }
        
        Ok(match ext.as_str() {
            "raf" => {
                info!("Loading Fuji RAF file");
                self.load_raw_image()
            }
            "raw" | "cr2" | "nef" | "arw" | "rw2" | "dng" | "orf" => {
                info!("Loading other RAW format: {}", ext);
                self.load_raw_image()
            }
            _ => {
                info!("Loading as regular image file");
                image::open(&self.path).context("Failed to open regular image file")
            }
        }?)
    }

    fn load_raw_image(&self) -> Result<DynamicImage> {
        info!("Loading RAW image: {}", self.path.display());
        
        // Verify file exists and is readable
        if !self.path.exists() {
            error!("RAW file does not exist: {}", self.path.display());
            return Err(anyhow::anyhow!("RAW file does not exist"));
        }
        
        // Create a temporary directory for the PPM output
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join(format!("{}.ppm", 
            self.path.file_stem().unwrap_or_default().to_string_lossy()));
        
        info!("Using dcraw to convert RAW to PPM: {}", output_path.display());
        
        // Use dcraw to convert RAF to PPM
        let status = std::process::Command::new("dcraw")
            .arg("-c")  // Write to standard output
            .arg("-w")  // Use camera white balance
            .arg("-q", 3)  // Use high-quality interpolation
            .arg("-T")  // Write TIFF with metadata
            .arg(self.path.as_os_str())
            .output()
            .context("Failed to run dcraw")?;
            
        if !status.status.success() {
            let error = String::from_utf8_lossy(&status.stderr);
            error!("dcraw failed: {}", error);
            return Err(anyhow::anyhow!("dcraw failed: {}", error));
        }
        
        // Read the PPM data from stdout
        info!("Reading PPM data from dcraw output");
        let img = image::load_from_memory_with_format(&status.stdout, image::ImageFormat::Ppm)
            .context("Failed to load PPM data from dcraw output")?;
            
        info!("Successfully loaded RAW image: {}x{}", img.width(), img.height());
        Ok(img)

        debug!("Loading RAW file: {}", self.path.display());
        let raw_image = match decode_file(&self.path) {
            Ok(img) => {
                info!("RAW image decoded successfully:");
                info!("  - Dimensions: {}x{}", img.width, img.height);
                info!("  - Data type: {:?}", img.data);
                info!("  - Make: {}", img.make);
                info!("  - Model: {}", img.model);
                info!("  - Clean Make: {}", img.clean_make);
                info!("  - Clean Model: {}", img.clean_model);
                info!("  - Color format: {} cpp", img.cpp);
                img
            },
            Err(e) => {
                error!("Failed to decode RAW file {}: {}", self.path.display(), e);
                error!("RAW decoder error details: {:?}", e);
                return Err(anyhow::anyhow!("Failed to decode RAW file: {}", e));
            }
        };
        
        let width = raw_image.width as u32;
        let height = raw_image.height as u32;
        
        // Convert raw image data to RGB
        info!("Converting RAW data to RGB...");
        let rgb_data = match raw_image.data {
            RawImageData::Integer(data) => {
                debug!("Converting integer RAW data");
                let mut rgb = Vec::with_capacity(width as usize * height as usize * 3);
                let max_value = data.iter().copied().max().unwrap_or(65535) as f32;
                debug!("Max value in RAW data: {}", max_value);
                
                // Apply gamma correction for better visual appearance
                let gamma = 2.2;
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
                debug!("Max value in RAW data: {}", max_value);
                
                // Apply gamma correction for better visual appearance
                let gamma = 2.2;
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



    fn load_exif(&mut self) -> Result<()> {
        debug!("Loading EXIF data from: {:?}", self.path);
        let file = File::open(&self.path)?;
        let mut bufreader = BufReader::new(&file);
        let exif = Reader::new().read_from_container(&mut bufreader)?;

        let mut data = ExifData {
            make: None,
            model: None,
            exposure_time: None,
            f_number: None,
            iso: None,
            focal_length: None,
            datetime: None,
        };

        // Process all fields
        for field in exif.fields() {
            debug!("Found EXIF field: {:?} = {:?}", field.tag, field.value);
            match field.tag {
                Tag::Make => {
                    data.make = Some(field.value.display_as(field.tag).to_string());
                }
                Tag::Model => {
                    data.model = Some(field.value.display_as(field.tag).to_string());
                }
                Tag::ExposureTime => {
                    if let Value::Rational(rationals) = &field.value {
                        if let Some(r) = rationals.first() {
                            data.exposure_time = Some(format!("{}/{}", r.num, r.denom));
                        }
                    }
                }
                Tag::FNumber => {
                    if let Value::Rational(rationals) = &field.value {
                        if let Some(r) = rationals.first() {
                            data.f_number = Some(r.num as f32 / r.denom as f32);
                        }
                    }
                }
                Tag::ISOSpeed => {
                    if let Value::Short(v) = &field.value {
                        data.iso = v.first().map(|&x| x as u32);
                    }
                }
                Tag::FocalLength => {
                    if let Value::Rational(rationals) = &field.value {
                        if let Some(r) = rationals.first() {
                            data.focal_length = Some(r.num as f32 / r.denom as f32);
                        }
                    }
                }
                Tag::DateTimeOriginal => {
                    data.datetime = Some(field.value.display_as(field.tag).to_string());
                }
                _ => {}
            }
        }

        debug!("Extracted EXIF data: {:?}", data);
        self.exif_data = Some(data);
        Ok(())
    }
}
