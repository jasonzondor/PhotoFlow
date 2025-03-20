#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    use std::path::PathBuf;
    use tracing::info;

    fn setup_test_image(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test_data")
            .join(name)
    }

    #[test]
    fn test_image_loading_performance() {
        // Initialize logging
        let _ = tracing_subscriber::fmt::try_init();

        // Test image paths
        let jpeg_path = setup_test_image("test.jpg");
        let raw_path = setup_test_image("test.raf");

        // Test JPEG loading performance
        info!("Testing JPEG loading performance...");
        let start = Instant::now();
        for _ in 0..5 {
            let photo = Photo::new(&jpeg_path).expect("Failed to load JPEG");
            assert!(photo.image.is_some());
        }
        let jpeg_time = start.elapsed();
        info!("JPEG loading time (5 iterations): {:?}", jpeg_time);

        // Test cache hit performance
        info!("Testing cache hit performance...");
        let start = Instant::now();
        for _ in 0..5 {
            let photo = Photo::new(&jpeg_path).expect("Failed to load JPEG");
            assert!(photo.image.is_some());
        }
        let cache_time = start.elapsed();
        info!("Cache hit time (5 iterations): {:?}", cache_time);
        assert!(cache_time < jpeg_time, "Cache should be faster than loading from disk");

        // Test RAW loading performance
        if raw_path.exists() {
            info!("Testing RAW loading performance...");
            let start = Instant::now();
            let photo = Photo::new(&raw_path).expect("Failed to load RAW");
            assert!(photo.image.is_some());
            let raw_time = start.elapsed();
            info!("RAW loading time: {:?}", raw_time);
        }
    }
}
