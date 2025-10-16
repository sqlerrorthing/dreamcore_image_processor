use ab_glyph::{Font, Glyph, Point, ScaleFont, point};
use image::imageops::{FilterType, resize};
use image::{DynamicImage, GenericImageView};
use rand::{Rng, rng};

pub mod assets;
pub mod transformation;

/// Took from https://github.com/alexheretic/ab-glyph/blob/main/dev/src/layout.rs
pub fn layout_paragraph<F, SF>(
    font: SF,
    position: Point,
    max_width: f32,
    text: &str,
    target: &mut Vec<Glyph>,
) where
    F: Font,
    SF: ScaleFont<F>,
{
    let v_advance = font.height() + font.line_gap();
    let mut caret = position + point(0.0, font.ascent());
    let mut last_glyph: Option<Glyph> = None;
    for c in text.chars() {
        if c.is_control() {
            if c == '\n' {
                caret = point(position.x, caret.y + v_advance);
                last_glyph = None;
            }
            continue;
        }
        let mut glyph = font.scaled_glyph(c);
        if let Some(previous) = last_glyph.take() {
            caret.x += font.kern(previous.id, glyph.id);
        }
        glyph.position = caret;

        last_glyph = Some(glyph.clone());
        caret.x += font.h_advance(glyph.id);

        if !c.is_whitespace() && caret.x > position.x + max_width {
            caret = point(position.x, caret.y + v_advance);
            glyph.position = caret;
            last_glyph = None;
        }

        target.push(glyph);
    }
}

pub fn crop_and_resize(img: &mut DynamicImage, final_size: u32) {
    let (width, height) = img.dimensions();
    let crop_size = width.min(height);

    let mut rng = rng();
    let left = if width > crop_size {
        rng.random_range(0..=width.saturating_sub(crop_size))
    } else {
        0
    };

    let top = if height > crop_size {
        rng.random_range(0..=height.saturating_sub(crop_size))
    } else {
        0
    };

    let right = left + crop_size;
    let bottom = top + crop_size;

    *img = img.crop(left, top, right - left, bottom - top);

    let resized = image::imageops::resize(img, final_size, final_size, FilterType::Lanczos3);
    *img = DynamicImage::ImageRgba8(resized);
}

fn resize_to_background_image_scale(
    img: &DynamicImage,
    background: &DynamicImage,
    scale: f32,
) -> DynamicImage {
    let target_width = (background.width() as f32 * scale) as u32;
    let aspect_ratio = img.height() as f32 / img.width() as f32;
    let target_height = (target_width as f32 * aspect_ratio) as u32;

    DynamicImage::ImageRgba8(resize(
        img,
        target_width,
        target_height,
        FilterType::Lanczos3,
    ))
}
