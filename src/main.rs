use dreamcore_image_processor::crop_and_resize;
use dreamcore_image_processor::transformation::distortion::Distortion;
use dreamcore_image_processor::transformation::eyes::{Eyeball, Eyeballs};
use dreamcore_image_processor::transformation::text::DreamcoreStyledTextTransform;
use dreamcore_image_processor::transformation::{ImageTransformation, Pipeline};
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let original = image::open("image.jpg")?;

    let pipeline = Pipeline::default()
        + DreamcoreStyledTextTransform::default()
        + Distortion::new(2.0)
        + Eyeballs::new(Eyeball::SimpleEye, 1..=3)
        + Eyeballs::new(Eyeball::EyeWithWings, 0..=2);

    let now = Instant::now();
    (0..4).into_par_iter().for_each(|i| {
        let mut img = original.clone();
        crop_and_resize(&mut img, 512);

        pipeline.transform(&mut img);

        let path = format!("output/image-{i:02}.png");
        println!("saving {path}");
        img.save(path).unwrap();
    });

    let end = now.elapsed();

    println!(
        "All transformations took {:0.2} seconds to complete",
        end.as_secs_f32()
    );

    Ok(())
}
