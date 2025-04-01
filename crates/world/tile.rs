use std::cmp::min;
use bytes_cast::BytesCast;
use serde::{Deserialize, Serialize};
use std::ops::AddAssign;
use std::fmt::{Display, Formatter};
use quickcheck::{Arbitrary, Gen};

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
                        String::from_utf8_unchecked(vec![self.adjacent() + b'0'])
                    }
                }
            } else if self.is_flag() {
                "F"
            } else {
                " "
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
        Tile(self.0 | (1<<4))
    }
    pub fn is_mine(&self) -> bool {
        self.0 == self.with_mine().0
    }
    pub fn with_flag(&self) -> Tile {
        Tile(self.0 | (1<<5))
    }
    pub fn without_flag(&self) -> Tile {
        Tile(self.0 & !(1<<5))
    }
    pub fn is_flag(&self) -> bool {
        self.0 == self.with_flag().0
    }
    pub fn with_revealed(&self) -> Tile {
        Tile(self.0 | (1<<6))
    }
    pub fn is_revealed(&self) -> bool {
        self.0 == self.with_revealed().0
    }
    pub fn adjacent(&self) -> u8 {
        let adjacent = self.0 & 0b1111;
        min(adjacent, 8)
    }
}

impl From<Tile> for u8 {
    fn from(value: Tile) -> Self {
        value.0
    }
}

impl AddAssign<u8> for Tile {
    fn add_assign(&mut self, rhs: u8) {
        self.0 += rhs;
    }
}

impl Arbitrary for Tile {
    /// You may want to use PublicTile::arbitrary(g).into() instead
    fn arbitrary(g: &mut Gen) -> Self {
        Tile(u8::arbitrary(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item=Self>> {
        todo!()
    }
}