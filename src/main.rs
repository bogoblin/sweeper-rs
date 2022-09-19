use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops;
use rand::{SeedableRng};
use rand::prelude::IteratorRandom;

fn main() {
    let mut world = World::new();
    let chunk_id = world.generate_chunk(Position(0, 0));
    world.fill_adjacent_mines(Position(0, 0));
    println!("{}", world.display_chunk(chunk_id));
}

struct World {
    chunk_ids: HashMap<Position, usize>,

    mines: Vec<ChunkBool>,
    flags: Vec<ChunkBool>,
    revealed: Vec<ChunkBool>,
    adjacent_mines: Vec<Option<AdjacentMines>>,

    rng: rand::rngs::StdRng,
}
impl World {
    fn new() -> World {
        let mut world = World {
            chunk_ids: Default::default(),
            mines: Default::default(),
            flags: Default::default(),
            revealed: Default::default(),
            adjacent_mines: Default::default(),
            rng: rand::rngs::StdRng::seed_from_u64(0),
        };
        world.generate_chunk(Position(0, 0));
        world
    }

    fn generate_chunk(&mut self, position: Position) -> usize {
        let new_id = self.chunk_ids.len();
        let existing = self.chunk_ids.entry(position.chunk_position());
        match existing {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                entry.insert(new_id);

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

    fn generate_surrounding_chunks(&mut self, position: Position) -> [usize; 9] {
        [
            self.generate_chunk(&position + (-1, -1)),
            self.generate_chunk(&position + ( 0, -1)),
            self.generate_chunk(&position + ( 1, -1)),
            self.generate_chunk(&position + (-1,  0)),
            self.generate_chunk(&position + ( 0,  0)),
            self.generate_chunk(&position + ( 1,  0)),
            self.generate_chunk(&position + (-1,  1)),
            self.generate_chunk(&position + ( 0,  1)),
            self.generate_chunk(&position + ( 1,  1)),
        ]
    }

    fn fill_adjacent_mines(&mut self, position: Position) {
        let surrounding_chunk_ids = self.generate_surrounding_chunks(position);
        let chunk_id = surrounding_chunk_ids[4];
        let adjacent_mines = AdjacentMines::for_chunk(&self.mines, surrounding_chunk_ids);
        self.adjacent_mines.insert(chunk_id, Some(adjacent_mines));
    }

    fn display_chunk(&self, chunk_id: usize) -> String {
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
}

#[derive(Eq, Hash, PartialEq, Copy, Clone)]
struct Position(i32, i32);
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

struct AdjacentMines([[u8; 16]; 16]);

impl AdjacentMines {
    fn get(&self, position: Position) -> u8 {
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
struct ChunkBool([u16; 16]);

impl ChunkBool {
    fn empty() -> ChunkBool {
        ChunkBool([0; 16])
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
    fn get(&self, position: Position) -> bool {
        // Making sure that x and y are between 0 and 15 so we can get unchecked
        let Position(x, y) = position.position_in_chunk();
        (self.0[x as usize] & (1 << y as usize)) != 0
    }
}

// impl Display for Chunk {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         let mut output = String::new();
//         for y in 0..16 {
//             for x in 0..16 {
//                 match self.mines.get(Position(x, y)) {
//                     true  => {output.write_char('*')?}
//                     false => {
//                         match &self.adjacent {
//                             None => {output.write_char('_')?}
//                             Some(adj) => {
//                                 output.write_str(&*format!("{}", adj.0[x as usize][y as usize]))?
//                             }
//                         }
//                     }
//                 }
//             }
//             output.write_char('\n')?;
//         }
//         f.write_str(output.as_str())
//     }
// }
