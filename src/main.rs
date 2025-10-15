use std::time::Instant;
use rayon::iter::IntoParallelIterator;
use dreamcore_image_processor::crop_and_resize;
use dreamcore_image_processor::transformation::eyes::{Eyeball, Eyeballs};
use dreamcore_image_processor::transformation::text::AddTextTransformer;
use dreamcore_image_processor::transformation::{ImageTransformation, Pipeline};
use rayon::iter::ParallelIterator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let original = image::open("image.jpg")?;

    let pipeline = Pipeline::default()
        + AddTextTransformer::default()
        + Eyeballs::new(Eyeball::SimpleEye, 1..=3)
        + Eyeballs::new(Eyeball::EyeWithWings, 0..=2);

    let now = Instant::now();
    (0..5).into_par_iter().for_each(|i| {
        let mut img = original.clone();
        crop_and_resize(&mut img, 512);

        pipeline.transform(&mut img);

        let path = format!("output/image-{i:02}.png");
        println!("saving {path}");
        img.save(path).unwrap();
    });

    let end = now.elapsed();

    println!("All transformations took {:0.2} seconds to complete", end.as_secs_f32());

    Ok(())
}
