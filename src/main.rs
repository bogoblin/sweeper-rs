use std::collections::HashMap;
use std::fmt::{Display, Formatter, Write};
use std::ops;
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
    fn generate_chunk(&mut self, position: Position) -> Option<&Chunk> {
        let chunk = Chunk::new(position.chunk_position(), 40, 23098723);
        self.add_chunk(chunk);
        self.get_chunk(position)
    }
    fn get_chunk(&self, position: Position) -> Option<&Chunk> {
        let position = position.chunk_position();
        self.chunks.get(&position)
    }
    fn get_or_generate_chunk(&mut self, position: Position) -> &Chunk {
        let key = &position.chunk_position();
        return if self.chunks.contains_key(key) {
            self.chunks.get(key).expect("Got existing chunk")
        } else {
            self.generate_chunk(position).expect("Generated new chunk")
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
struct Vector(i32, i32);
impl ops::Add<&Vector> for &Position {
    type Output = Position;

    fn add(self, rhs: &Vector) -> Position {
        Position(self.0 + rhs.0, self.1 + rhs.1)
    }
}
impl ops::Mul<i32> for &Vector {
    type Output = Vector;

    fn mul(self, rhs: i32) -> Vector {
        Vector(self.0*rhs, self.1*rhs)
    }
}
impl ops::Mul<&Vector> for i32 {
    type Output = Vector;

    fn mul(self, rhs: &Vector) -> Vector {
        rhs*self
    }
}
impl ops::Sub<&Vector> for &Position {
    type Output = Position;

    fn sub(self, rhs: &Vector) -> Position {
        self + &(-1 * rhs)
    }
}

struct Chunk {
    position: Position,
    mines: ChunkBool,
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
            adjacent: None
        }
    }
}

struct AdjacentMines([[u8; 16]; 16]);

impl AdjacentMines {
    fn for_chunk(chunk: &Chunk, world: &mut World) -> AdjacentMines {
        let top_left = world.get_or_generate_chunk(&chunk.position + &Vector(-16, -16))
            .mines.get(Position(15, 15));
        let top_right = world.get_or_generate_chunk(&chunk.position + &Vector(16, -16))
            .mines.get(Position(0, 15));
        let bottom_left = world.get_or_generate_chunk(&chunk.position + &Vector(-16, 16))
            .mines.get(Position(15, 0));
        let bottom_right = world.get_or_generate_chunk(&chunk.position + &Vector(16, 16))
            .mines.get(Position(0, 0));

        let top = &world.get_or_generate_chunk(&chunk.position + &Vector(0, -16)).mines;
        let bottom = &world.get_or_generate_chunk(&chunk.position + &Vector(0, 16)).mines;
        let right = &world.get_or_generate_chunk(&chunk.position + &Vector(16, 0)).mines;
        let left = &world.get_or_generate_chunk(&chunk.position + &Vector(-16, 0)).mines;

        let this = &chunk.mines;

        let is_mine = |position| {
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

        for x in 1..16-1 {
            for y in 1..16-1 {
                for xo in -1..=1 {
                    for yo in -1..=1 {
                        adj[x][y] += is_mine(Position(x+xo, y+yo));
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
                    false => {output.write_char('_')?}
                }
            }
            output.write_char('\n')?;
        }
        f.write_str(output.as_str())
    }
}
