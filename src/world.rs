use std::cmp::PartialEq;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::ops;
use std::ops::AddAssign;

use rand::prelude::IteratorRandom;
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};
use serde::Serialize;

pub struct World {
    pub chunk_ids: HashMap<Position, usize>,
    pub positions: Vec<Position>,
    pub chunks: Vec<Chunk>,
    pub rng: StdRng,
}

impl World {
    pub(crate) fn new() -> World {
        let mut world = World {
            chunk_ids: Default::default(),
            positions: Default::default(),
            chunks: Default::default(),
            rng: StdRng::seed_from_u64(0),
        };
        world.generate_chunk(Position(0, 0));
        world
    }

    pub fn get_chunk_id(&self, position: Position) -> Option<&usize> {
        self.chunk_ids.get(&position.chunk_position())
    }

    pub fn generate_chunk(&mut self, position: Position) -> usize {
        let new_id = self.chunk_ids.len();
        let existing = self.chunk_ids.entry(position.chunk_position());
        match existing {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let new_chunk = Chunk::generate(position.chunk_position(), self.rng.next_u64(), 40);
                entry.insert(new_id);
                self.positions.push(position.chunk_position());
                self.chunks.push(new_chunk);
                new_id
            }
        }
    }

    pub fn generate_surrounding_chunks(&mut self, position: Position) -> [usize; 9] {
        [
            self.generate_chunk(&position + (-16, -16)),
            self.generate_chunk(&position + (-16,   0)),
            self.generate_chunk(&position + (-16,  16)),
            self.generate_chunk(&position + (  0, -16)),
            self.generate_chunk(&position + (  0,   0)),
            self.generate_chunk(&position + (  0,  16)),
            self.generate_chunk(&position + ( 16, -16)),
            self.generate_chunk(&position + ( 16,   0)),
            self.generate_chunk(&position + ( 16,  16)),
        ]
    }

    pub fn fill_adjacent_mines(&mut self, position: Position) {
        let surrounding_chunk_ids = self.generate_surrounding_chunks(position);
        let chunk_id = surrounding_chunk_ids[4];
        if let Some(chunk) = self.chunks.get(chunk_id) {
            if chunk.adjacent_mines_filled {
                return;
            }
        }
        unsafe {
            let surrounding_chunks = surrounding_chunk_ids.map(|chunk_id| {
                self.chunks.get_unchecked(chunk_id)
            });
            self.chunks[surrounding_chunk_ids[4]] = Chunk::fill_adjacent_mines(surrounding_chunks)
        }
    }

    pub(crate) fn reveal(&mut self, position: Position) -> RevealResult {
        let chunk_id = self.generate_chunk(position);
        let chunk = match self.chunks.get(chunk_id) {
            None => return RevealResult::Nothing,
            Some(c) => c,
        };
        let tile = chunk.get_tile(position);

        if tile.is_revealed() {
            return RevealResult::Nothing
        }
        if tile.is_mine() {
            return RevealResult::Death(position)
        }

        let mut reveal_stack = vec![position];
        let mut updated_chunk_ids = HashSet::new();

        while let Some(position) = reveal_stack.pop() {
            let current_chunk_id = self.generate_chunk(position);
            self.fill_adjacent_mines(position);
            let mut current_chunk = match self.chunks.get_mut(current_chunk_id) {
                None => continue,
                Some(c) => c,
            };
            let tile = current_chunk.get_tile(position);
            if !tile.is_revealed() {
                let tile = current_chunk.set_tile(position, tile.with_revealed());
                if tile.adjacent() == 0 {
                    for x in -1..=1 {
                        for y in -1..=1 {
                            reveal_stack.push(Position(position.0 + x, position.1 + y));
                        }
                    }
                }
                updated_chunk_ids.insert(current_chunk_id);
            }
        }

        RevealResult::Revealed(updated_chunk_ids)
    }

    pub fn flag(&mut self, position: Position) -> FlagResult {
        if let Some(&chunk_id) = self.get_chunk_id(position) {
            if let Some(&ref chunk) = self.chunks.get(chunk_id) {
                if !chunk.get_tile(position).is_flag() {
                    return FlagResult::Flagged(position)
                }
            }
        }
        FlagResult::Nothing
    }

    pub fn unflag(&mut self, position: Position) -> FlagResult {
        if let Some(&chunk_id) = self.get_chunk_id(position) {
            if let Some(&ref chunk) = self.chunks.get(chunk_id) {
                if chunk.get_tile(position).is_flag() {
                    return FlagResult::Unflagged(position)
                }
            }
        }
        FlagResult::Nothing
    }
}

pub enum RevealResult {
    Death(Position),
    Revealed(HashSet<usize>),
    Nothing,
}
pub enum FlagResult {
    Flagged(Position),
    Unflagged(Position),
    Nothing
}

#[derive(Eq, Hash, PartialEq, Copy, Clone)]
pub struct Position(pub i32, pub i32);
impl Position {
    fn chunk_position(&self) -> Position {
        self - (self.position_in_chunk().0, self.position_in_chunk().1)
    }
    fn position_in_chunk(&self) -> Position {
        Position(self.0 & 0b1111, self.1 & 0b1111)
    }
}
impl ops::Add<(i32, i32)> for &Position {
    type Output = Position;

    fn add(self, rhs: (i32, i32)) -> Position {
        Position(self.0 + rhs.0, self.1 + rhs.1)
    }
}
impl ops::Sub<(i32, i32)> for &Position {
    type Output = Position;

    fn sub(self, rhs: (i32, i32)) -> Position {
        Position(self.0 - rhs.0, self.1 - rhs.1)
    }
}

#[derive(Default, Eq, PartialEq, Clone, Copy, Serialize)]
pub struct Tile (u8);

impl Tile {
    pub fn empty() -> Tile {
        Tile(0)
    }
    pub fn mine() -> Tile {
        Tile::empty().with_mine()
    }
    pub fn with_mine(&self) -> Tile {
        Tile(self.0 | 1<<4)
    }
    pub fn is_mine(&self) -> bool {
        self.0 == self.with_mine().0
    }
    pub fn with_flag(&self) -> Tile {
        Tile(self.0 | 1<<5)
    }
    pub fn is_flag(&self) -> bool {
        self.0 == self.with_flag().0
    }
    pub fn with_revealed(&self) -> Tile {
        Tile(self.0 | 1<<6)
    }
    pub fn is_revealed(&self) -> bool {
        self.0 == self.with_revealed().0
    }
    pub fn with_adjacent(&self, adjacent: u8) -> Tile {
        Tile(self.0 + adjacent)
    }
    pub fn adjacent(&self) -> u8 {
        self.0 & 0b1111
    }
}

pub struct Chunk {
    tiles: [[Tile; 16]; 16],
    pub position: Position,
    adjacent_mines_filled: bool
}

impl AddAssign<u8> for Tile {
    fn add_assign(&mut self, rhs: u8) {
        self.0 += rhs;
    }
}

impl Chunk {
    pub fn generate(position: Position, seed: u64, number_of_mines: u8) -> Chunk {
        let mut new_chunk = Chunk {
            tiles: Default::default(),
            position,
            adjacent_mines_filled: false
        };
        let mut rng = StdRng::seed_from_u64(seed);
        for mine_index in (0..255).choose_multiple(&mut rng, number_of_mines as usize) {
            let x = mine_index % 16;
            let y = (mine_index - x) >> 4;
            new_chunk.set_tile(Position(x, y), Tile::mine());
        }
        new_chunk
    }

    pub fn get_tile(&self, position: Position) -> Tile {
        let x = position.0 as usize;
        let y = position.1 as usize;
        self.tiles[x%16][y%16]
    }
    pub fn set_tile(&mut self, position: Position, tile: Tile) -> Tile {
        let x = position.0 as usize;
        let y = position.1 as usize;
        self.tiles[x%16][y%16] = tile;
        tile
    }

    pub fn fill_adjacent_mines(surrounding_chunks: [&Chunk; 9]) -> Chunk {
        let is_mine = |position: Position| {
            let (x, y) = (position.0, position.1);
            let tile_is_mine = |index: usize| -> bool {
                return surrounding_chunks[index].get_tile(position).is_mine();
            };
            // 0 3 6
            // 1 4 7
            // 2 5 8
            if x < 0 {
                if y < 0 {
                    return tile_is_mine(0);
                } else if y > 15 {
                    return tile_is_mine(2);
                }
                return tile_is_mine(1);
            }
            else if x > 15 {
                if y < 0 {
                    return tile_is_mine(6);
                } else if y > 15 {
                    return tile_is_mine(8);
                }
                return tile_is_mine(7);
            }
            else if y < 0 {
                return tile_is_mine(3);
            }
            else if y > 15 {
                return tile_is_mine(5);
            }
            tile_is_mine(4)
        };

        let mut new_tiles: [[Tile; 16]; 16] = Default::default();

        for x in 0..16 {
            for y in 0..16 {
                for xo in -1..=1 {
                    for yo in -1..=1 {
                        if is_mine(Position(x+xo, y+yo)) {
                            new_tiles[x as usize][y as usize] += 1;
                        }
                    }
                }
            }
        }

        Chunk {
            tiles: new_tiles,
            position: surrounding_chunks[4].position,
            adjacent_mines_filled: true,
        }
    }
}