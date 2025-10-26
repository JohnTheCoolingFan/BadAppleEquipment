use std::{collections::HashMap, fs::File, io::Write, process::exit, time::Instant};

use image::{GenericImageView, ImageReader, Rgb};
use indicatif::ProgressBar;
use rayon::prelude::*;

const BLACK: Rgb<u8> = Rgb([0, 0, 0]);
const WHITE: Rgb<u8> = Rgb([253, 255, 255]);

const IMAGE_AMOUNT: u64 = 6575;

const DOWNSCALE: u8 = 1;
const WIDTH_ORIGINAL: u32 = 480;
const HEIGHT_ORIGINAL: u32 = 360;
const WIDTH: u32 = WIDTH_ORIGINAL / DOWNSCALE as u32;
const HEIGHT: u32 = HEIGHT_ORIGINAL / DOWNSCALE as u32;

const SAMPLE_SIZE: u32 = 4;
const CHUNKS_X: u32 = WIDTH / SAMPLE_SIZE;
const CHUNKS_Y: u32 = HEIGHT / SAMPLE_SIZE;

fn main() {
    let progress_bar = ProgressBar::new(IMAGE_AMOUNT).with_message("Processing images");
    let mut chunk_counts = [0_u32; 65536];
    let mut out_file = File::create("frameseq.lua").unwrap();
    out_file.write_all(b"return {").unwrap();
    for inum in 0..IMAGE_AMOUNT {
        let inum = inum + 1;
        let image = image::open(format!("images/{DOWNSCALE}x/frame{inum:04}.png")).unwrap();
        let image = image.into_rgb8();

        let mut chunk_seq = "{".to_owned();

        let chunks_masks = (0..(CHUNKS_X * CHUNKS_Y))
            .map(|i| {
                let chunk_y = i / CHUNKS_X;
                let chunk_x = i % CHUNKS_X;

                let view = image.view(
                    chunk_x * SAMPLE_SIZE,
                    chunk_y * SAMPLE_SIZE,
                    SAMPLE_SIZE,
                    SAMPLE_SIZE,
                );
                let mut bools = [false; 16];
                for x in 0..SAMPLE_SIZE {
                    for y in 0..SAMPLE_SIZE {
                        if view.get_pixel(x, y) != BLACK {
                            bools[((x * 4) + y) as usize] = true
                        }
                    }
                }
                let res_mask: u16 = bools
                    .map(|b| if b { 1 } else { 0 })
                    .iter()
                    .fold(0, |acc, val| (acc << 1) + val);
                res_mask
            })
            .collect::<Vec<_>>();

        let mut val = 0;
        let mut length = 0;
        for chunk_mask in chunks_masks {
            chunk_counts[chunk_mask as usize] += 1;
            if chunk_mask == val {
                length += 1
            } else {
                if length != 0 {
                    chunk_seq.push_str(&format!("{{{length}, {val}}},"));
                }
                val = chunk_mask;
                length = 1;
            }
        }

        chunk_seq.push_str("},");

        let chunk_seq = chunk_seq.replace("{,", "{");

        out_file.write_all(chunk_seq.as_ref()).unwrap();

        progress_bar.inc(1);
    }
    out_file.write_all(b"}").unwrap();
    println!("All done");
    println!(
        "Number of unique chunks: {}",
        chunk_counts.iter().filter(|v| **v > 0).count()
    );

    let mut used_numbers_file = File::create("used-numbers.lua").unwrap();
    used_numbers_file.write_all(b"return {").unwrap();
    for (mask, _) in chunk_counts.iter().enumerate().filter(|(_, v)| **v > 0) {
        used_numbers_file
            .write_all(format!("{mask},").as_ref())
            .unwrap();
    }
    used_numbers_file.write_all(b"}").unwrap();
}
