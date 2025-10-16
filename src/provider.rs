use image::DynamicImage;
use thiserror::Error;

pub mod pinterest;

#[derive(Debug, Error)]
pub enum FetchBackgroundError {
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("No more images")]
    NoImages,

    #[error("Invalid image: {0}")]
    InvalidImage(#[from] image::ImageError),
}

pub trait BackgroundProvider {
    fn fetch_background(&self) -> impl Future<Output = Result<DynamicImage, FetchBackgroundError>>;
}
