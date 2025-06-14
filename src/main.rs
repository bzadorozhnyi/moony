use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use clap::Parser;
use image::RgbImage;
use kmeans_colors::{get_kmeans, Kmeans};
use moony::error::AppError;
use palette::{cast::from_component_slice, IntoColor, Lab, Srgb};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Parser)]
#[command(version, about)]
/// Make image look cartoonish using k-means color clustering
struct Cli {
    /// Input img
    #[arg(short, long)]
    input: PathBuf,
    /// Ouput img
    #[arg(short, long)]
    output: PathBuf,
    /// Number of runs for k-means
    #[arg(short, long, default_value_t = 10)]
    runs: usize,
    /// Number of cluster for k-means
    #[arg(short, long, default_value_t = 10)]
    clusters: usize,
    #[arg(short, long, default_value_t = 0)]
    // Seed for k-means
    seed: u64,
    #[arg(short, long, default_value_t = 10)]
    // Max number of iterations for k-means
    max_iters: usize,
    #[arg(short, long, default_value_t = 255.0)]
    // Converge for k-means
    converge: f32,
    // Max number of threads
    #[arg(short, long, default_value_t = 1)]
    max_threads: usize,
}

fn main() -> Result<(), AppError> {
    let cli = Cli::parse();

    // set max threads
    rayon::ThreadPoolBuilder::new()
        .num_threads(cli.max_threads)
        .build_global()
        .unwrap();

    // read the image
    let img = image::open(cli.input)
        .map_err(|_| AppError::FailedOpenImg)?
        .to_rgb8();

    // flatten the image pixels into a flat RGB vector: [R, G, B, R, G, B, ...]
    let (width, height) = img.dimensions();
    let pixels = img
        .pixels()
        .map(|p| [p[0], p[1], p[2]])
        .flatten()
        .collect::<Vec<_>>();

    // convert the RGB pixels to Lab color space for better perceptual clustering
    let lab_pixels: Vec<Lab> = from_component_slice::<Srgb<u8>>(&pixels)
        .iter()
        .map(|x| x.into_linear().into_color())
        .collect::<Vec<_>>();

    // run k-means clustering multiple times in parallel with different seeds
    // and keep the best result based on the lowest score
    let best_kmeans = Arc::new(Mutex::new(Kmeans::<Lab>::new()));
    (0..=cli.runs)
        .into_par_iter()
        .map(|i| {
            let candidate_result = get_kmeans(
                cli.clusters,
                cli.max_iters,
                cli.converge,
                false,
                &lab_pixels,
                cli.seed + i as u64,
            );

            let mut result = best_kmeans.lock().unwrap();

            if candidate_result.score < result.score {
                *result = candidate_result;
            }
        })
        .collect::<Vec<_>>();

    // get the best result (for simplicity)
    let best_kmeans = Arc::try_unwrap(best_kmeans)
        .map_err(|_| AppError::Other("Failed unwrap Arc".to_string()))?
        .into_inner()
        .unwrap();

    // convert Lab cluster centroids back to RGB colors
    let rgb_centroids = &best_kmeans
        .centroids
        .iter()
        .map(|&lab| {
            let srgb = Srgb::<u8>::from_linear(lab.into_color());
            image::Rgb([srgb.red, srgb.green, srgb.blue])
        })
        .collect::<Vec<_>>();

    // cluster assignment for each pixel (i.e. which centroid it belongs to)
    let cluster_assignments = best_kmeans.indices;

    // construct the final image by replacing each pixel with the RGB value of its assigned cluster
    let output_img: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> =
        RgbImage::from_fn(width, height, |x, y| {
            let i = (y * width + x) as usize;
            let pixel_index = cluster_assignments[i] as usize;
            let c = rgb_centroids[pixel_index];

            image::Rgb([c[0] as u8, c[1] as u8, c[2] as u8])
        });

    output_img
        .save(cli.output)
        .map_err(|_| AppError::FailedSaveImg)?;

    Ok(())
}
