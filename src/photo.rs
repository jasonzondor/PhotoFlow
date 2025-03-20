use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use image::DynamicImage;
use exif::{Reader, Tag, Value};
use std::fs::File;
use std::io::BufReader;
use tracing::{debug, error, info};

use crate::processors;

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
        
        // Get appropriate processor for this file type
        let processor = processors::get_processor(&self.path);
        
        // Load the image using the processor
        processor.load_image(&self.path)
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
