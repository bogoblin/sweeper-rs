use serde::{Deserialize, Serialize};
use crate::Position;
use crate::position::position_in_chunk::PositionInChunk;

#[derive(Copy, Clone, Eq, Hash, PartialEq, Debug, derive_more::Add, derive_more::Sub)]
#[derive(Serialize, Deserialize)]
pub struct ChunkPosition(pub i32, pub i32);

impl ChunkPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self(x & !0b1111, y & !0b1111)
    }

    pub fn seed(&self, salt: u64) -> u64 {
        (self.0 as u64).overflowing_add(
            (self.1 as u64) << 31
        ).0 + salt
    }

    pub fn position(&self) -> Position {
        Position(self.0, self.1)
    }
    pub fn bottom_right(&self) -> Self {
        Self::new(self.0+16, self.1+16)
    }

    pub fn position_iter(&self) -> ChunkPositionIter {
        ChunkPositionIter {
            position: *self,
            in_chunk: PositionInChunk::first().into()
        }
    }
}

pub struct ChunkPositionIter {
    position: ChunkPosition,
    in_chunk: Option<PositionInChunk>,
}

impl Iterator for ChunkPositionIter {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        let in_chunk = self.in_chunk?;
        let result = Some(Position::from_chunk_positions(&self.position, &in_chunk));
        self.in_chunk = in_chunk.next();
        result
    }
}