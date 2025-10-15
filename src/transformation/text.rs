use crate::transformation::ImageTransformation;
use crate::{assets, layout_paragraph};
use ab_glyph::{Font, FontRef, PxScale, ScaleFont, point};
use image::{DynamicImage, Rgba};
use imageproc::drawing::draw_text_mut;
use rand::seq::IndexedRandom;
use rand::{Rng, rng};

pub struct AddTextTransformer<'a> {
    fonts: Vec<FontRef<'a>>,
    texts: Vec<&'static str>,
}

impl<'a> Default for AddTextTransformer<'a> {
    fn default() -> Self {
        let mut fonts = Vec::new();

        for font in assets::FONTS.entries() {
            let font = font.as_file().expect(".ttf font file");
            fonts.push(FontRef::try_from_slice(font.contents()).expect("valid ttf font"));
        }

        assert!(
            !fonts.is_empty(),
            "fonts cannot be empty, at least one font is required"
        );

        Self {
            fonts,
            texts: vec![
                "Why do you keep coming back?",
                "The walls remember you.",
                "I dreamt of you last night.",
                "Why do you always return?",
                "Exit?",
                "WAKE UP",
                "This is a Dream",
                "come with me, dear",
                "i want to go back",
                "he is watching!",
                "It's time to go home",
                "It's funny!"
            ],
        }
    }
}

enum RepeatedDirection {
    Top,
    TopLeft,
    TopRight,
    Bottom,
    BottomLeft,
    BottomRight
}

enum PlacementStyle {
    Single,
    Repeated {
        times: u32,
        direction: RepeatedDirection,
    }
}

fn get_random_placement_style() -> PlacementStyle {
    let mut rng = rng();

    if rng.random_bool(0.8) {
        PlacementStyle::Single
    } else {
        let times = rng.random_range(2..=4);

        let direction = match rng.random_range(0..=5) {
            0 => RepeatedDirection::Top,
            1 => RepeatedDirection::TopLeft,
            2 => RepeatedDirection::TopRight,
            3 => RepeatedDirection::Bottom,
            4 => RepeatedDirection::BottomLeft,
            _ => RepeatedDirection::BottomRight,
        };

        PlacementStyle::Repeated { times, direction }
    }
}

fn random_text_params(font: &FontRef, text: &str, image: &DynamicImage) -> (PxScale, f32, f32, i32, i32, Rgba<u8>) {
    let mut rng = rng();
    let scale = PxScale::from(rng.random_range(28.0..34.0));
    let scaled_font = font.into_scaled(scale);

    let mut glyphs = Vec::new();
    layout_paragraph(
        scaled_font,
        point(0.0, 0.0),
        image.width() as _,
        text,
        &mut glyphs,
    );

    let width = glyphs
        .iter()
        .last()
        .map(|g| g.position.x + scaled_font.h_advance(g.id))
        .unwrap_or(0.0)
        .ceil();

    let height = scale.y;
    let max_x = (image.width() as f32 - width).max(0.0);
    let max_y = (image.height() as f32 - height).max(0.0);

    let x = rng.random_range(0.0..max_x) as _;
    let y = rng.random_range(0.0..max_y) as _;

    let color = Rgba([rng.random_range(200..255), rng.random_range(0..30), 0, rng.random_range(200..255)]);

    (scale, width, height, x, y, color)
}

fn draw_random_text(font: &FontRef, text: &str, image: &mut DynamicImage) {
    let (scale, _, _, x, y, color) = random_text_params(font, text, image);
    draw_text_mut(image, color, x, y, scale, font, text);
}

fn apply_repeated_text(
    font: &FontRef,
    text: &str,
    image: &mut DynamicImage,
    times: u32,
    direction: &RepeatedDirection,
) {
    let (scale, _, height, mut x, mut y, color) = random_text_params(font, text, image);

    for _ in 0..times {
        draw_text_mut(image, color, x, y, scale, font, text);

        let step = 10;
        match direction {
            RepeatedDirection::Top => y -= height as i32,
            RepeatedDirection::Bottom => y += height as i32,
            RepeatedDirection::TopLeft => { y -= height as i32; x -= step; }
            RepeatedDirection::TopRight => { y -= height as i32; x += step; }
            RepeatedDirection::BottomLeft => { y += height as i32; x -= step; }
            RepeatedDirection::BottomRight => { y += height as i32; x += step; }
        }
    }
}

impl ImageTransformation for AddTextTransformer<'_> {
    fn transform(&self, image: &mut DynamicImage) {
        let mut rng = rng();
        for _ in 0..rng.random_range(1..3) {
            let font = unsafe { self.fonts.choose(&mut rng).unwrap_unchecked() };
            let text = unsafe { self.texts.choose(&mut rng).unwrap_unchecked() };

            match get_random_placement_style() {
                PlacementStyle::Single => {
                    draw_random_text(font, text, image);
                }
                PlacementStyle::Repeated { times, direction } => {
                    apply_repeated_text(font, text, image, times, &direction);
                }
            }
        }
    }
}
