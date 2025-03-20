use std::path::{Path, PathBuf};
use std::sync::Arc;
use anyhow::Result;
use image::DynamicImage;
use exif::{Reader, Tag, Value};
use std::fs::File;
use std::io::BufReader;
use tracing::{debug, info};
use std::time::SystemTime;
use lru::LruCache;
use parking_lot::Mutex;
use once_cell::sync::Lazy;

use crate::processors;

// Cache for loaded images
static IMAGE_CACHE: Lazy<Arc<Mutex<LruCache<PathBuf, (DynamicImage, SystemTime)>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(LruCache::new(std::num::NonZeroUsize::new(32).unwrap())))); // Cache up to 32 images

#[derive(Debug, Clone)]
pub struct Photo {
    path: PathBuf,
    exif_data: Option<ExifData>,
    pub image: Option<DynamicImage>,
    rgb_data: Option<Vec<u8>>,
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
            rgb_data: None,
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
        // Convert to RGB8 once and cache it
        let rgb = image.to_rgb8();
        self.rgb_data = Some(rgb.to_vec());
        self.image = Some(image);
    }

    pub fn get_rgb_data(&self) -> Vec<u8> {
        if let Some(data) = &self.rgb_data {
            data.clone()
        } else if let Some(img) = &self.image {
            let rgb = img.to_rgb8();
            rgb.to_vec()
        } else {
            Vec::new()
        }
    }

    pub fn load_image(&self) -> Result<DynamicImage> {
        info!("Loading image: {}", self.path.display());
        
        // Try to load from cache first
        if let Some((cached_image, cached_time)) = IMAGE_CACHE.lock().get(&self.path).cloned() {
            // Check if file has been modified
            if let Ok(metadata) = std::fs::metadata(&self.path) {
                if let Ok(modified) = metadata.modified() {
                    if modified <= cached_time {
                        debug!("Loading image from cache: {}", self.path.display());
                        return Ok(cached_image);
                    }
                }
            }
        }
        
        // Not in cache, load using processor
        let processor = processors::get_processor(&self.path);
        let image = processor.load_image(&self.path)?;
        
        // Add to cache
        if let Ok(metadata) = std::fs::metadata(&self.path) {
            if let Ok(modified) = metadata.modified() {
                IMAGE_CACHE.lock().put(self.path.clone(), (image.clone(), modified));
            }
        }
        
        Ok(image)
    }


    fn load_exif(&mut self) -> Result<()> {
        debug!("Loading metadata from: {:?}", self.path);
        
        // First try to load metadata from rawloader for RAW files
        if let Ok(raw_image) = rawloader::decode_file(&self.path) {
            debug!("Got metadata from rawloader");
            let data = ExifData {
                make: Some(raw_image.make),
                model: Some(raw_image.model),
                exposure_time: None, // TODO: Add these from rawloader
                f_number: None,
                iso: None,
                focal_length: None,
                datetime: None,
            };
            self.exif_data = Some(data);
            return Ok(());
        }
        
        // Fall back to EXIF for non-RAW files
        debug!("Falling back to EXIF parser");
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
