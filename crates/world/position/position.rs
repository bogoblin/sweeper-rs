use std::ops::{Add, Sub};
use quickcheck::{Arbitrary, Gen};
use serde::{Deserialize, Serialize};
use crate::ChunkPosition;
use crate::position::position_in_chunk::PositionInChunk;

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone, Default)]
#[derive(Serialize, Deserialize)]
#[derive(derive_more::Mul, derive_more::Div)]
pub struct Position(pub i32, pub i32);

impl Position {
    pub fn neighbors(&self) -> Vec<Position> {
        let mut result = self.neighbors_and_self();
        result.remove(4);
        result
    }

    pub fn neighbors_and_self(&self) -> Vec<Position> {
        let mut result = vec![];

        for x in -1..=1 {
            for y in -1..=1 {
                result.push(Position(self.0.overflowing_add(x).0, self.1.overflowing_add(y).0))
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

impl Add<Position> for Position {
    type Output = Position;

    fn add(self, rhs: Position) -> Self::Output {
        Position(
            self.0.overflowing_add(rhs.0).0,
            self.1.overflowing_add(rhs.1).0,
        )
    }
}

impl Sub<Position> for Position {
    type Output = Position;

    fn sub(self, rhs: Position) -> Self::Output {
        Position(
            self.0.overflowing_sub(rhs.0).0,
            self.1.overflowing_sub(rhs.1).0,
        )
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

impl Arbitrary for Position {
    fn arbitrary(g: &mut Gen) -> Self {
        Self(i32::arbitrary(g), i32::arbitrary(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item=Self>> {
        Box::from(self.0.shrink().zip(self.1.shrink())
            .map(|(x, y)| Self(x, y)))
    }
}
