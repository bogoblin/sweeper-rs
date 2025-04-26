use std::fmt::{Display, Formatter};
use quickcheck::{Arbitrary, Gen};
use PublicTile::*;
use huffman_derive::huffman_derive;
use crate::tile::Tile;

#[huffman_derive(
    Hidden => 40,
    Flag => 10,
    Exploded => 5,
    Adjacent0 => 25,
    Adjacent1 => 20,
    Adjacent2 => 12,
    Adjacent3 => 3,
    Adjacent4 => 0.5,
    Adjacent5 => 0.1,
    Adjacent6 => 0.04,
    Adjacent7 => 0.001,
    Adjacent8 => 0.0001,
    Newline => 15
)]
#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash)]
#[repr(u8)]
pub enum PublicTile {
    Hidden = 0,
    Flag = Tile::empty().with_flag().0,
    Exploded = Tile::empty().with_revealed().with_mine().0,
    Adjacent0 = Tile::empty().with_revealed().0,
    Adjacent1 = Tile::empty().with_revealed().0 + 1,
    Adjacent2 = Tile::empty().with_revealed().0 + 2,
    Adjacent3 = Tile::empty().with_revealed().0 + 3,
    Adjacent4 = Tile::empty().with_revealed().0 + 4,
    Adjacent5 = Tile::empty().with_revealed().0 + 5,
    Adjacent6 = Tile::empty().with_revealed().0 + 6,
    Adjacent7 = Tile::empty().with_revealed().0 + 7,
    Adjacent8 = Tile::empty().with_revealed().0 + 8,
    Newline = u8::MAX,
}

impl From<&Tile> for PublicTile {
    fn from(value: &Tile) -> Self {
        if value.is_revealed() {
            if value.is_mine() {
                Exploded
            } else {
                match value.adjacent() {
                    0 => Adjacent0,
                    1 => Adjacent1,
                    2 => Adjacent2,
                    3 => Adjacent3,
                    4 => Adjacent4,
                    5 => Adjacent5,
                    6 => Adjacent6,
                    7 => Adjacent7,
                    8 => Adjacent8,
                    _ => panic!("Uh oh what have we got here...")
                }
            }
        } else {
            if value.is_flag() {
                Flag
            } else {
                Hidden
            }
        }
    }
}

impl From<Tile> for PublicTile {
    fn from(value: Tile) -> Self {
        (&value).into()
    }
}


impl From<&PublicTile> for Tile {
    fn from(value: &PublicTile) -> Self {
        Tile(value.clone() as u8)
    }
}

impl From<PublicTile> for Tile {
    fn from(value: PublicTile) -> Self {
        (&value).into()
    }
}

impl Arbitrary for PublicTile {
    fn arbitrary(g: &mut Gen) -> Self {
        let tile = Tile::arbitrary(g);
        tile.into()
    }
}

impl Display for PublicTile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let tile = Tile::from(self);
        write!(f, "{tile}")
    }
}