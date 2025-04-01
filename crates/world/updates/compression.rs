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
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum PublicTile {
    Hidden,
    Flag,
    Exploded,
    Adjacent0,
    Adjacent1,
    Adjacent2,
    Adjacent3,
    Adjacent4,
    Adjacent5,
    Adjacent6,
    Adjacent7,
    Adjacent8,
    Newline,
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
        match value {
            Hidden => Tile::empty(),
            Flag => Tile::empty().with_flag(),
            Exploded => Tile::mine().with_revealed(),
            Adjacent0 => Tile(0).with_revealed(),
            Adjacent1 => Tile(1).with_revealed(),
            Adjacent2 => Tile(2).with_revealed(),
            Adjacent3 => Tile(3).with_revealed(),
            Adjacent4 => Tile(4).with_revealed(),
            Adjacent5 => Tile(5).with_revealed(),
            Adjacent6 => Tile(6).with_revealed(),
            Adjacent7 => Tile(7).with_revealed(),
            Adjacent8 => Tile(8).with_revealed(),
            Newline => Tile::empty(),
        }
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