use crate::chunk_store::ChunkStore;
use crate::compression::PublicTile;
use crate::events::Event;
use crate::player::Player;
use crate::server_messages::ServerMessage;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use bitvec::prelude::*;
use bytes_cast::BytesCast;
use derive_more::{Div, Mul};
use huffman::{BitWriter, HuffmanCode};
use rand::prelude::IteratorRandom;
use rand::rngs::StdRng;
use rand::{thread_rng, RngCore, SeedableRng};
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::{max, min, PartialEq};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, AddAssign, Deref, DerefMut, Sub};

pub mod server_messages;
pub mod events;
pub mod compression;
pub mod client_messages;
pub mod chunk_store;
pub mod player;

pub struct World {
    pub chunk_ids: HashMap<ChunkPosition, usize>,
    pub positions: Vec<ChunkPosition>,
    pub chunks: Vec<Chunk>,
    pub seed: u64,

    pub generated_chunks: VecDeque<(ChunkPosition, ChunkMines)>,
    pub chunk_store: ChunkStore,
    pub players: HashMap<String, Player>,
}

impl World {
    pub fn get_tile(&self, position: &Position) -> Tile {
        if let Some(&chunk_id) = self.get_chunk_id(position.clone()) {
            self.chunks[chunk_id].get_tile(position.clone())
        } else {
            Tile::empty()
        }
    }
    
    pub fn get_rect(&self, rect: &Rect) -> UpdatedRect {
        let mut updated_tiles = vec![];
        for position in rect.positions() {
            updated_tiles.push(UpdatedTile {
                position,
                tile: self.get_tile(&position),
            })
        }
        UpdatedRect::new(updated_tiles)
    }
}

impl World {
    pub fn apply_server_message(&mut self, message: &ServerMessage) {
        match message {
            ServerMessage::Event(event) => {
                self.apply_event(event);
            }
            ServerMessage::Chunk(chunk) => {
                self.insert_chunk(chunk.clone());
            }
            ServerMessage::Player(player) => {
                self.players.insert(player.player_id.clone(), player.clone());
            }
            ServerMessage::Welcome(_) => {}
            ServerMessage::Disconnected(_) => {}
            ServerMessage::Connected => {}
            ServerMessage::Rect(rect) => {
                self.apply_updated_rect(rect);
            }
        }
    }

    pub fn apply_event(&mut self, event: &Event) {
        for UpdatedTile {position, tile} in event.tiles_updated() {
            let chunk_id = self.generate_chunk(position);
            self.chunks[chunk_id].set_tile(position, tile);
        }
    }
    
    pub fn apply_updated_rect(&mut self, updated_rect: &UpdatedRect) {
        for UpdatedTile {position, tile} in updated_rect.tiles_updated() {
            let chunk_id = self.generate_chunk(position);
            self.chunks[chunk_id].set_tile(position, tile);
        }
    }
}

impl World {
    pub fn new() -> World {
        let mut world = World {
            chunk_ids: Default::default(),
            positions: vec![],
            chunks: vec![],
            seed: 0,
            generated_chunks: Default::default(),
            chunk_store: ChunkStore::new(),
            players: Default::default(),
        };
        world.generate_chunk(Position(0, 0));
        world
    }
    
    pub fn new_player_id(&mut self) -> String {
        let mut buf: [u8; 8] = Default::default();
        thread_rng().fill_bytes(&mut buf);
        let player_id = BASE64_STANDARD.encode(buf);
        let new_player = Player::new(player_id.clone());
        self.players.insert(player_id.clone(), new_player);
        player_id
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
    
    pub fn query_chunks(&self, rect: &Rect) -> Vec<&Chunk> {
        let query = self.chunk_store.get_chunks(rect);
        if let Ok(query) = query {
            query.into_iter()
                .map(|chunk_id| &self.chunks[chunk_id])
                .collect()
        } else {
            vec![]
        }
    }
    
    pub fn insert_chunk(&mut self, chunk: Chunk) -> usize {
        let new_id = self.chunk_ids.len();
        let existing = self.chunk_ids.entry(chunk.position);
        let position = chunk.position;
        match existing {
            Entry::Occupied(entry) => {
                let chunk_id = *entry.get();
                self.chunks[chunk_id] = chunk;
                chunk_id
            }
            Entry::Vacant(entry) => {
                entry.insert(new_id);
                self.positions.push(position);
                self.chunks.push(chunk);
                self.chunk_store.insert(position, new_id);
                new_id
            }
        }
    }

    pub fn generate_chunk(&mut self, position: Position) -> usize {
        let new_id = self.chunk_ids.len();
        let existing = self.chunk_ids.entry(position.chunk_position());
        match existing {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let position = entry.key().clone();
                let rng = StdRng::seed_from_u64(position.seed(0));
                let mines = ChunkMines::random(40, rng);
                let new_chunk = mines.to_chunk(position);
                entry.insert(new_id);
                self.positions.push(position);
                self.chunks.push(new_chunk);
                self.chunk_store.insert(position, new_id);
                self.generated_chunks.push_back((position, mines));
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
    
    fn set_player_position(&mut self, player_id: &str, position: Position) {
        self.players.insert(player_id.to_string(), Player {
            player_id: player_id.to_string(),
            position,
        });
    }

    pub fn click(&mut self, at: Position, by_player_id: &str) -> Option<Event> {
        self.set_player_position(by_player_id, at);
        let updated = self.reveal(vec![at]);
        if !updated.updated.is_empty() {
            Some(Event::Clicked {
                player_id: by_player_id.to_string(),
                at,
                updated,
            })
        } else {
            None
        }
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
    
    pub fn check_double_click(&self, position: &Position) -> Option<Vec<Position>> {
        let tile = self.get_tile(position);
        if !tile.is_revealed() || tile.adjacent() == 0 {
            return None;
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
            Some(to_reveal)
        } else {
            None
        }
    }

    pub fn double_click(&mut self, position: Position, by_player_id: &str) -> Option<Event> {
        self.set_player_position(by_player_id, position);
        if let Some(to_reveal) = self.check_double_click(&position) {
            let updated = self.reveal(to_reveal);
            Some(Event::DoubleClicked {
                player_id: by_player_id.to_string(),
                at: position,
                updated
            })
        } else { None }
    }

    pub fn flag(&mut self, position: Position, by_player_id: &str) -> Option<Event> {
        self.set_player_position(by_player_id, position);
        if let Some(&chunk_id) = self.get_chunk_id(position) {
            if let Some(&mut ref mut chunk) = self.chunks.get_mut(chunk_id) {
                if let Some(&mut ref mut tile) = chunk.tiles.0.get_mut(position.position_in_chunk().index()) {
                    if !tile.is_revealed() {
                        return if tile.is_flag() {
                            // Unflag
                            *tile = tile.without_flag();
                            Some(Event::Unflag {
                                player_id: by_player_id.to_string(),
                                at: position
                            })
                        } else {
                            // Flag
                            *tile = tile.with_flag();
                            Some(Event::Flag {
                                player_id: by_player_id.to_string(),
                                at: position
                            })
                        }
                    }
                }
            }
        } 
        None
    }
}

#[derive(Serialize, Deserialize)]
#[derive(Default, Clone, Debug)]
pub struct ChunkMines ([u8; 32]);

impl ChunkMines {
    pub fn random(number_of_mines: u8, mut rng: StdRng) -> Self {
        let mut result = Self::default();
        for mine_index in (0..255).choose_multiple(&mut rng, number_of_mines as usize) {
            result.set(mine_index, true);
        }
        result
    }

    pub fn positions(&self) -> Vec<PositionInChunk> {
        let n_ones = self.count_ones();
        let mut result = Vec::with_capacity(n_ones);
        for index in self.iter_ones() {
            result.push(PositionInChunk(index as u8))
        }
        result
    }

    pub fn to_chunk(&self, position: ChunkPosition) -> Chunk {
        let mut new_chunk = Chunk {
            tiles: ChunkTiles([Tile::empty(); 256]),
            position,
            adjacent_mines_filled: false
        };
        for index in self.iter_ones() {
            new_chunk.tiles.0[index] = Tile::mine();
        }
        new_chunk
    }
}

impl Deref for ChunkMines {
    type Target = BitSlice<u8>;

    fn deref(&self) -> &Self::Target {
        self.0.view_bits::<Lsb0>()
    }
}

impl DerefMut for ChunkMines {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.view_bits_mut::<Lsb0>()
    }
}

impl TryFrom<Vec<u8>> for ChunkMines {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let mines: [u8; 32] = *value.first_chunk().ok_or(())?;
        Ok(Self(mines))
    }
}

impl AsRef<[u8]> for ChunkMines {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Copy, Clone, Eq, Hash, PartialEq, Debug, derive_more::Add, derive_more::Sub)]
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
    
    pub fn position(&self) -> Position {
        Position(self.0, self.1)
    }
    pub fn bottom_right(&self) -> Self {
        Self::new(self.0+16, self.1+16)
    }
    
    pub fn position_iter(&self) -> ChunkPositionIter {
        ChunkPositionIter {
            position: self.clone(),
            position_in_chunk_index: 0,
        }
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

    pub fn tile_index(&self) -> usize { self.position_in_chunk().index() }
    
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

#[repr(C)]
#[derive(BytesCast)]
#[derive(Default, Eq, PartialEq, Clone, Copy)]
#[derive(Serialize, Deserialize, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Tile (pub u8);

impl Display for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let char: &str = {
            if self.is_revealed() {
                if self.is_mine() {
                    "*"
                } else {
                    &unsafe {
                        String::from_utf8_unchecked(vec![self.adjacent() + '0' as u8])
                    }
                }
            } else {
                if self.is_flag() {
                    "F"
                } else {
                    " "
                }
            }
        };
        write!(f, "{}", char)
    }
}

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
        let adjacent = self.0 & 0b1111;
        min(adjacent, 8)
    }
}

impl Into<u8> for Tile {
    fn into(self) -> u8 {
        self.0
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ChunkTiles(pub [Tile; 256]);

impl ChunkTiles {
    pub fn from(bytes: [u8; 256]) -> Self {
        Self(bytes.map(|b| Tile(b)))
    }
    
    pub fn bytes(&self) -> &[u8] {
        self.0.as_bytes()
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
#[derive(Debug, Clone)]
pub struct Chunk {
    pub tiles: ChunkTiles,
    pub position: ChunkPosition,
    adjacent_mines_filled: bool
}

impl Chunk {
    pub fn rect(&self) -> Rect {
        Rect {
            left: self.position.0,
            top: self.position.1,
            right: self.position.0 + 16,
            bottom: self.position.1 + 16,
        }
    }
}

impl AddAssign<u8> for Tile {
    fn add_assign(&mut self, rhs: u8) {
        self.0 += rhs;
    }
}

impl Chunk {
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


#[derive(Default, Clone)]
#[derive(Serialize, Deserialize)]
pub struct UpdatedRect {
    pub top_left: Position,
    pub updated: Vec<Vec<Tile>>,
}

impl Debug for UpdatedRect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "UpdatedRect: {:?} with tiles:\n", self.top_left)?;
        for row in &self.updated {
            for tile in row {
                write!(f, "{}", tile)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
        
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
                    tile: tile.clone()
                });
            }
        }
        result
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

pub struct ChunkPositionIter {
    position: ChunkPosition,
    position_in_chunk_index: usize,
}

impl Iterator for ChunkPositionIter {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position_in_chunk_index < 256 {
            let result = Some(Position::from_chunk_positions(&self.position, &PositionInChunk(self.position_in_chunk_index as u8)));
            self.position_in_chunk_index += 1;
            result
        } else {
            None
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
#[derive(Deserialize, Serialize)]
#[derive(Mul, Div)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn from_corners(top_left: Position, bottom_right: Position) -> Rect {
        let Position(left, top) = top_left;
        let Position(right, bottom) = bottom_right;
        Self { left, top, right, bottom }
    }
}

impl Rect {
    pub fn from_top_left_and_size(top_left: Position, width: i32, height: i32) -> Rect {
        let Position(left, top) = top_left;
        Self {
            left, top,
            right: left + width,
            bottom: top + height,
        }
    }
}

impl Rect {
    pub fn positions(&self) -> Vec<Position> {
        if self.right <= self.left || self.bottom <= self.top {
            return vec![];
        }
        let mut result = vec![];
        for x in self.left..self.right {
            for y in self.top..self.bottom {
                result.push(Position(x, y))
            }
        }
        result
    }
}

impl Rect {
    pub fn expand_to_contain(&mut self, rect: Rect) {
        self.left = min(self.left, rect.left);
        self.top = min(self.top, rect.top);
        self.right = max(self.right, rect.right);
        self.bottom = max(self.bottom, rect.bottom);
    }
}

impl Rect {
    pub fn split_x(&self, split: i32) -> Vec<Rect> {
        if split > self.left && split < self.right {
            let mut r1 = self.clone();
            let mut r2 = self.clone();
            r1.right = split;
            r2.left = split;
            vec![r1, r2]
        } else {
            vec![self.clone()]
        }
    }
    
    pub fn split_y(&self, split: i32) -> Vec<Rect> {
        if split > self.top && split < self.bottom {
            let mut r1 = self.clone();
            let mut r2 = self.clone();
            r1.bottom = split;
            r2.top = split;
            vec![r1, r2]
        } else {
            vec![self.clone()]
        }       
    }
    
    pub fn shift(&mut self, x: i32, y: i32) {
        self.left += x;
        self.right += x;
        self.top += y;
        self.bottom += y;
    }
    
    pub fn modulo(&self, modulo: i32) -> Rect {
        let mut result = self.clone();
        result.left %= modulo;
        result.top %= modulo;
        result.right = result.left + self.width();
        result.bottom = result.top + self.height();
        result
    }
}

impl Rect {
    pub fn from_center_and_size(center: Position, width: i32, height: i32) -> Self {
        let left = center.0 - width/2;
        let top = center.1 - height/2;
        Self {
            left, top,
            right: left + width,
            bottom: top + height,
        }
    }
    
    pub fn contains(&self, Position(x, y): Position) -> bool {
        x > self.left &&
            x < self.right &&
            y > self.top &&
            y < self.bottom
    }
    
    pub fn intersection(&self, other: &Rect) -> Option<Self> {
        let left = max(self.left, other.left);
        let right = min(self.right, other.right);
        let top = max(self.top, other.top);
        let bottom = min(self.bottom, other.bottom);
        if left <= right && top <= bottom {
            Some(Self { left, right, top, bottom })
        } else {
            None
        }
    }
    
    pub fn top_left(&self) -> Position {
        Position(self.left, self.top)
    }
    
    pub fn bottom_right(&self) -> Position {
        Position(self.right, self.bottom)
    }
    
    pub fn top_right(&self) -> Position {
        Position(self.right, self.top)
    }
    
    pub fn bottom_left(&self) -> Position {
        Position(self.left, self.bottom)
    }
    
    pub fn area(&self) -> i64 {
        self.width() as i64 * self.height() as i64
    }
    
    pub fn width(&self) -> i32 {
        self.right - self.left
    }
    
    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
    
    pub fn chunks_contained(&self) -> Vec<ChunkPosition> {
        // Adding 15, 15 here so that when it's rounded down, it will always be the first chunk that's totally in it
        let top_left = (self.top_left() + Position(15, 15)).chunk_position();
        let bottom_right = self.bottom_right().chunk_position();
        
        let mut x = top_left.0;
        
        let mut chunks = vec![];
        while x < bottom_right.0 {
            let mut y = top_left.1;
            while y < bottom_right.1 {
                chunks.push(ChunkPosition(x, y));
                y += 16;
            }
            x += 16;
        }
        
        chunks
    }
    
    pub fn chunks_containing(&self) -> Vec<ChunkPosition> {
        let mut new_rect = self.clone();
        new_rect.left -= 15;
        new_rect.top -= 15;
        new_rect.right += 15;
        new_rect.bottom += 15;
        new_rect.chunks_contained()
    }
}