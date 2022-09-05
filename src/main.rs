use std::fmt::{Display, Formatter, Write};
use rand::{Error, Fill, Rng};

fn main() {
    let mut mines = ChunkBool([0; 16]);
    for i in 0..25 {
        mines.set(i, 2*i+1, true);
    }
    let chunk = Chunk{mines};
    println!("{}", &chunk);
}

struct Chunk {
    mines: ChunkBool,
}

// A boolean value for each tile in a 16x16 chunk
struct ChunkBool([u16; 16]);

impl ChunkBool {
    fn set(&mut self, x: usize, y: usize, value: bool) {
        // Making sure that x and y are between 0 and 15 so we can get unchecked
        let x = x & 0b1111;
        let y = y & 0b1111;

        let col = unsafe {
            self.0.get_unchecked_mut(x)
        };
        match value {
            true  => *col |= 1 << y,
            false => *col &= !(1 << y)
        };
    }
    fn get(&self, x: usize, y: usize) -> bool {
        // Making sure that x and y are between 0 and 15 so we can get unchecked
        let x = x & 0b1111;
        let y = y & 0b1111;

        (self.0[x] & (1 << y)) != 0
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        for y in 0..16 {
            for x in 0..16 {
                match self.mines.get(x, y) {
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