use quadtree_rs::area::{Area, AreaBuilder};
use quadtree_rs::point::Point;
use quadtree_rs::Quadtree;
use wasm_bindgen::prelude::wasm_bindgen;
use world::ChunkPosition;

#[wasm_bindgen]
pub struct ChunkQuadtree(Quadtree<u64, ChunkPosition>);

impl ChunkQuadtree {
    pub fn new() -> Self {
        Self {
            0: Quadtree::new(32)
        }
    }

    pub fn insert(&mut self, position: ChunkPosition) {
        if let Ok(area) = area_from(position) {
            self.0.insert(area, position);
        } else {
        }
    }

    pub fn query(&self, top_left: ChunkPosition, bottom_right: ChunkPosition) -> Vec<&ChunkPosition> {
        let ChunkPosition(width, height) = bottom_right.bottom_right() - top_left;
        if let Ok(area) = AreaBuilder::default()
            .anchor(to_quadtree_coords(top_left))
            .dimensions((width as u64, height as u64))
            .build() {
            self.0.query(area)
                .map(|item| {
                    item.value_ref()
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        }
    }
}

fn area_from (chunk_position: ChunkPosition) -> Result<Area<u64>, String> {
    AreaBuilder::default()
        .anchor(to_quadtree_coords(chunk_position))
        .dimensions((16, 16))
        .build()
}


/// The quadtree crate uses coordinates starting from 0, so I have to map my i32s to u64s like so:
/// ```
/// use quadtree_rs::point::Point;
/// use world::ChunkPosition;
/// use client::chunk_quadtree::to_quadtree_coords;
/// assert_eq!(Point::from((0, 0)), to_quadtree_coords(ChunkPosition::new(i32::MIN, i32::MIN)));
pub fn to_quadtree_coords(chunk_position: ChunkPosition) -> Point<u64> {
    let ChunkPosition(x, y) = chunk_position;
    Point::from((
        x.abs_diff(i32::MIN) as u64,
        y.abs_diff(i32::MIN) as u64
    ))
}