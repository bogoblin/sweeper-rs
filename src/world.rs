use std::collections::hash_map::Entry;
use std::collections::{HashMap};
use std::ops;
use rand::{SeedableRng};
use rand::prelude::IteratorRandom;

pub struct World {
    pub chunk_ids: HashMap<Position, usize>,

    pub positions: Vec<Position>,
    pub mines: Vec<ChunkBool>,
    pub flags: Vec<ChunkBool>,
    pub revealed: Vec<ChunkBool>,
    pub adjacent_mines: Vec<Option<AdjacentMines>>,

    pub rng: rand::rngs::StdRng,
}

impl World {
    pub(crate) fn new() -> World {
        let mut world = World {
            chunk_ids: Default::default(),
            positions: Default::default(),
            mines: Default::default(),
            flags: Default::default(),
            revealed: Default::default(),
            adjacent_mines: Default::default(),
            rng: rand::rngs::StdRng::seed_from_u64(0),
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
                entry.insert(new_id);
                self.positions.push(position.chunk_position());

                // Generate mines
                let mut mines = ChunkBool::empty();
                let number_of_mines = 40;
                let mine_indices = (0..255).choose_multiple(&mut self.rng, number_of_mines as usize);
                mine_indices.into_iter().for_each(|i| mines.set_index(i, true));
                self.mines.push(mines);

                self.flags.push(ChunkBool::empty());
                self.revealed.push(ChunkBool::empty());
                self.adjacent_mines.push(None);

                new_id
            }
        }
    }

    pub fn generate_surrounding_chunks(&mut self, position: Position) -> [usize; 9] {
        [
            self.generate_chunk(&position + (-16, -16)),
            self.generate_chunk(&position + (  0, -16)),
            self.generate_chunk(&position + ( 16, -16)),
            self.generate_chunk(&position + (-16,   0)),
            self.generate_chunk(&position + (  0,   0)),
            self.generate_chunk(&position + ( 16,   0)),
            self.generate_chunk(&position + (-16,  16)),
            self.generate_chunk(&position + (  0,  16)),
            self.generate_chunk(&position + ( 16,  16)),
        ]
    }

    pub fn get_or_fill_adjacent_mines(&mut self, position: Position) -> &AdjacentMines {
        let surrounding_chunk_ids = self.generate_surrounding_chunks(position);
        let chunk_id = surrounding_chunk_ids[4];
        self.adjacent_mines[chunk_id].get_or_insert(
            AdjacentMines::for_chunk(&self.mines, surrounding_chunk_ids))
    }

    pub fn display_chunk(&self, chunk_id: usize) -> String {
        let mines = &self.mines[chunk_id];

        let empty_adjacent = &AdjacentMines::empty();
        let adjacent = self.adjacent_mines[chunk_id].as_ref().unwrap_or(empty_adjacent);

        let mut output = String::new();
        for y in 0..16 {
            for x in 0..16 {
                let position = Position(x, y);
                let mine_string = match (mines.get(position), adjacent.get(position)) {
                    (true, _) => String::from("*"),
                    (false, 0) => String::from("_"),
                    (false, adj) => format!("{}", adj),
                };
                output.push_str(&mine_string);
            }
            output.push('\n');
        }
        output
    }

    pub(crate) fn reveal(&mut self, position: Position) -> RevealResult {
        let chunk_id = self.generate_chunk(position);

        if self.revealed[chunk_id].get(position) {
            return RevealResult::Nothing
        }
        if self.mines[chunk_id].get(position) {
            return RevealResult::Death(position)
        }

        let mut to_reveal = HashMap::new();
        let mut reveal_stack = Vec::new();
        reveal_stack.push(position);

        let mut current_chunk = position.chunk_position();
        let mut current_chunk_id = chunk_id;
        let mut current_chunk_adjacent = self.get_or_fill_adjacent_mines(position);
        let mut current_chunk_to_reveal = to_reveal.entry(current_chunk_id).or_insert(ChunkBool::empty());
        while let Some(position) = reveal_stack.pop() {
            if position.chunk_position() != current_chunk {
                current_chunk = position.chunk_position();
                current_chunk_id = self.generate_chunk(position);
                current_chunk_adjacent = self.get_or_fill_adjacent_mines(position);
                current_chunk_to_reveal = to_reveal.entry(current_chunk_id).or_insert(ChunkBool::empty());
            }
            if current_chunk_to_reveal.get(position) {
                continue;
            }
            current_chunk_to_reveal.set(position, true);
            if current_chunk_adjacent.get(position) == 0 {
                // First push positions in other chunks:
                for x in -1..=1 {
                    for y in -1..=1 {
                        let pos = &position + (x, y);
                        if pos.chunk_position() != current_chunk {
                            reveal_stack.push(pos);
                        }
                    }
                }
                // Then push positions in this chunk:
                for x in -1..=1 {
                    for y in -1..=1 {
                        let pos = &position + (x, y);
                        if pos.chunk_position() == current_chunk
                            && !current_chunk_to_reveal.get(pos) {
                            reveal_stack.push(pos);
                        }
                    }
                }
            }
        }

        let revealed_in_chunk = ChunkBool::with_true(&[position]);
        to_reveal.insert(chunk_id, revealed_in_chunk);
        RevealResult::Revealed(to_reveal)
    }

    pub fn apply_reveal(&mut self, reveal_result: RevealResult) {
        match reveal_result {
            RevealResult::Death(position) => {
                match self.get_chunk_id(position) {
                    None => return,
                    Some(&chunk_id) => {
                        self.revealed[chunk_id].set(position, true);
                    }
                }
            }
            RevealResult::Revealed(to_reveal) => {
                for (chunk_id, mut chunk_reveal) in to_reveal.into_iter() {
                    let already_revealed = &self.revealed[chunk_id];
                    for i in 0..16 {
                        chunk_reveal.0[i] |= already_revealed.0[i];
                    }
                    self.revealed[chunk_id] = chunk_reveal;
                }
            }
            RevealResult::Nothing => {}
        };
    }

    pub fn flag(&mut self, position: Position) -> FlagResult {
        if let Some(&chunk_id) = self.get_chunk_id(position) {
            if !self.flags[chunk_id].get(position) {
                return FlagResult::Flagged(position);
            }
        }
        FlagResult::Nothing
    }

    pub fn unflag(&mut self, position: Position) -> FlagResult {
        if let Some(&chunk_id) = self.get_chunk_id(position) {
            if self.flags[chunk_id].get(position) {
                return FlagResult::Unflagged(position);
            }
        }
        FlagResult::Nothing
    }

    pub fn apply_flag_result(&mut self, flag_result: FlagResult) {
        match flag_result {
            FlagResult::Flagged(position) => {
                if let Some(&chunk_id) = self.get_chunk_id(position) {
                    self.flags[chunk_id].set(position, true);
                }
            }
            FlagResult::Unflagged(position) => {
                if let Some(&chunk_id) = self.get_chunk_id(position) {
                    self.flags[chunk_id].set(position, false);
                }
            }
            FlagResult::Nothing => {}
        }
    }
}

pub enum RevealResult {
    Death(Position),
    Revealed(HashMap<usize, ChunkBool>),
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
        Position(self.0 & !0b1111, self.1 & !0b1111)
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

pub struct AdjacentMines([[u8; 16]; 16]);

impl AdjacentMines {
    pub(crate) fn get(&self, position: Position) -> u8 {
        let position = position.position_in_chunk();
        self.0[position.0 as usize][position.1 as usize]
    }

    fn empty() -> AdjacentMines { AdjacentMines([[0; 16]; 16]) }

    fn for_chunk(mines: &Vec<ChunkBool>, surrounding_chunk_ids: [usize; 9]) -> AdjacentMines {
        let surrounding_chunks_mines: Vec<&ChunkBool> = surrounding_chunk_ids.into_iter().map(|id| &mines[id]).collect();

        let is_mine = |position: Position| {
            let (x, y) = (position.0, position.1);
            // 0 1 2
            // 3 4 5
            // 6 7 8
            if x < 0 {
                if y < 0 {
                    return surrounding_chunks_mines[0].get(position);
                } else if y > 15 {
                    return surrounding_chunks_mines[6].get(position);
                }
                return surrounding_chunks_mines[3].get(position);
            }
            else if x > 15 {
                if y < 0 {
                    return surrounding_chunks_mines[2].get(position);
                } else if y > 15 {
                    return surrounding_chunks_mines[8].get(position);
                }
                return surrounding_chunks_mines[5].get(position);
            }
            else if y < 0 {
                return surrounding_chunks_mines[1].get(position);
            }
            else if y > 15 {
                return surrounding_chunks_mines[7].get(position);
            }
            return surrounding_chunks_mines[4].get(position);
        };

        let mut adj = AdjacentMines::empty();

        for x in 0..16 {
            for y in 0..16 {
                for xo in -1..=1 {
                    for yo in -1..=1 {
                        adj.0[x as usize][y as usize] += match is_mine(Position(x+xo, y+yo)) {
                            true => 1,
                            false => 0
                        }
                    }
                }
            }
        }
        adj
    }
}

// A boolean value for each tile in a 16x16 chunk
pub struct ChunkBool([u16; 16]);

impl ChunkBool {
    fn empty() -> ChunkBool {
        ChunkBool([0; 16])
    }

    fn with_true(positions: &[Position]) -> ChunkBool {
        let mut result = ChunkBool::empty();
        for position in positions {
            result.set(position.position_in_chunk(), true);
        }
        result
    }

    fn set(&mut self, position: Position, value: bool) {
        // Making sure that x and y are between 0 and 15 so we can get unchecked
        let Position(x, y) = position.position_in_chunk();
        let col = unsafe { self.0.get_unchecked_mut(x as usize) };
        match value {
            true  => *col |= 1 << y,
            false => *col &= !(1 << y)
        };
    }
    fn set_index(&mut self, index: u8, value: bool) {
        let position = Position(
            (index & 0b1111) as i32,
            (index >> 4) as i32,
        );
        self.set(position, value);
    }
    pub(crate) fn get(&self, position: Position) -> bool {
        // Making sure that x and y are between 0 and 15 so we can get unchecked
        let Position(x, y) = position.position_in_chunk();
        (self.0[x as usize] & (1 << y as usize)) != 0
    }
}
