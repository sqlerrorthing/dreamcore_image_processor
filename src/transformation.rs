pub mod distortion;
pub mod eyes;
pub mod text;

use image::DynamicImage;
use std::ops::Add;

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
pub trait ImageTransformation: Send + Sync {
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

impl ImageTransformation for Pipeline {
    fn transform(&self, image: &mut DynamicImage) {
        for step in &self.steps {
            step.transform(image)
        }
    }
}
