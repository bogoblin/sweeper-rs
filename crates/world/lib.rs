use crate::chunk_store::ChunkStore;
use crate::player::Player;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use rand::rngs::StdRng;
use rand::{thread_rng, RngCore, SeedableRng};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};

pub mod chunk_store;
pub mod player;
mod position;
mod rect;
mod chunk;
mod tile;
mod updates;

pub use rect::Rect;
pub use position::*;
pub use chunk::*;
pub use chunk::ChunkMines;
pub use tile::Tile;
pub use updates::*;

pub struct World {
    pub chunk_ids: HashMap<ChunkPosition, usize>,
    pub chunks: Vec<Chunk>,
    pub seed: u64,

    pub generated_chunks: VecDeque<(ChunkPosition, ChunkMines)>,
    pub chunk_store: ChunkStore,
    pub players: HashMap<String, Player>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn get_tile(&self, position: &Position) -> Tile {
        if let Some(&chunk_id) = self.get_chunk_id(*position) {
            self.chunks[chunk_id].get_tile(*position)
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

    pub fn apply_updated_rect(&mut self, updated_rect: UpdatedRect) -> Vec<usize> {
        // OPTIMIZATION: we are able to optimize this by doing smarter caching
        // on the chunk_ids so that we don't have to do the hash lookup each time
        let mut chunk_ids_updated = vec![];
        for UpdatedTile {position, tile} in updated_rect.tiles_updated() {
            let chunk_id = self.empty_or_existing_chunk(position.chunk_position());
            self.chunks[chunk_id].set_tile(position, tile);
            chunk_ids_updated.push(chunk_id);
        }
        chunk_ids_updated
    }

    fn empty_or_existing_chunk(&mut self, position: ChunkPosition) -> usize {
        self.get_chunk_id(position.position()).copied()
            .unwrap_or_else(|| self.insert_chunk(Chunk::empty(position)))
    }

    pub fn new() -> World {
        let mut world = World {
            chunk_ids: Default::default(),
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
    
    pub fn query_chunks(&self, rect: &Rect) -> Vec<usize> {
        self.chunk_store.get_chunks(rect).unwrap_or_default()
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
                let position = *entry.key();
                let rng = StdRng::seed_from_u64(position.seed(0));
                let mines = ChunkMines::random(40, rng);
                let new_chunk = mines.to_chunk(position);
                entry.insert(new_id);
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
            if chunk.adjacent_mines_filled() {
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
            let tile = &mut current_chunk.tiles[position.tile_index()];
            if !tile.is_revealed() {
                *tile = tile.with_revealed();
                if tile.adjacent() == 0 {
                    to_reveal.append(&mut position.neighbors());
                }
                updated_tiles.push(UpdatedTile {position, tile: *tile});
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
        for pos in position.neighbors() {
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
        let chunk_id = *self.get_chunk_id(position)?;
        self.set_player_position(by_player_id, position);
        let chunk = self.chunks.get_mut(chunk_id)?;
        let tile = &mut chunk.tiles[*position.position_in_chunk()];
        if !tile.is_revealed() {
            if tile.is_flag() {
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
        } else {
            None
        }
    }
}