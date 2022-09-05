use std::collections::HashMap;
use std::fmt::{Display, Formatter, Write};
use rand::{Error, Fill, Rng};

fn main() {
    let mut mines = ChunkBool([0; 16]);
    for i in 0..25 {
        mines.set(Position(i, 2*i+1), true);
    }
    let chunk = Chunk{
        position: Position(0, 0),
        mines,
    };
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
        world.add_chunk(Chunk::new(Position(0, 0)));
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
}
impl Chunk {
    fn new(position: Position) -> Chunk {
        Chunk {
            position: position.chunk_position(),
            mines: ChunkBool::empty(),
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
impl Fill for ChunkBool {
    fn try_fill<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Result<(), Error> {
        rng.fill(&mut self.0);
        Ok(())
    }
}