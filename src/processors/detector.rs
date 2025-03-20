use std::path::Path;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use anyhow::{Context, Result};
use tracing::debug;

#[derive(Debug, PartialEq)]
pub enum ImageType {
    Jpeg,
    Png,
    Gif,
    Tiff,
    WebP,
    RawFuji,    // RAF
    RawCanon,   // CR2/CR3
    RawNikon,   // NEF
    RawSony,    // ARW
    RawPanasonic, // RW2
    RawGeneric, // Other RAW formats
    Unknown,
}

impl ImageType {
    pub fn is_raw(&self) -> bool {
        matches!(
            self,
            ImageType::RawFuji
                | ImageType::RawCanon
                | ImageType::RawNikon
                | ImageType::RawSony
                | ImageType::RawPanasonic
                | ImageType::RawGeneric
        )
    }
}

pub fn detect_image_type(path: &Path) -> Result<ImageType> {
    let mut file = File::open(path).context("Failed to open file for type detection")?;
    let mut buffer = [0u8; 16]; // Most magic numbers are within first 16 bytes
    
    file.read_exact(&mut buffer).context("Failed to read file header")?;
    
    // First check for common image formats
    if &buffer[0..2] == b"\xFF\xD8" {
        debug!("Detected JPEG format");
        return Ok(ImageType::Jpeg);
    }
    
    if &buffer[0..8] == b"\x89PNG\r\n\x1A\n" {
        debug!("Detected PNG format");
        return Ok(ImageType::Png);
    }
    
    if &buffer[0..3] == b"GIF" {
        debug!("Detected GIF format");
        return Ok(ImageType::Gif);
    }
    
    if &buffer[0..4] == b"RIFF" && &buffer[8..12] == b"WEBP" {
        debug!("Detected WebP format");
        return Ok(ImageType::WebP);
    }
    
    // Check for TIFF (both little and big endian)
    if (&buffer[0..4] == b"MM\x00*" || &buffer[0..4] == b"II*\x00") {
        debug!("Detected TIFF format");
        return Ok(ImageType::Tiff);
    }
    
    // Now check for various RAW formats
    
    // Fuji RAF
    if &buffer[0..4] == b"FUJI" {
        debug!("Detected Fuji RAF format");
        return Ok(ImageType::RawFuji);
    }
    
    // Canon CR2/CR3
    if &buffer[8..11] == b"CR\x02" || &buffer[8..11] == b"CR\x03" {
        debug!("Detected Canon RAW format");
        return Ok(ImageType::RawCanon);
    }
    
    // Nikon NEF (usually starts with TIFF header)
    if (&buffer[0..4] == b"MM\x00*" || &buffer[0..4] == b"II*\x00") {
        // Need to check deeper in the file for Nikon specific markers
        let mut extended_buffer = [0u8; 4096];
        file.seek(SeekFrom::Start(0))?;
        file.read_exact(&mut extended_buffer)?;
        
        if extended_buffer.windows(4).any(|window| window == b"NIKON") {
            debug!("Detected Nikon NEF format");
            return Ok(ImageType::RawNikon);
        }
    }
    
    // Sony ARW
    if &buffer[0..4] == b"SONY" {
        debug!("Detected Sony ARW format");
        return Ok(ImageType::RawSony);
    }
    
    // Panasonic RW2
    if &buffer[0..4] == b"IIU\x00" {
        debug!("Detected Panasonic RW2 format");
        return Ok(ImageType::RawPanasonic);
    }
    
    // Generic RAW check (look for common RAW markers)
    let raw_markers = [
        b"CIFF", // Canon old format
        b"HEIC", // New format that might contain RAW
        b"DNGK", // DNG marker
        b"EPAK", // Some Sigma cameras
    ];
    
    for marker in raw_markers.iter() {
        if buffer.windows(marker.len()).any(|window| window == *marker) {
            debug!("Detected generic RAW format");
            return Ok(ImageType::RawGeneric);
        }
    }
    
    debug!("Unknown image format");
    Ok(ImageType::Unknown)
}
