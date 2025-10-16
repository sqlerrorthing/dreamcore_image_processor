use crate::assets;
use crate::transformation::ImageTransformation;
use image::imageops::{FilterType, overlay, resize};
use image::{DynamicImage, Rgba};
use imageproc::definitions::Image;
use imageproc::geometric_transformations::{Interpolation, rotate_about_center};
use include_dir::Dir;
use rand::seq::IndexedRandom;
use rand::{Rng, rng};
use std::ops::RangeInclusive;

pub enum Eyeball {
    SimpleEye,
    EyeWithWings,
}

pub struct Eyeballs {
    r#type: Eyeball,
    count: RangeInclusive<u32>,
    balls: Vec<DynamicImage>,
    wings: Option<Vec<DynamicImage>>,
}

#[inline(always)]
fn scale_and_rotate(image: &DynamicImage, scale: f32, angle_deg: Option<f32>) -> Image<Rgba<u8>> {
    let new_width = (image.width() as f32 * scale) as u32;
    let new_height = (image.height() as f32 * scale) as u32;
    let resized_ball = resize(image, new_width, new_height, FilterType::Lanczos3);

    if let Some(angle_deg) = angle_deg
        && angle_deg != 0.0
    {
        rotate_about_center(
            &resized_ball,
            angle_deg.to_radians(),
            Interpolation::Bilinear,
            Rgba([0, 0, 0, 0]),
        )
    } else {
        resized_ball
    }
}

fn place_simple_ball(ball: &DynamicImage, image: &mut DynamicImage) {
    let mut rng = rng();

    let rotated_ball = scale_and_rotate(
        ball,
        rng.random_range(0.9..=1.2),
        Some(rng.random_range(-40.0..=40.0)),
    );

    let max_x = image.width().saturating_sub(rotated_ball.width());
    let max_y = image.height().saturating_sub(rotated_ball.height());
    let x = rng.random_range(0..=max_x);
    let y = rng.random_range(0..=max_y);

    overlay(image, &rotated_ball, x as _, y as _);
}

fn place_ball_with_wing(wing: &DynamicImage, ball: &DynamicImage, image: &mut DynamicImage) {
    let mut rng = rng();

    let scaled_wing = scale_and_rotate(wing, rng.random_range(0.9..=1.2), None);

    let max_x = image.width().saturating_sub(scaled_wing.width());
    let max_y = image.height().saturating_sub(scaled_wing.height());
    let wing_x = rng.random_range(0..=max_x);
    let wing_y = rng.random_range(0..=max_y);

    overlay(image, &scaled_wing, wing_x as _, wing_y as _);

    let rotated_ball = scale_and_rotate(
        ball,
        rng.random_range(0.5..=0.8),
        Some(rng.random_range(-10.0..=10.0)),
    );

    let center_x = wing_x + (scaled_wing.width() / 2).saturating_sub(rotated_ball.width() / 2);
    let center_y = wing_y + (scaled_wing.height() / 2).saturating_sub(rotated_ball.height() / 2);

    overlay(image, &rotated_ball, center_x as _, center_y as _);
}

impl ImageTransformation for Eyeballs {
    fn transform(&self, image: &mut DynamicImage) {
        let mut rng = rng();

        if self.count.is_empty() {
            return;
        }

        for _ in 0..rng.random_range(self.count.clone()) {
            let ball = self.balls.choose(&mut rng).unwrap();
            let ball = crate::resize_to_background_image_scale(ball, image, 0.2);

            match self.r#type {
                Eyeball::SimpleEye => place_simple_ball(&ball, image),
                Eyeball::EyeWithWings => {
                    let wing = self.wings.as_ref().unwrap().choose(&mut rng).unwrap();
                    let wing = crate::resize_to_background_image_scale(wing, image, 0.3);
                    place_ball_with_wing(&wing, &ball, image)
                }
            }
        }
    }
}

impl Eyeballs {
    pub fn new(r#type: Eyeball, count: RangeInclusive<u32>) -> Self {
        let balls: Vec<DynamicImage> = load_images(assets::EYEBALLS);

        let mut wings: Option<Vec<DynamicImage>> = None;

        assert!(!balls.is_empty(), "at least one eyeball is required");

        if matches!(r#type, Eyeball::EyeWithWings) {
            wings = Some(load_images(assets::WINGS));
            assert!(
                unsafe { !&wings.as_ref().unwrap_unchecked().is_empty() },
                "at least one wing image is required when using EyeBall::EyeWithWings"
            );
        }

        Self {
            r#type,
            count,
            balls,
            wings,
        }
    }
}

#[inline(always)]
fn load_images(dir: &Dir) -> Vec<DynamicImage> {
    dir.entries()
        .iter()
        .map(|file| file.as_file().expect("valid image files"))
        .map(|ball| image::load_from_memory(ball.contents()).expect("valid image"))
        .collect()
}
