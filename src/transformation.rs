pub mod distortion;
pub mod eyes;
pub mod text;

use std::fmt::{Display, Formatter};
use image::DynamicImage;
use std::ops::Add;
use log::info;

/// # Requirements
///
/// - The input image **must be square**, meaning:
///   [`DynamicImage::width()`] == [`DynamicImage::height()`].
/// - Implementations should operate directly on the provided [`DynamicImage`]
///   without changing its dimensions.
///
/// # Parameters
///
/// - `image`: A mutable reference to a [`DynamicImage`] that will be transformed.
pub trait ImageTransformation: Send + Sync + Display {
    fn transform(&self, image: &mut DynamicImage);
}

#[derive(Default)]
pub struct Pipeline {
    steps: Vec<Box<dyn ImageTransformation>>,
}

impl Add<Box<dyn ImageTransformation>> for Pipeline {
    type Output = Self;

    fn add(mut self, rhs: Box<dyn ImageTransformation>) -> Self::Output {
        self.steps.push(rhs);
        self
    }
}

impl<T: ImageTransformation + 'static> Add<T> for Pipeline {
    type Output = Self;

    fn add(mut self, rhs: T) -> Self::Output {
        self.steps.push(Box::new(rhs));
        self
    }
}

impl Display for Pipeline {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pipeline")
    }
}

impl ImageTransformation for Pipeline {
    fn transform(&self, image: &mut DynamicImage) {
        let chain = self.steps
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(" -> ");

        info!("Running transformations for image {image:p}: {chain}",);

        for step in &self.steps {
            step.transform(image)
        }
    }
}
