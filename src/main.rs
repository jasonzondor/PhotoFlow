use iced::{
    executor,
    widget::{button, column, container, row, scrollable, text, Column, Container},
    Application, Command, Element, Length, Settings, Theme,
};
use std::path::PathBuf;
use tracing::{info, debug};
use image::DynamicImage;
use anyhow::Result;

mod photo;
mod ui;
mod processors;

use photo::Photo;
use ui::PhotoView;

pub fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::fmt::init();
    info!("Starting PhotoFlow...");

    // Start the application
    PhotoFlow::run(Settings::default())
}

#[derive(Debug)]
struct PhotoFlow {
    photos: Vec<Photo>,
    current_photo: Option<usize>,
    photo_view: PhotoView,
    error: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    LoadDirectory,
    DirectoryLoaded(Vec<PathBuf>),
    PhotoSelected(usize),
    NextPhoto,
    PreviousPhoto,
    Error(String),
    ImageLoaded(PathBuf, Option<DynamicImage>),
}

impl Application for PhotoFlow {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                photos: Vec::new(),
                current_photo: None,
                photo_view: PhotoView::new(),
                error: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("PhotoFlow")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::LoadDirectory => {
                debug!("Loading directory...");
                Command::perform(
                    async {

                    if let Some(folder) = rfd::AsyncFileDialog::new()
                        .set_title("Select Photo Directory")
                        .pick_folder()
                        .await
                    {
                        let mut paths = Vec::new();
                        if let Ok(entries) = std::fs::read_dir(folder.path()) {
                            for entry in entries {
                                if let Ok(entry) = entry {
                                    let path = entry.path();
                                    if let Some(ext) = path.extension() {
                                        let ext = ext.to_string_lossy().to_lowercase();
                                        if ext == "jpg" || ext == "jpeg" || ext == "raf" || ext == "raw" {
                                            paths.push(path);
                                        }
                                    }
                                }
                            }
                        }
                        Message::DirectoryLoaded(paths)
                    } else {
                        Message::Error("No directory selected".to_string())
                    }
                    },
                    Message::from,
                )
            }
            Message::DirectoryLoaded(paths) => {
                debug!("Directory loaded with {} paths", paths.len());
                self.error = None;
                let photos: Vec<_> = paths
                    .iter()
                    .filter_map(|path| {
                        debug!("Attempting to load photo: {}", path.display());
                        Photo::new(path.clone()).ok()
                    })
                    .collect();
                
                if !photos.is_empty() {
                    self.photos = photos;
                    self.current_photo = Some(0);
                    
                    // Load the first image
                    if let Some(photo) = self.photos.first() {
                        let path = photo.path().to_path_buf();
                        let path_clone = path.clone();
                        let photo_clone = photo.clone();
                        return Command::perform(
                            async move { photo_clone.load_image() },
                            move |result| Message::ImageLoaded(path_clone, result.ok())
                        );
                    }
                } else if !paths.is_empty() {
                    self.error = Some("No valid photos found in directory".to_string());
                }
                
                Command::none()
            }
            Message::PhotoSelected(index) => {
                if index < self.photos.len() {
                    self.current_photo = Some(index);
                    let photo = &self.photos[index];
                    let path = photo.path().to_path_buf();
                    let path_clone = path.clone();
                    let photo_clone = photo.clone();
                    return Command::perform(
                        async move { photo_clone.load_image() },
                        move |result| Message::ImageLoaded(path_clone, result.ok())
                    );
                }
                Command::none()
            }
            Message::NextPhoto => {
                if let Some(current) = self.current_photo {
                    if current + 1 < self.photos.len() {
                        self.current_photo = Some(current + 1);
                        let photo = &self.photos[current + 1];
                        let path = photo.path().to_path_buf();
                        let path_clone = path.clone();
                        let photo_clone = photo.clone();
                        return Command::perform(
                            async move { photo_clone.load_image() },
                            move |result| Message::ImageLoaded(path_clone, result.ok())
                        );
                    }
                }
                Command::none()
            }
            Message::PreviousPhoto => {
                if let Some(current) = self.current_photo {
                    if current > 0 {
                        self.current_photo = Some(current - 1);
                        let photo = &self.photos[current - 1];
                        let path = photo.path().to_path_buf();
                        let path_clone = path.clone();
                        let photo_clone = photo.clone();
                        return Command::perform(
                            async move { photo_clone.load_image() },
                            move |result| Message::ImageLoaded(path_clone, result.ok())
                        );
                    }
                }
                Command::none()
            }
            Message::Error(error) => {
                info!("Error: {}", error);
                self.error = Some(error);
                Command::none()
            }
            Message::ImageLoaded(path, image) => {
                debug!("Image loaded: {}", path.display());
                if let Some(image) = image {
                    // Find the photo with matching path and update it
                    if let Some(photo) = self.photos.iter_mut().find(|p| p.path() == path) {
                        photo.set_image(image);
                    }
                } else {
                    info!("Failed to load image: {}", path.display());
                    self.error = Some(format!("Failed to load image: {}", path.display()));
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let controls = row![
            button("Previous").on_press(Message::PreviousPhoto),
            button("Load Directory").on_press(Message::LoadDirectory),
            button("Next").on_press(Message::NextPhoto),
        ]
        .spacing(10);

        let content = if let Some(current) = self.current_photo {
            if let Some(photo) = self.photos.get(current) {
                self.photo_view.view(photo)
            } else {
                text("No photo selected").into()
            }
        } else {
            text("No photos loaded").into()
        };

        let error_text = if let Some(error) = &self.error {
            Element::from(
                container(
                    text(error)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.8, 0.0, 0.0)))
                )
                .padding(10)
            )
        } else {
            Element::from(container(text("")).padding(10))
        };

        let layout = column![controls, error_text, content].spacing(20).padding(20);

        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
