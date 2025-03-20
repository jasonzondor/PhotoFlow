use iced::{
    executor,
    widget::{button, column, container, row, text},
    Application, Command, Element, Length, Settings, Theme,
};
use std::path::PathBuf;
use tracing::{info, debug};
use image::DynamicImage;

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
    photo_paths: Vec<PathBuf>,
    photos: Vec<Option<Photo>>,
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
                photo_paths: Vec::new(),
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
                
                if !paths.is_empty() {
                    // Store paths and initialize photos vector with None
                    let paths_len = paths.len();
                    let first_path = paths[0].clone();
                    self.photo_paths = paths;
                    self.photos = vec![None; paths_len];
                    self.current_photo = Some(0);
                    
                    // Load only the first photo
                    let first_path_clone = first_path.clone();
                    return Command::perform(
                        async move {
                            match Photo::new(first_path.clone()) {
                                Ok(mut photo) => {
                                    if let Ok(image) = photo.load_image() {
                                        photo.set_image(image);
                                        Some(photo)
                                    } else {
                                        None
                                    }
                                },
                                Err(_) => None
                            }
                        },
                        move |result| {
                            if let Some(photo) = result {
                                Message::ImageLoaded(first_path_clone, photo.image)
                            } else {
                                Message::Error(format!("Failed to load image: {}", first_path_clone.display()))
                            }
                        }
                    );
                } else {
                    self.error = Some("No photos found in directory".to_string());
                }
                
                Command::none()
            }
            Message::PhotoSelected(index) => {
                if index < self.photos.len() {
                    self.current_photo = Some(index);
                    
                    // If photo isn't loaded yet, load it
                    if self.photos[index].is_none() {
                        let path = self.photo_paths[index].clone();
                        let path_clone = path.clone();
                        return Command::perform(
                            async move {
                                match Photo::new(path.clone()) {
                                    Ok(mut photo) => {
                                        if let Ok(image) = photo.load_image() {
                                            photo.set_image(image.clone());
                                            Some((photo, image))
                                        } else {
                                            None
                                        }
                                    },
                                    Err(_) => None
                                }
                            },
                            move |result| {
                                if let Some((photo, image)) = result {
                                    Message::ImageLoaded(path_clone, Some(image))
                                } else {
                                    Message::Error(format!("Failed to load image: {}", path_clone.display()))
                                }
                            }
                        );
                    }
                }
                Command::none()
            }
            Message::NextPhoto => {
                if let Some(current) = self.current_photo {
                    if current + 1 < self.photos.len() {
                        let next = current + 1;
                        if self.photos[next].is_none() {
                            let path = self.photo_paths[next].clone();
                            let path_clone = path.clone();
                            return Command::perform(
                                async move {
                                    match Photo::new(path.clone()) {
                                        Ok(mut photo) => {
                                            if let Ok(image) = photo.load_image() {
                                                photo.set_image(image);
                                                Some(photo)
                                            } else {
                                                None
                                            }
                                        },
                                        Err(_) => None
                                    }
                                },
                                move |result| {
                                    if let Some(photo) = result {
                                        Message::ImageLoaded(path_clone, photo.image)
                                    } else {
                                        Message::Error(format!("Failed to load image: {}", path_clone.display()))
                                    }
                                }
                            );
                        }
                        self.current_photo = Some(next);
                    }
                }
                Command::none()
            }
            Message::PreviousPhoto => {
                if let Some(current) = self.current_photo {
                    if current > 0 {
                        self.current_photo = Some(current - 1);
                        if self.photos[current - 1].is_none() {
                            let path = self.photo_paths[current - 1].clone();
                            let path_clone = path.clone();
                            return Command::perform(
                                async move {
                                    match Photo::new(path.clone()) {
                                        Ok(mut photo) => {
                                            if let Ok(image) = photo.load_image() {
                                                photo.set_image(image);
                                                Some(photo)
                                            } else {
                                                None
                                            }
                                        },
                                        Err(_) => None
                                    }
                                },
                                move |result| {
                                    if let Some(photo) = result {
                                        Message::ImageLoaded(path_clone, photo.image)
                                    } else {
                                        Message::Error(format!("Failed to load image: {}", path_clone.display()))
                                    }
                                }
                            );
                        }
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
                if let Some(index) = self.photo_paths.iter().position(|p| p == &path) {
                    // Create new photo if it doesn't exist
                    if self.photos[index].is_none() {
                        if let Ok(mut photo) = Photo::new(path.clone()) {
                            if let Some(img) = image {
                                photo.set_image(img);
                            }
                            self.photos[index] = Some(photo);
                        }
                    } else if let Some(photo) = &mut self.photos[index] {
                        if let Some(img) = image {
                            photo.set_image(img);
                        }
                    }
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let current_photo = self.current_photo
            .and_then(|i| self.photos[i].as_ref());
        
        let controls = row![
            button("Previous").on_press(Message::PreviousPhoto),
            button("Load Directory").on_press(Message::LoadDirectory),
            button("Next").on_press(Message::NextPhoto),
        ]
        .spacing(10);

        let content = if let Some(photo) = current_photo {
            self.photo_view.view(photo)
        } else {
            text("No photo selected").into()
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
