pub mod eyes;
pub mod text;
pub mod compression_artifact;

use std::ops::Add;
use image::DynamicImage;

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

impl Pipeline {
    pub fn add_ref(mut self, step: Box<dyn ImageTransformation>) -> Self {
        self.steps.push(step);
        self
    }
}

impl<T: ImageTransformation + 'static> Add<T> for Pipeline {
    type Output = Pipeline;

    fn add(self, rhs: T) -> Self::Output {
        self.add_ref(Box::new(rhs))
    }
}

impl ImageTransformation for Pipeline {
    fn transform(&self, image: &mut DynamicImage) {
        for step in &self.steps {
            step.transform(image)
        }
    }
}
