use serde::{Deserialize, Serialize};
use crate::{ChunkPosition, Position, PositionInChunk, Rect};
use crate::chunk::chunk_tiles::ChunkTiles;
use crate::tile::Tile;

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
pub struct Chunk {
    pub tiles: ChunkTiles,
    pub position: ChunkPosition,
    adjacent_mines_filled: bool
}

impl Chunk {
    pub fn empty(position: ChunkPosition) -> Self {
        Self {
            tiles: ChunkTiles([Tile(0); 256]),
            position,
            adjacent_mines_filled: false,
        }
    }

    pub fn from_position_and_tiles(position: ChunkPosition, tiles: ChunkTiles) -> Self {
        Self {
            tiles, position,
            adjacent_mines_filled: true
        }
    }
}

impl Chunk {
    pub fn rect(&self) -> Rect {
        Rect::from_corners(self.position.position(), self.position.bottom_right().position())
    }
}

impl Chunk {
    pub fn should_send(&self) -> bool {
        self.adjacent_mines_filled
    }

    pub fn get_tile(&self, position: Position) -> Tile {
        self.tiles[*position.position_in_chunk()]
    }
    pub fn set_tile(&mut self, position: Position, tile: Tile) -> Tile {
        self.tiles[*position.position_in_chunk()] = tile;
        tile
    }

    pub fn fill_adjacent_mines(surrounding_chunks: [&Chunk; 9]) -> Chunk {
        let is_mine = |position: Position| {
            let Position(x, y) = position;
            // 0 3 6
            // 1 4 7
            // 2 5 8
            let index = if x < 0 {
                if y < 0 {
                    0
                } else if y > 15 {
                    2
                } else {
                    1
                }
            }
            else if x > 15 {
                if y < 0 {
                    6
                } else if y > 15 {
                    8
                } else {
                    7
                }
            }
            else if y < 0 {
                3
            }
            else if y > 15 {
                5
            } else {
                4
            };
            surrounding_chunks[index].get_tile(position).is_mine()
        };

        let mut new_tiles = surrounding_chunks[4].tiles;

        let zero = ChunkPosition::new(0, 0);
        for index in 0..=255 {
            for neighbor in Position::from_chunk_positions(&zero, &PositionInChunk::from_index(index))
                .neighbors() {
                if is_mine(neighbor) { new_tiles[index] += 1; }
            }
        }

        Chunk {
            tiles: new_tiles,
            position: surrounding_chunks[4].position,
            adjacent_mines_filled: true,
        }
    }

    pub fn adjacent_mines_filled(&self) -> bool {
        self.adjacent_mines_filled
    }
}