use std::fmt::{Display, Formatter};
use crate::transformation::ImageTransformation;
use derive_new::new;
use image::{DynamicImage, GenericImage, GenericImageView, Rgba};
use log::info;
use rand::{Rng, rng};
use rand::distr::uniform::SampleRange;

#[derive(Debug, new)]
pub struct Distortion<R> {
    intensity: R,
}

impl<R: SampleRange<f32>> Display for Distortion<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Distortion")
    }
}

impl<R: SampleRange<f32> + Send + Sync + Clone> ImageTransformation for Distortion<R> {
    fn transform(&self, image: &mut DynamicImage) {
        let (width, height) = image.dimensions();
        let mut rng = rng();
        let intensity = rng.random_range(self.intensity.clone());

        info!("Applying distortion for image {image:p} with {intensity:.2} intensity");

        for x in 0..width {
            for y in 0..height {
                let pixel = image.get_pixel(x, y);
                let channels = pixel.0;

                let new_pixel = Rgba([
                    (channels[0] as f32 + rng.random_range(-15.0..15.0) * intensity)
                        .clamp(0.0, 255.0) as u8,
                    (channels[1] as f32 + rng.random_range(-15.0..15.0) * intensity)
                        .clamp(0.0, 255.0) as u8,
                    (channels[2] as f32 + rng.random_range(-15.0..15.0) * intensity)
                        .clamp(0.0, 255.0) as u8,
                    channels[3],
                ]);

                image.put_pixel(x, y, new_pixel);
            }
        }
    }
}
