use std::ops::{Add, Sub};
use serde::{Deserialize, Serialize};
use crate::ChunkPosition;
use crate::position::position_in_chunk::PositionInChunk;

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone, Default)]
#[derive(Serialize, Deserialize)]
#[derive(derive_more::Mul, derive_more::Div, derive_more::Add, derive_more::Sub)]
pub struct Position(pub i32, pub i32);

impl Position {
    pub fn neighbors(&self) -> Vec<Position> {
        let mut result = vec![];

        for x in self.0-1..=self.0+1 {
            for y in self.1-1..=self.1+1 {
                if x != self.0 || y != self.1 {
                    result.push(Position(x, y))
                }
            }
        }

        result
    }

    pub fn neighbours_and_self(&self) -> Vec<Position> {
        let mut result = vec![];

        for x in self.0-1..=self.0+1 {
            for y in self.1-1..=self.1+1 {
                result.push(Position(x, y))
            }
        }

        result
    }
}

impl Position {
    pub fn origin() -> Self { Self(0, 0) }

    pub fn chunk_position(&self) -> ChunkPosition { ChunkPosition::new(self.0, self.1) }

    pub fn position_in_chunk(&self) -> PositionInChunk { PositionInChunk::new(self.0, self.1) }

    pub fn tile_index(&self) -> u8 { *self.position_in_chunk() }

    pub fn from_chunk_positions(chunk_position: &ChunkPosition, position_in_chunk: &PositionInChunk) -> Self {
        Self(chunk_position.0 + position_in_chunk.x() as i32, chunk_position.1 + position_in_chunk.y() as i32)
    }

    pub fn from_compressed(bytes: &[u8]) -> Option<Self> {
        let x = i32::from_be_bytes(*bytes[0..].first_chunk()?);
        let y = i32::from_be_bytes(*bytes[4..].first_chunk()?);
        Some(Position(x, y))
    }
}

impl Add<(i32, i32)> for &Position {
    type Output = Position;

    fn add(self, rhs: (i32, i32)) -> Position {
        Position(self.0 + rhs.0, self.1 + rhs.1)
    }
}
impl Sub<(i32, i32)> for &Position {
    type Output = Position;

    fn sub(self, rhs: (i32, i32)) -> Position {
        Position(self.0 - rhs.0, self.1 - rhs.1)
    }
}
