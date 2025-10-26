use std::{collections::HashMap, fs::File, io::Write, process::exit};

use image::{ImageReader, Rgb};
use indicatif::ProgressBar;
use rayon::prelude::*;

const BLACK: Rgb<u8> = Rgb([0, 0, 0]);
const WHITE: Rgb<u8> = Rgb([253, 255, 255]);

const IMAGE_AMOUNT: u64 = 6575;

const DOWNSCALE: u8 = 3;
const WIDTH_ORIGINAL: u32 = 480;
const HEIGHT_ORIGINAL: u32 = 360;
const WIDTH: u32 = WIDTH_ORIGINAL / DOWNSCALE as u32;
const HEIGHT: u32 = HEIGHT_ORIGINAL / DOWNSCALE as u32;

const SAMPLE_SIZE: u8 = 4;

fn main() {
    let progress_bar = ProgressBar::new(IMAGE_AMOUNT).with_message("Processing images");
    for i in 0..4 {
        let mut out_file = File::create(format!("frames-{i}.lua")).unwrap();
        out_file.write_all(b"return {").unwrap();
        for inum in (i * 1792)..IMAGE_AMOUNT.min((i + 1) * 1792) {
            let inum = inum + 1;
            let image = image::open(format!("images/{DOWNSCALE}x/frame{inum:04}.png")).unwrap();
            let image = image.into_rgb8();
            let mut coords = "{".to_owned();
            coords.par_extend(
                image
                    .par_enumerate_pixels()
                    .filter(|(_, _, col)| **col != BLACK)
                    .map(|(x, y, _)| format!("{{{x},{y}}}"))
                    .intersperse(",".into()),
            );
            /*
            if inum == IMAGE_AMOUNT.min((i + 1) * 1792) {
                coords.push('}');
            } else {
                coords.push_str("},");
            }
            */

            coords.push_str("},");
            let coords = coords.replace("{,", "{");

            out_file.write_all(coords.as_ref()).unwrap();

            progress_bar.inc(1);
        }
        out_file.write_all(b"}").unwrap();
        progress_bar.println(format!("File {i} done"));
    }
    println!("All done");
}
