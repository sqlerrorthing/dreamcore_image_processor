use dreamcore_image_processor::crop_and_resize;
use dreamcore_image_processor::provider::BackgroundProvider;
use dreamcore_image_processor::provider::pinterest::PinterestProvider;
use dreamcore_image_processor::transformation::distortion::Distortion;
use dreamcore_image_processor::transformation::eyes::{Eyeball, Eyeballs};
use dreamcore_image_processor::transformation::text::DreamcoreStyledTextTransform;
use dreamcore_image_processor::transformation::{ImageTransformation, Pipeline};
use futures::future::join_all;
use image::GenericImageView;
use log::{error, info};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::task::spawn_blocking;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let provider = Arc::new(PinterestProvider::new("dreamcore landscape"));

    let pipeline = Pipeline::default()
        + DreamcoreStyledTextTransform::default()
        + Distortion::new(2.0)
        + Eyeballs::new(Eyeball::SimpleEye, 1..=3)
        + Eyeballs::new(Eyeball::EyeWithWings, 0..=2);

    let pipeline = Arc::new(pipeline);

    let now = Instant::now();

    let tasks = (0..1024).map(|i| {
        let pipeline = pipeline.clone();
        let provider = provider.clone();

        async move {
            let mut img = {
                loop {
                    match provider.fetch_background().await {
                        Ok(img) => break img,
                        Err(err) => {
                            error!("Failed to fetch background image: {err}, sleeping for 1 second");
                            sleep(Duration::from_secs(1)).await;
                        }
                    }
                }
            };

            let (w, h) = img.dimensions();
            info!("Resizing image {i} from {w}x{h} to 512x512");

            crop_and_resize(&mut img, 512);

            info!("Transforming image {i}");

            let img = spawn_blocking(move || {
                pipeline.transform(&mut img);
                img
            })
            .await
            .unwrap();

            let path = format!("output/image-{i:02}.png");
            info!("Saving {path}");
            img.save(path).unwrap();
        }
    });

    join_all(tasks).await;

    let end = now.elapsed();

    println!(
        "All transformations took {:0.2} seconds to complete",
        end.as_secs_f32()
    );

    Ok(())
}
