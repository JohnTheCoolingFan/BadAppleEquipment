use image::{GenericImageView, ImageBuffer, Rgb, RgbImage, SubImage};

use crate::{BLACK, CANVAS_SIZE, SAMPLE_SIZE, TileId, WHITE};

#[derive(Debug)]
pub struct QuadTree {
    pub root: QuadTreeNode,
}

impl QuadTree {
    pub fn build(image: &ImageBuffer<Rgb<u8>, &mut [u8]>) -> Self {
        QuadTree {
            root: QuadTreeNode::node_from_view(
                &image.view(0, 0, CANVAS_SIZE, CANVAS_SIZE),
                CANVAS_SIZE,
            ),
        }
    }

    pub fn reconstruct_img(&self) -> RgbImage {
        let mut res = RgbImage::new(CANVAS_SIZE, CANVAS_SIZE);
        Self::reconstruct_region(&self.root, &mut res, 0, 0, CANVAS_SIZE);
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
                let masknum = u64::from_str_radix(&tile_id.to_string(), 16).unwrap();
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

    pub fn all_filled_squares(&self) -> Vec<(u32, u32, u32)> {
        self.iter()
            .filter(|node| matches!(node.node, QuadTreeNode::Full))
            .map(|node| (node.x, node.y, node.side_size(512)))
            .collect()
    }

    pub fn iter(&self) -> QuadTreeIter<'_> {
        QuadTreeIter {
            top_tile_size: 512,
            stack: vec![QuadTreeNodeWithCoords {
                node: &self.root,
                x: 0,
                y: 0,
                depth: 0,
            }],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuadTreeNodeWithCoords<'a> {
    pub node: &'a QuadTreeNode,
    pub x: u32,
    pub y: u32,
    pub depth: u8,
}

impl<'a> QuadTreeNodeWithCoords<'a> {
    pub fn side_size(&self, top_side_size: u32) -> u32 {
        top_side_size / (2_u32.pow(self.depth as u32 + 1))
    }
}

#[derive(Debug)]
pub struct QuadTreeIter<'a> {
    top_tile_size: u32,
    stack: Vec<QuadTreeNodeWithCoords<'a>>,
}

impl<'a> Iterator for QuadTreeIter<'a> {
    type Item = QuadTreeNodeWithCoords<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        match node.node {
            QuadTreeNode::Subdivided(children) => {
                let side_size = node.side_size(self.top_tile_size);
                for (child, (x, y)) in children.iter().zip([(0, 0), (1, 0), (0, 1), (1, 1)]).rev() {
                    let x = (x * (side_size / 2)) + node.x;
                    let y = (y * (side_size / 2)) + node.y;
                    let child_context = QuadTreeNodeWithCoords {
                        node: child,
                        x,
                        y,
                        depth: node.depth + 1,
                    };
                    self.stack.push(child_context);
                }
                self.next()
            }
            _ => Some(node),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum QuadTreeNode {
    Full,
    Empty,
    Subdivided(Box<[QuadTreeNode; 4]>),
    Shaped(TileId),
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

    pub fn node_from_view<I: GenericImageView<Pixel = Rgb<u8>>>(
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
            Self::Shaped(TileId::from_samples(bools))
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

    fn get_squares(
        &self,
        anchor_x: u32,
        anchor_y: u32,
        side_size: u32,
    ) -> Option<Vec<(u32, u32, u32)>> {
        match self {
            Self::Shaped(_) | Self::Empty => None,
            Self::Full => Some(vec![(anchor_x, anchor_y, side_size)]),
            Self::Subdivided(children) => Some(
                children
                    .iter()
                    .zip([(0, 0), (1, 0), (0, 1), (1, 1)])
                    .flat_map(|(child, (x, y))| {
                        let new_side_size = side_size / 2;
                        let new_anchor_x = anchor_x + (new_side_size * x);
                        let new_anchor_y = anchor_y + (new_side_size * y);
                        child.get_squares(new_anchor_x, new_anchor_y, new_side_size)
                    })
                    .flatten()
                    .collect(),
            ),
        }
    }

    pub fn get_shapes(&self) -> Vec<&TileId> {
        match self {
            Self::Full | Self::Empty => vec![],
            Self::Shaped(s) => vec![s],
            Self::Subdivided(nodes) => nodes.iter().flat_map(|node| node.get_shapes()).collect(),
        }
    }

    pub fn as_lua(&self) -> String {
        match self {
            Self::Empty => "0".to_string(),
            Self::Full => "1".to_string(),
            Self::Shaped(s) => format!("\"{s}\""),
            Self::Subdivided(children) => {
                let c0 = children[0].as_lua();
                let c1 = children[1].as_lua();
                let c2 = children[2].as_lua();
                let c3 = children[3].as_lua();

                format!("{{{c0},{c1},{c2},{c3}}}")
            }
        }
    }
}
