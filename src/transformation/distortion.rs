use crate::transformation::ImageTransformation;
use derive_new::new;
use image::{DynamicImage, GenericImage, GenericImageView, Rgba};
use rand::{Rng, rng};

#[derive(Debug, new)]
pub struct Distortion {
    intensity: f32,
}

impl ImageTransformation for Distortion {
    fn transform(&self, image: &mut DynamicImage) {
        let (width, height) = image.dimensions();
        let mut rng = rng();

        for x in 0..width {
            for y in 0..height {
                let pixel = image.get_pixel(x, y);
                let channels = pixel.0;

                let new_pixel = Rgba([
                    (channels[0] as f32 + rng.random_range(-15.0..15.0) * self.intensity)
                        .clamp(0.0, 255.0) as u8,
                    (channels[1] as f32 + rng.random_range(-15.0..15.0) * self.intensity)
                        .clamp(0.0, 255.0) as u8,
                    (channels[2] as f32 + rng.random_range(-15.0..15.0) * self.intensity)
                        .clamp(0.0, 255.0) as u8,
                    channels[3],
                ]);

                image.put_pixel(x, y, new_pixel);
            }
        }
    }
}
