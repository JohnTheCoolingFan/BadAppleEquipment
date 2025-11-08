use std::{collections::HashMap, fmt::Display, fs::File, io::Write, path::PathBuf, time::Instant};

use image::{GenericImage, ImageBuffer, Rgb};
use indicatif::ProgressBar;
use rayon::prelude::*;

mod quadtree;
use quadtree::*;

const BLACK: Rgb<u8> = Rgb([0, 0, 0]);
const WHITE: Rgb<u8> = Rgb([253, 255, 255]);

const IMAGE_AMOUNT: u64 = 6575;

const DOWNSCALE: u8 = 1;
const WIDTH_ORIGINAL: u32 = 480;
const HEIGHT_ORIGINAL: u32 = 360;
const WIDTH: u32 = WIDTH_ORIGINAL / DOWNSCALE as u32;
const HEIGHT: u32 = HEIGHT_ORIGINAL / DOWNSCALE as u32;

const SAMPLE_SIZE: u32 = 8;
const CHUNKS_X: u32 = WIDTH / SAMPLE_SIZE;
const CHUNKS_Y: u32 = HEIGHT / SAMPLE_SIZE;

const CANVAS_SIZE: u32 = 512 / (DOWNSCALE as u32);

type TileData = [u8; ((SAMPLE_SIZE * SAMPLE_SIZE) as usize).div_ceil(8)];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TileId(TileData);

impl TileId {
    fn from_samples(mut samples: [bool; (SAMPLE_SIZE * SAMPLE_SIZE) as usize]) -> Self {
        let mut res = TileData::default();
        samples.reverse();
        let (full, remainder) = samples.as_chunks::<8>();
        for (i, chunk) in full.iter().enumerate() {
            let val = chunk
                .iter()
                .map(|v| if *v { 1 } else { 0 })
                .fold(0, |acc, v| (acc << 1) | v);
            res[i] = val;
        }
        if !remainder.is_empty() {
            let mut remainder_buf = [false; 8];
            remainder_buf[0..(remainder.len())].copy_from_slice(remainder);
            let val = remainder_buf
                .iter()
                .map(|v| if *v { 1 } else { 0 })
                .fold(0, |acc, v| (acc << 1) | v);
            res[res.len() - 1] = val;
        }

        Self(res)
    }
}

impl Display for TileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

fn encode_pixels_hex(values: [bool; (SAMPLE_SIZE * SAMPLE_SIZE) as usize]) -> String {
    TileId::from_samples(values).to_string()
}

fn main() {
    let start = Instant::now();

    let out_dir = PathBuf::from("output");
    if !out_dir.exists() {
        std::fs::create_dir(&out_dir).unwrap();
    }

    let progress_bar = ProgressBar::new(IMAGE_AMOUNT).with_message("Processing images");

    let mut chunk_counts: HashMap<TileId, u32> = HashMap::new();

    let mut tree_out_file = File::create(out_dir.join("frames-tree.lua")).unwrap();
    tree_out_file.write_all(b"return {").unwrap();

    let mut filled_squares_file = File::create(out_dir.join("filled-squares.lua")).unwrap();
    filled_squares_file.write_all(b"return {").unwrap();

    let mut working_buffer = ImageBuffer::new(CANVAS_SIZE, CANVAS_SIZE);

    let processing_start = Instant::now();
    progress_bar.println(format!(
        "Started processing at {}ms",
        (processing_start - start).as_millis()
    ));

    for inum in 0..IMAGE_AMOUNT {
        let inum = inum + 1;
        let image = image::open(format!("images/{DOWNSCALE}x/frame{inum:04}.png")).unwrap();
        let image = image.into_rgb8();
        working_buffer.copy_from(&image, 0, 0).unwrap();

        let quad_tree = QuadTree::build(&working_buffer);

        for tile_id in quad_tree.root.get_shapes() {
            *chunk_counts.entry(*tile_id).or_insert(0) += 1;
        }

        let quad_tree_lua = quad_tree.root.as_lua();

        tree_out_file.write_all(quad_tree_lua.as_bytes()).unwrap();
        tree_out_file.write_all(b",").unwrap();

        filled_squares_file.write_all(b"{").unwrap();
        for (x, y, size) in quad_tree.all_filled_squares() {
            filled_squares_file
                .write_all(format!("{{{x},{y},{size}}},").as_bytes())
                .unwrap();
        }
        filled_squares_file.write_all(b"},").unwrap();

        progress_bar.inc(1);
    }

    tree_out_file.write_all(b"}").unwrap();
    filled_squares_file.write_all(b"}").unwrap();

    progress_bar.finish_with_message("All done");

    let processing_end = Instant::now();
    println!(
        "Processing ended at {}ms, took {}ms",
        (processing_end - processing_start).as_millis(),
        (processing_end - start).as_millis()
    );

    println!("Number of unique chunks: {}", chunk_counts.len());

    let used_numbers_start = Instant::now();

    let mut used_numbers_file = File::create(out_dir.join("used-tiles.lua")).unwrap();
    used_numbers_file.write_all(b"return {").unwrap();

    for tile_id in chunk_counts.keys() {
        used_numbers_file
            .write_all(format!("\"{tile_id}\",").as_ref())
            .unwrap();
    }

    used_numbers_file.write_all(b"}").unwrap();

    let used_numbers_end = Instant::now();

    println!(
        "Used numbers collection took {}ms",
        (used_numbers_end - used_numbers_start).as_millis()
    );

    let mut out_stats = File::create(out_dir.join(format!("tile_stats_{DOWNSCALE}x.csv"))).unwrap();
    writeln!(&mut out_stats, "tile_id,amount").unwrap();

    for (tile_id, amount) in chunk_counts.iter() {
        writeln!(&mut out_stats, "tile_{tile_id},{amount}").unwrap();
    }

    out_stats.flush().unwrap();

    let tile_stats_end = Instant::now();

    println!(
        "Tile stats took {}ms",
        (tile_stats_end - used_numbers_end).as_millis()
    );
}
