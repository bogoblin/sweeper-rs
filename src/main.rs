use std::collections::HashMap;
use std::fmt::{Display, Formatter, Write};
use rand::{Error, Rng, SeedableRng};
use rand::prelude::IteratorRandom;

fn main() {
    let world = World::new();
    let chunk = world.chunks.get(&Position(0, 0)).unwrap();
    println!("{}", &chunk);
}

struct World {
    chunks: HashMap<Position, Chunk>,
}
impl World {
    fn new() -> World {
        let mut world = World {
            chunks: Default::default()
        };
        world.add_chunk(Chunk::new(Position(0, 0), 40, 123456));
        world
    }

    fn add_chunk(&mut self, chunk: Chunk) {
        self.chunks.insert(chunk.position.chunk_position(), chunk);
    }
}

#[derive(Eq, Hash, PartialEq)]
struct Position(usize, usize);
impl Position {
    fn chunk_position(&self) -> Position {
        Position(self.0 & !0b1111, self.1 & !0b1111)
    }
    fn position_in_chunk(&self) -> Position {
        Position(self.0 & 0b1111, self.1 & 0b1111)
    }
}

struct Chunk {
    position: Position,
    mines: ChunkBool,
    adjacent: Option<[[u8; 16]; 16]>
}
impl Chunk {
    fn new(position: Position, number_of_mines: u8, seed: u64) -> Chunk {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let mine_indices = (0..255).choose_multiple(&mut rng, number_of_mines as usize);
        let mut mines = ChunkBool::empty();
        for i in mine_indices {
            mines.set_index(i, true);
        }
        Chunk {
            position: position.chunk_position(),
            mines,
            adjacent: None
        }
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
        let col = unsafe { self.0.get_unchecked_mut(x) };
        match value {
            true  => *col |= 1 << y,
            false => *col &= !(1 << y)
        };
    }
    fn set_index(&mut self, index: u8, value: bool) {
        let position = Position(
            (index & 0b1111) as usize,
            (index >> 4) as usize,
        );
        self.set(position, value);
    }
    fn get(&self, position: Position) -> bool {
        // Making sure that x and y are between 0 and 15 so we can get unchecked
        let Position(x, y) = position.position_in_chunk();
        (self.0[x] & (1 << y)) != 0
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        for y in 0..16 {
            for x in 0..16 {
                match self.mines.get(Position(x, y)) {
                    true  => {output.write_char('*')?}
                    false => {output.write_char('_')?}
                }
            }
            output.write_char('\n')?;
        }
        f.write_str(output.as_str())
    }
}