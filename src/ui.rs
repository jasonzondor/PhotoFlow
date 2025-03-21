use iced::{
    advanced::image::Handle,
    widget::{column, container, text, Image},
    Element, Length,
};

use crate::photo::Photo;
use crate::Message;

#[derive(Debug, Default)]
pub struct PhotoView {}

impl PhotoView {
    pub fn new() -> Self {
        Self {}
    }

    pub fn view(&self, photo: &Photo) -> Element<Message> {
        let mut info = column![];

        // Add filename
        info = info.push(
            text(format!("File: {}", photo.path().file_name().unwrap_or_default().to_string_lossy()))
                .size(16),
        );

        // Add EXIF data if available
        if let Some(exif) = photo.exif_data() {
            let make_model = match (exif.make.as_ref(), exif.model.as_ref()) {
                (Some(make), Some(model)) => format!("{} {}", make, model),
                (Some(make), None) => make.clone(),
                (None, Some(model)) => model.clone(),
                (None, None) => String::from("Unknown Camera"),
            };
            info = info.push(text(make_model));

            if let Some(datetime) = &exif.datetime {
                info = info.push(text(format!("Date: {}", datetime)));
            }

            let mut settings = Vec::new();
            if let Some(exposure) = &exif.exposure_time {
                settings.push(format!("{}s", exposure));
            }
            if let Some(f_number) = exif.f_number {
                settings.push(format!("f/{:.1}", f_number));
            }
            if let Some(iso) = exif.iso {
                settings.push(format!("ISO {}", iso));
            }
            if let Some(focal_length) = exif.focal_length {
                settings.push(format!("{}mm", focal_length));
            }

            if !settings.is_empty() {
                info = info.push(text(settings.join(" • ")));
            }
        }

        // Create the image widget
        let image_widget = if let Some(img) = &photo.image {
            // The image should already be in RGB8 format
            Image::new(Handle::from_memory(photo.get_rgb_data()))
                .width(Length::Fill)
                .height(Length::Fill)
        } else {
            Image::new(Handle::from_memory(Vec::new()))
                .width(Length::Fill)
                .height(Length::Fill)
        };

        let content = column!(info, image_widget).spacing(20);

        container(content)
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
