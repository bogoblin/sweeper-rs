use std::fmt::{Debug, Formatter};
use huffman::{BitWriter, HuffmanCode};
use serde::{Deserialize, Serialize};
use crate::{Position, Tile};
use crate::PublicTile;

#[derive(Clone, Debug)]
pub struct UpdatedTile {
    pub position: Position,
    pub tile: Tile,
}

#[derive(Default, Clone)]
#[derive(Serialize, Deserialize)]
pub struct UpdatedRect {
    pub top_left: Position,
    pub updated: Vec<Vec<Tile>>,
}

impl Debug for UpdatedRect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "UpdatedRect: {:?} with tiles:", self.top_left)?;
        for y in 0..self.height() {
            for x in 0..self.width() {
                write!(f, "{}", self.updated[x][y])?;
            }
            writeln!(f)?;
        }
        Ok(())

    }
}

impl UpdatedRect {
    pub fn empty() -> Self {
        Self {top_left: Position::origin(), updated: vec![]}
    }
    pub fn new(updated_tiles: Vec<UpdatedTile>) -> Self {
        let first_tile = match updated_tiles.first() {
            None => return Self::empty(),
            Some(t) => t,
        };

        let Position(mut min_x, mut min_y) = first_tile.position;
        let mut max_x = min_x;
        let mut max_y = min_y;

        for updated_tile in &updated_tiles {
            let Position(x, y) = updated_tile.position;
            if x < min_x { min_x = x; }
            if x > max_x { max_x = x; }
            if y < min_y { min_y = y; }
            if y > max_y { max_y = y; }
        }

        let top_left = Position(min_x, min_y);

        let n_cols = max_x + 1 - min_x;
        let n_rows = max_y + 1 - min_y;

        let mut updated = vec![];
        for i in 0..n_cols {
            updated.push(vec![]);
            for _j in 0..n_rows {
                updated[i as usize].push(Tile::empty())
            }
        }

        for updated_tile in &updated_tiles {
            let Position(x, y) = updated_tile.position - top_left;
            // It crashed here at the 2 billion boundary because it was trying to make a huge rect.
            // This is possible to fix with a wrapping boundary, which I haven't implemented just yet.
            // Maybe we can configure the boundary size hmm
            if x > 1000 || y > 1000 || x < 0 || y < 0 {
                return Self::empty()
            }
            updated[x as usize][y as usize] = updated_tile.tile;
        }

        Self {
            top_left,
            updated
        }
    }

    pub fn public_tiles(&self) -> Vec<PublicTile> {
        let mut result = vec![];
        for row in &self.updated {
            for tile in row {
                result.push(tile.into())
            }
            result.push(PublicTile::Newline)
        }
        result
    }

    pub fn tiles_updated(&self) -> Vec<UpdatedTile> {
        let mut result = vec![];
        for (x, col) in self.updated.iter().enumerate() {
            for (y, tile) in col.iter().enumerate() {
                if *tile == Tile::empty() {
                    continue
                }
                let position = self.top_left + Position(x as i32, y as i32);
                result.push(UpdatedTile {
                    position,
                    tile: *tile
                });
            }
        }
        result
    }

    pub fn width(&self) -> usize {
        self.updated.first().map_or(0, |col| col.len())
    }

    pub fn height(&self) -> usize {
        self.updated.len()
    }
}

impl From<&UpdatedRect> for Vec<u8> {
    fn from(updated: &UpdatedRect) -> Self {
        let mut binary = vec![];
        let Position(x, y) = updated.top_left;
        binary.append(&mut x.to_be_bytes().to_vec());
        binary.append(&mut y.to_be_bytes().to_vec());
        let mut bw = BitWriter::new();
        let public_tiles = updated.public_tiles();
        for tile in public_tiles {
            tile.encode(&mut bw);
        }
        binary.append(&mut bw.to_bytes());
        binary
    }
}