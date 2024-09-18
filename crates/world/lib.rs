use std::cmp::PartialEq;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::fmt::{Debug, Formatter};
use std::ops;
use std::ops::{AddAssign, Sub};

use rand::{SeedableRng};
use rand::prelude::IteratorRandom;
use rand::rngs::StdRng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, Visitor};
use crate::compression::PublicTile;
use crate::events::Event;

pub mod server_messages;
pub mod events;
pub mod compression;

#[derive(Serialize, Deserialize)]
pub struct World {
    pub chunk_ids: HashMap<ChunkPosition, usize>,
    pub positions: Vec<ChunkPosition>,
    pub chunks: Vec<Chunk>,
    pub seed: u64,

    #[serde(skip)]
    pub events: Vec<Event>,
}


impl World {
    pub fn new() -> World {
        let mut world = World {
            chunk_ids: Default::default(),
            positions: vec![],
            chunks: vec![],
            seed: 0,
            events: vec![],
        };
        world.generate_chunk(Position(0, 0));
        world
    }

    pub fn get_chunk_id(&self, position: Position) -> Option<&usize> {
        self.chunk_ids.get(&position.chunk_position())
    }

    pub fn get_chunk(&self, position: Position) -> Option<&Chunk> {
        if let Some(&chunk_id) = self.get_chunk_id(position) {
            return self.chunks.get(chunk_id);
        }
        None
    }

    pub fn generate_chunk(&mut self, position: Position) -> usize {
        let new_id = self.chunk_ids.len();
        let existing = self.chunk_ids.entry(position.chunk_position());
        match existing {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let position = entry.key().clone();
                let new_chunk = Chunk::generate(position.clone(), 40, self.seed);
                entry.insert(new_id);
                self.positions.push(position);
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
        let surrounding_chunks = surrounding_chunk_ids.map(|chunk_id| unsafe {
            self.chunks.get_unchecked(chunk_id)
        });
        self.chunks[surrounding_chunk_ids[4]] = Chunk::fill_adjacent_mines(surrounding_chunks)
    }

    pub fn click(&mut self, at: Position, by_player_id: &str) {
        let updated = self.reveal(vec![at]);
        self.events.push(Event::Clicked {
            player_id: by_player_id.to_string(),
            at,
            updated
        });
    }

    fn reveal(&mut self, mut to_reveal: Vec<Position>) -> UpdatedRect {
        if to_reveal.is_empty() {
            return Default::default();
        }

        let mut updated_chunk_ids = HashSet::new();
        let mut updated_tiles = vec![];

        while let Some(position) = to_reveal.pop() {
            let current_chunk_id = self.generate_chunk(position);
            self.fill_adjacent_mines(position);
            let current_chunk = match self.chunks.get_mut(current_chunk_id) {
                None => continue,
                Some(c) => c,
            };
            // It's okay to get unchecked here because we know that tile_index() always returns a number < 256
            let tile = unsafe { current_chunk.tiles.0.get_unchecked_mut(position.tile_index()) };
            if !tile.is_revealed() {
                *tile = tile.with_revealed();
                if tile.adjacent() == 0 {
                    for x in -1..=1 {
                        for y in -1..=1 {
                            to_reveal.push(Position(position.0 + x, position.1 + y));
                        }
                    }
                }
                updated_tiles.push(UpdatedTile {position, tile: tile.clone()});
                updated_chunk_ids.insert(current_chunk_id);
            }
        }

        UpdatedRect::new(updated_tiles)
    }

    pub fn double_click(&mut self, position: Position, by_player_id: &str) {
        let chunk = match self.get_chunk(position) {
            Some(chunk) => chunk,
            None => return,
        };
        let tile = chunk.get_tile(position);
        if !tile.is_revealed() || tile.adjacent() == 0 {
            return;
        }
        let mut surrounding_flags = 0;
        let mut to_reveal = vec![];
        for x in -1..=1 {
            for y in -1..=1 {
                let pos = Position(position.0 + x, position.1 + y);
                // TODO: speed up this part, it will be slow because it keeps getting the same chunk from the hashmap
                if let Some(chunk) = self.get_chunk(pos) {
                    let t = chunk.get_tile(pos);
                    if !t.is_revealed() {
                        if t.is_flag() {
                            surrounding_flags += 1;
                        } else {
                            to_reveal.push(pos);
                        }
                    } else if t.is_mine() {
                        surrounding_flags += 1;
                    }
                }
            }
        }
        if surrounding_flags == tile.adjacent() {
            let updated = self.reveal(to_reveal);
            self.events.push(Event::DoubleClicked {
                player_id: by_player_id.to_string(),
                at: position,
                updated
            });
            return;
        }
    }

    pub fn flag(&mut self, position: Position, by_player_id: &str) {
        if let Some(&chunk_id) = self.get_chunk_id(position) {
            if let Some(&mut ref mut chunk) = self.chunks.get_mut(chunk_id) {
                if let Some(&mut ref mut tile) = chunk.tiles.0.get_mut(position.position_in_chunk().index()) {
                    if tile.is_flag() {
                        // Unflag
                        *tile = tile.without_flag();
                        self.events.push(Event::Unflag {
                            player_id: by_player_id.to_string(),
                            at: position
                        });
                    } else {
                        // Flag
                        *tile = tile.with_flag();
                        self.events.push(Event::Flag {
                            player_id: by_player_id.to_string(),
                            at: position
                        });
                    }
                }
            }
        }
    }
}

#[derive(Copy, Clone, Eq, Hash, PartialEq, Debug)]
#[derive(Serialize, Deserialize)]
pub struct ChunkPosition(pub i32, pub i32);

impl ChunkPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self(x & !0b1111, y & !0b1111)
    }

    fn seed(&self, salt: u64) -> u64 {
        (self.0 as u64).overflowing_add(
            (self.1 as u64) << 31
        ).0 + salt
    }
}

#[derive(Clone, Copy)]
#[derive(Serialize, Deserialize)]
pub struct PositionInChunk(u8);

impl PositionInChunk {
    pub fn new(x: i32, y:i32) -> Self {
        Self(((x & 0b1111) + ((y & 0b1111) << 4)) as u8)
    }

    pub fn x(&self) -> u8 {
        self.0 & 0b1111
    }

    pub fn y(&self) -> u8 {
        (self.0 >> 4) & 0b1111
    }

    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone, Default)]
#[derive(Serialize, Deserialize)]
pub struct Position(pub i32, pub i32);
impl Position {
    pub fn origin() -> Self { Self(0, 0) }

    fn chunk_position(&self) -> ChunkPosition { ChunkPosition::new(self.0, self.1) }

    fn position_in_chunk(&self) -> PositionInChunk { PositionInChunk::new(self.0, self.1) }

    pub fn tile_index(&self) -> usize { self.position_in_chunk().index() }
}
impl ops::Add<(i32, i32)> for &Position {
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

#[derive(Default, Eq, PartialEq, Clone, Copy)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Tile (pub u8);

impl Tile {
    pub const fn empty() -> Tile {
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
    pub fn without_flag(&self) -> Tile {
        Tile(self.0 & !(1<<5))
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
    pub fn adjacent(&self) -> u8 {
        self.0 & 0b1111
    }
}

impl Into<u8> for Tile {
    fn into(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Debug)]
pub struct ChunkTiles(pub [Tile; 256]);

impl ChunkTiles {
    pub fn from(bytes: [u8; 256]) -> Self {
        Self(bytes.map(|b| Tile(b)))
    }
}

impl Serialize for ChunkTiles {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        serializer.serialize_bytes(self.0.map(|t| t.0).as_slice())
    }
}

struct ChunkTileVisitor;
impl<'de> Visitor<'de> for ChunkTileVisitor {
    type Value = ChunkTiles;

    fn expecting(&self, _formatter: &mut Formatter) -> std::fmt::Result {
        todo!()
    }

    fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if let Some(&chunk_bytes) = bytes.first_chunk::<256>() {
            Ok(Self::Value::from(chunk_bytes))
        } else {
            Err(Error::invalid_length(bytes.len(), &self))
        }
    }
}
impl<'de> Deserialize<'de> for ChunkTiles {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let visitor = ChunkTileVisitor{};
        deserializer.deserialize_bytes(visitor)
    }
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Chunk {
    pub tiles: ChunkTiles,
    pub position: ChunkPosition,
    adjacent_mines_filled: bool
}

impl AddAssign<u8> for Tile {
    fn add_assign(&mut self, rhs: u8) {
        self.0 += rhs;
    }
}

impl Chunk {
    pub fn generate(position: ChunkPosition, number_of_mines: u8, salt: u64) -> Chunk {
        let mut new_chunk = Chunk {
            tiles: ChunkTiles([Tile::empty(); 256]),
            position,
            adjacent_mines_filled: false
        };
        let mut rng = StdRng::seed_from_u64(position.seed(salt));
        for mine_index in (0..255).choose_multiple(&mut rng, number_of_mines as usize) {
            let x = mine_index % 16;
            let y = (mine_index - x) >> 4;
            new_chunk.set_tile(Position(x, y), Tile::mine());
        }
        new_chunk
    }
    
    pub fn should_send(&self) -> bool {
        self.adjacent_mines_filled
    }

    pub fn get_tile(&self, position: Position) -> Tile {
        self.tiles.0[position.position_in_chunk().index()]
    }
    pub fn set_tile(&mut self, position: Position, tile: Tile) -> Tile {
        self.tiles.0[position.position_in_chunk().index()] = tile;
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

        let mut new_tiles = surrounding_chunks[4].tiles.clone();

        for x in 0..16 {
            for y in 0..16 {
                let tile_index = PositionInChunk::new(x, y).index();
                for xo in -1..=1 {
                    for yo in -1..=1 {
                        if is_mine(Position(x+xo, y+yo)) {
                            new_tiles.0[tile_index] += 1;
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

#[derive(Clone, Debug)]
pub struct UpdatedTile {
    position: Position,
    tile: Tile,
}


#[derive(Default)]
#[derive(Serialize, Deserialize)]
pub struct UpdatedRect {
    pub top_left: Position,
    pub updated: Vec<Vec<Tile>>,
}

impl Debug for UpdatedRect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("UpdatedRect")
    }
}

impl Sub for Position {
    type Output = Position;

    fn sub(self, rhs: Self) -> Self::Output {
        Position(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl UpdatedRect {
    pub fn empty() -> Self {
        Self {top_left: Position::origin(), updated: vec![]}
    }
    pub fn new(updated_tiles: Vec<UpdatedTile>) -> Self {
        let first_tile = match updated_tiles.get(0) {
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
}