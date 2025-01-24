use crate::{Chunk, ChunkPosition, Position, Rect};
use quadtree_rs::area::{Area, AreaBuilder};
use quadtree_rs::point::Point;
use quadtree_rs::Quadtree;

pub struct ChunkStore {
    quadtree: Quadtree<u64, usize>
}

impl Default for ChunkStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ChunkStore {
    pub fn new() -> Self {
        Self {
            quadtree: Quadtree::new(32)
        }
    }
    
    pub fn insert_chunks(&mut self, chunks: &Vec<Chunk>) {
        for (chunk_id, chunk) in chunks.iter().enumerate() {
            self.insert(chunk.position, chunk_id);
        }
    }
    
    pub fn insert(&mut self, chunk_position: ChunkPosition, chunk_id: usize) -> Option<u64> {
        self.quadtree.insert(self.area_from(chunk_position), chunk_id)
    }
    
    pub fn get_chunks(&self, rect: &Rect) -> Result<Vec<usize>, String> {
        let area = self.area_from_rect(rect)?;
        Ok(self.quadtree.query(area)
            .map(|entry| *entry.value_ref())
            .collect())
    }
    
    fn quadtree_coords(&self, position: Position) -> Point<u64> {
        let Position(x, y) = position;
        Point::from((
            x.abs_diff(i32::MIN) as u64,
            y.abs_diff(i32::MIN) as u64
        ))
    }
    
    fn area_from(&self, chunk_position: ChunkPosition) -> Area<u64> {
        AreaBuilder::default()
            .anchor(self.quadtree_coords(chunk_position.position()))
            .dimensions((16, 16))
            .build()
            .unwrap()
    }
    
    fn area_from_rect(&self, rect: &Rect) -> Result<Area<u64>, String> {
        AreaBuilder::default()
            .anchor(self.quadtree_coords(rect.top_left()))
            .dimensions((rect.width() as u64, rect.height() as u64))
            .build()
    }
}