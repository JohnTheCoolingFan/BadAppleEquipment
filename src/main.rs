use std::{collections::HashMap, fs::File, io::Write, process::exit, time::Instant};

use image::{
    GenericImage, GenericImageView, ImageBuffer, ImageReader, Pixel, Rgb, RgbImage, SubImage,
};
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

const SAMPLE_SIZE: u32 = 8;
const CHUNKS_X: u32 = WIDTH / SAMPLE_SIZE;
const CHUNKS_Y: u32 = HEIGHT / SAMPLE_SIZE;

fn encode_pixels_hex(values: [bool; (SAMPLE_SIZE * SAMPLE_SIZE) as usize]) -> String {
    let mut res = String::with_capacity(values.len() / 4);

    for chunk in values.as_chunks::<8>().0.iter().rev() {
        let val: u8 = chunk
            .iter()
            .rev()
            .map(|v| if *v { 1 } else { 0 })
            .fold(0, |acc, v| (acc << 1) | v);

        res.push_str(&format!("{val:02x}"));
    }

    res
}

#[derive(Debug)]
struct QuadTree {
    root: QuadTreeNode,
}

impl QuadTree {
    fn build(image: &RgbImage) -> Self {
        QuadTree {
            root: QuadTreeNode::node_from_view(&image.view(0, 0, 512, 512), 512),
        }
    }

    fn reconstruct_img(&self) -> RgbImage {
        let mut res = RgbImage::new(512, 512);
        res.fill(0);
        Self::reconstruct_region(&self.root, &mut res, 0, 0, 512);
        res
    }

    fn reconstruct_region(
        node: &QuadTreeNode,
        img: &mut RgbImage,
        anchor_x: u32,
        anchor_y: u32,
        side_size: u32,
    ) {
        match node {
            // image empty by default
            QuadTreeNode::Empty => {}
            QuadTreeNode::Full => {
                for y in 0..side_size {
                    for x in 0..side_size {
                        img.put_pixel(x + anchor_x, y + anchor_y, WHITE);
                    }
                }
            }
            QuadTreeNode::Shaped(tile_id) => {
                let masknum = u64::from_str_radix(tile_id, 16).unwrap();
                for i in 0..(side_size * side_size) {
                    let x = i % side_size;
                    let y = i / side_size;
                    let pix_value = (masknum >> i) & 1;
                    if pix_value == 1 {
                        img.put_pixel(x + anchor_x, y + anchor_y, WHITE);
                    } else {
                        img.put_pixel(x + anchor_x, y + anchor_y, BLACK);
                    }
                }
            }
            QuadTreeNode::Subdivided(children) => {
                for (child, (x, y)) in children.iter().zip([(0, 0), (1, 0), (0, 1), (1, 1)]) {
                    let new_side_size = side_size / 2;
                    let new_anchor_x = anchor_x + (new_side_size * x);
                    let new_anchor_y = anchor_y + (new_side_size * y);
                    Self::reconstruct_region(child, img, new_anchor_x, new_anchor_y, new_side_size);
                }
            }
        }
    }
}

#[derive(Debug)]
enum QuadTreeNode {
    Full,
    Empty,
    Subdivided(Box<[QuadTreeNode; 4]>),
    Shaped(String),
}

impl QuadTreeNode {
    fn all_white<I: GenericImageView<Pixel = Rgb<u8>>>(img: &SubImage<&I>, side_size: u32) -> bool {
        (0..side_size)
            .flat_map(|y| (0..side_size).map(move |x| img.get_pixel(x, y)))
            .all(|v| v != BLACK)
    }

    fn all_black<I: GenericImageView<Pixel = Rgb<u8>>>(img: &SubImage<&I>, side_size: u32) -> bool {
        !(0..side_size)
            .flat_map(|y| (0..side_size).map(move |x| img.get_pixel(x, y)))
            .any(|v| v != BLACK)
    }

    fn node_from_view<I: GenericImageView<Pixel = Rgb<u8>>>(
        view: &SubImage<&I>,
        side_size: u32,
    ) -> Self {
        if Self::all_white(view, side_size) {
            Self::Full
        } else if Self::all_black(view, side_size) {
            Self::Empty
        } else if side_size == SAMPLE_SIZE {
            let mut bools = [false; (SAMPLE_SIZE * SAMPLE_SIZE) as usize];
            for x in 0..SAMPLE_SIZE {
                for y in 0..SAMPLE_SIZE {
                    bools[(x + y * SAMPLE_SIZE) as usize] = view.get_pixel(x, y) != BLACK
                }
            }
            Self::Shaped(encode_pixels_hex(bools))
        } else {
            let coords = [(0, 0), (1, 0), (0, 1), (1, 1)];
            let subdivisions = coords.map(|(x, y)| {
                let new_side_size = side_size / 2;
                let new_view = view.view(
                    x * new_side_size,
                    y * new_side_size,
                    new_side_size,
                    new_side_size,
                );
                Self::node_from_view(&new_view, new_side_size)
            });
            Self::Subdivided(Box::new(subdivisions))
        }
    }

    fn get_shapes(&self) -> Vec<&String> {
        match self {
            Self::Full | Self::Empty => vec![],
            Self::Shaped(s) => vec![s],
            Self::Subdivided(nodes) => nodes.iter().flat_map(|node| node.get_shapes()).collect(),
        }
    }

    fn as_lua(&self) -> String {
        match self {
            Self::Empty => "0".to_string(),
            Self::Full => "1".to_string(),
            Self::Shaped(s) => format!("\"{s}\""),
            Self::Subdivided(children) => {
                let c0 = children[0].as_lua();
                let c1 = children[1].as_lua();
                let c2 = children[2].as_lua();
                let c3 = children[3].as_lua();

                format!("{{{c0}, {c1}, {c2}, {c3}}}")
            }
        }
    }
}

fn main() {
    let progress_bar = ProgressBar::new(IMAGE_AMOUNT).with_message("Processing images");
    let mut chunk_counts: HashMap<String, u32> = HashMap::new();
    let mut out_file = File::create("frames-tree.lua").unwrap();
    out_file.write_all(b"return {").unwrap();
    let mut working_buffer = ImageBuffer::new(512, 512);
    for inum in 0..IMAGE_AMOUNT {
        let inum = inum + 1;
        let image = image::open(format!("images/{DOWNSCALE}x/frame{inum:04}.png")).unwrap();
        let image = image.into_rgb8();
        working_buffer.copy_from(&image, 0, 0).unwrap();

        let quad_tree = QuadTree::build(&working_buffer);

        for tile_id in quad_tree.root.get_shapes() {
            *chunk_counts.entry(tile_id.clone()).or_insert(0) += 1;
        }

        let quad_tree_lua = quad_tree.root.as_lua();

        out_file.write_all(quad_tree_lua.as_bytes()).unwrap();
        out_file.write_all(b",").unwrap();

        progress_bar.inc(1);
    }
    out_file.write_all(b"}").unwrap();
    println!("All done");
    println!("Number of unique chunks: {}", chunk_counts.len());

    let mut used_numbers_file = File::create("used-tiles.lua").unwrap();
    used_numbers_file.write_all(b"return {").unwrap();
    for tile_id in chunk_counts.keys() {
        used_numbers_file
            .write_all(format!("\"{tile_id}\",").as_ref())
            .unwrap();
    }
    used_numbers_file.write_all(b"}").unwrap();

    let mut out_stats = File::create(format!("tile_stats_{DOWNSCALE}x.csv")).unwrap();
    writeln!(&mut out_stats, "tile_id,amount").unwrap();
    for (tile_id, amount) in chunk_counts.iter() {
        writeln!(&mut out_stats, "tile_{tile_id},{amount}").unwrap();
    }
    out_stats.flush().unwrap();
}
