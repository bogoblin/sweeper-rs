use std::collections::HashMap;
use std::fmt::{Display, Formatter, Write};
use std::ops;
use rand::{SeedableRng};
use rand::prelude::IteratorRandom;

fn main() {
    let mut world = World::new();
    world.fill_adjacent_mines(Position(0, 0));
    let chunk = world.get_or_generate_chunk(Position(0, 0));
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
    fn get_chunk(&self, position: Position) -> Option<&Chunk> {
        let position = position.chunk_position();
        self.chunks.get(&position)
    }
    fn get_or_generate_chunk(&mut self, position: Position) -> &Chunk {
        self.chunks.entry(position.chunk_position()).or_insert(
            Chunk::new(position.chunk_position(), 40, 123456)
        )
    }
    fn fill_adjacent_mines(&mut self, position: Position) {
        let adj = AdjacentMines::for_chunk(position.chunk_position(), self);
        match self.chunks.get_mut(&position.chunk_position()) {
            None => {}
            Some(chunk) => {
                chunk.adjacent = Some(adj);
            }
        }
    }
}

#[derive(Eq, Hash, PartialEq)]
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

struct Chunk {
    position: Position,
    mines: ChunkBool,
    revealed: ChunkBool,
    adjacent: Option<AdjacentMines>,
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
            revealed: ChunkBool::empty(),
            adjacent: None
        }
    }
    fn is_mine(&self, position: Position) -> bool {
        let position = position.position_in_chunk();
        self.mines.get(position)
    }
    fn is_revealed(&self, position: Position) -> bool {
        let position = position.position_in_chunk();
        self.revealed.get(position)
    }
    fn adjacent_to(&self, position: Position) -> Option<u8> {
        self.adjacent?.get(position.position_in_chunk())
    }
}

struct AdjacentMines([[u8; 16]; 16]);

impl AdjacentMines {
    fn for_chunk(position: Position, world: &mut World) -> AdjacentMines {
        let top_left = world.get_or_generate_chunk(&position + (-16, -16))
            .mines.get(Position(15, 15));
        let top_right = world.get_or_generate_chunk(&position + (16, -16))
            .mines.get(Position(0, 15));
        let bottom_left = world.get_or_generate_chunk(&position + (-16, 16))
            .mines.get(Position(15, 0));
        let bottom_right = world.get_or_generate_chunk(&position + (16, 16))
            .mines.get(Position(0, 0));

        let _ = &world.get_or_generate_chunk(&position + (0, -16));
        let _ = &world.get_or_generate_chunk(&position + (0, 16));
        let _ = &world.get_or_generate_chunk(&position + (16, 0));
        let _ = &world.get_or_generate_chunk(&position + (-16, 0));
        let _ = &world.get_or_generate_chunk(position.chunk_position());

        let top = &world.get_chunk(&position + (0, -16)).unwrap().mines;
        let bottom = &world.get_chunk(&position + (0, 16)).unwrap().mines;
        let right = &world.get_chunk(&position + (16, 0)).unwrap().mines;
        let left = &world.get_chunk(&position + (-16, 0)).unwrap().mines;
        let this = &world.get_chunk(position.chunk_position()).unwrap().mines;

        let is_mine = |position: Position| {
            if position.0 < 0 {
                if position.1 < 0 {
                    return top_left
                } else if position.1 > 15 {
                    return bottom_left
                }
                return left.get(position);
            }
            else if position.0 > 15 {
                if position.1 < 0 {
                    return top_right
                } else if position.1 > 15 {
                    return bottom_right
                }
                return right.get(position);
            }
            else if position.1 < 0 {
                return top.get(position);
            }
            else if position.1 > 15 {
                return bottom.get(position);
            }
            return this.get(position);
        };

        let mut adj = AdjacentMines([[0; 16]; 16]);

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

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        for y in 0..16 {
            for x in 0..16 {
                match self.mines.get(Position(x, y)) {
                    true  => {output.write_char('*')?}
                    false => {
                        match &self.adjacent {
                            None => {output.write_char('_')?}
                            Some(adj) => {
                                output.write_str(&*format!("{}", adj.0[x as usize][y as usize]))?
                            }
                        }
                    }
                }
            }
            output.write_char('\n')?;
        }
        f.write_str(output.as_str())
    }
}
