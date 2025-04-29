use crate::tile::Tile;
use byte_pair_encoding::BytePairEncoding;
use bytemuck::NoUninit;
use huffman_derive::huffman_derive;
use lazy_static::lazy_static;
use quickcheck::{Arbitrary, Gen};
use std::fmt::{Display, Formatter};
use PublicTile::*;

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
#[derive(NoUninit)]
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

impl PublicTile {
    pub fn from_compressed_bytes(bytes: Vec<u8>) -> Vec<Self> {
        // Self::from_huffman_bytes(bytes)
        BPE.decode(&bytes[..])
            .iter().map(|&byte| PublicTile::from(byte))
            .collect()
    }
    
    pub fn compress_tiles(public_tiles: &[PublicTile]) -> Vec<u8> {
        // let mut bw = BitWriter::new();
        // for tile in public_tiles {
        //     tile.encode(&mut bw);
        // }
        // bw.to_bytes()
        BPE.encode(bytemuck::cast_slice(public_tiles))
    }
}

impl From<u8> for PublicTile {
    fn from(value: u8) -> Self {
        if value == 255 {
            Newline
        } else {
            Tile(value).into()
        }
    }
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

lazy_static! {
    static ref BPE: BytePairEncoding = BytePairEncoding::from_replacements([
        None,
        Some((0, 0)),
        Some((1, 1)),
        Some((2, 2)),
        Some((3, 3)),
        Some((4, 4)),
        Some((5, 5)),
        Some((64, 64)),
        Some((65, 65)),
        Some((3, 2)),
        Some((6, 6)),
        Some((1, 0)),
        Some((65, 7)),
        Some((64, 65)),
        Some((9, 0)),
        Some((7, 7)),
        Some((5, 4)),
        Some((10, 6)),
        Some((65, 66)),
        Some((9, 1)),
        Some((3, 11)),
        Some((66, 8)),
        Some((3, 1)),
        Some((65, 13)),
        Some((66, 66)),
        Some((64, 8)),
        Some((9, 11)),
        Some((2, 11)),
        Some((3, 0)),
        Some((2, 0)),
        Some((2, 1)),
        Some((12, 7)),
        None,
        Some((64, 66)),
        Some((8, 66)),
        Some((12, 65)),
        Some((66, 7)),
        Some((10, 16)),
        Some((17, 4)),
        Some((12, 15)),
        Some((8, 65)),
        Some((6, 4)),
        Some((66, 18)),
        Some((12, 13)),
        Some((6, 16)),
        Some((26, 65)),
        Some((10, 5)),
        Some((17, 16)),
        Some((17, 5)),
        Some((66, 65)),
        Some((6, 5)),
        Some((14, 65)),
        Some((13, 19)),
        Some((8, 8)),
        Some((10, 4)),
        Some((7, 13)),
        Some((7, 65)),
        Some((66, 15)),
        Some((26, 66)),
        Some((66, 9)),
        Some((14, 66)),
        Some((12, 64)),
        Some((14, 23)),
        Some((19, 65)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some((0, 66)),
        Some((65, 25)),
        Some((35, 9)),
        Some((67, 66)),
        Some((24, 66)),
        Some((20, 66)),
        Some((13, 66)),
        None,
        Some((65, 22)),
        Some((9, 66)),
        Some((52, 52)),
        Some((0, 65)),
        Some((11, 65)),
        Some((61, 9)),
        Some((12, 18)),
        Some((14, 12)),
        Some((66, 22)),
        Some((12, 25)),
        Some((20, 43)),
        Some((45, 45)),
        Some((8, 18)),
        Some((12, 8)),
        Some((3, 65)),
        Some((15, 65)),
        Some((1, 65)),
        Some((3, 66)),
        Some((7, 66)),
        Some((19, 66)),
        Some((63, 64)),
        Some((22, 66)),
        Some((15, 13)),
        Some((51, 33)),
        Some((27, 66)),
        Some((14, 21)),
        Some((36, 65)),
        Some((60, 13)),
        Some((67, 67)),
        Some((1, 66)),
        Some((56, 14)),
        Some((31, 13)),
        Some((0, 23)),
        Some((67, 9)),
        Some((33, 19)),
        Some((20, 21)),
        Some((67, 8)),
        Some((30, 66)),
        Some((14, 40)),
        Some((25, 66)),
        Some((55, 9)),
        Some((29, 66)),
        Some((20, 36)),
        Some((21, 66)),
        Some((80, 65)),
        Some((7, 8)),
        Some((31, 65)),
        Some((2, 65)),
        Some((28, 65)),
        Some((2, 66)),
        Some((23, 0)),
        Some((19, 49)),
        Some((14, 42)),
        Some((0, 67)),
        Some((24, 67)),
        Some((33, 28)),
        Some((21, 65)),
        Some((27, 67)),
        Some((20, 65)),
        Some((12, 66)),
        Some((14, 67)),
        Some((26, 67)),
        Some((22, 31)),
        Some((66, 67)),
        Some((7, 18)),
        Some((33, 0)),
        Some((21, 15)),
        Some((9, 65)),
        Some((28, 66)),
        Some((22, 65)),
        Some((20, 8)),
        Some((21, 8)),
        Some((29, 65)),
        Some((12, 59)),
        Some((25, 8)),
        Some((7, 33)),
        Some((18, 19)),
        Some((39, 65)),
        Some((14, 34)),
        Some((4, 19)),
        Some((7, 34)),
        Some((12, 34)),
        Some((24, 65)),
        Some((11, 66)),
        Some((0, 21)),
        Some((25, 65)),
        Some((23, 59)),
        Some((28, 67)),
        Some((9, 67)),
        Some((80, 16)),
        Some((30, 65)),
        Some((9, 75)),
        Some((27, 65)),
        Some((8, 67)),
        Some((24, 18)),
        Some((8, 13)),
        Some((31, 8)),
        Some((67, 65)),
        Some((27, 21)),
        Some((86, 86)),
        Some((67, 2)),
        Some((57, 81)),
        Some((31, 81)),
        Some((8, 55)),
        Some((34, 62)),
        Some((32, 32)),
        Some((36, 64)),
        Some((20, 31)),
        Some((5, 1)),
        Some((13, 0)),
        Some((27, 39)),
        Some((67, 22)),
        Some((51, 125)),
        Some((6, 3)),
        Some((82, 23)),
        Some((29, 67)),
        Some((100, 64)),
        Some((18, 66)),
        Some((9, 121)),
        Some((42, 8)),
        Some((47, 29)),
        Some((58, 58)),
        Some((15, 15)),
        Some((31, 18)),
        Some((2, 39)),
        Some((21, 18)),
        Some((15, 55)),
        Some((20, 67)),
        Some((12, 33)),
        Some((13, 28)),
        Some((83, 8)),
        Some((21, 56)),
        Some((14, 111)),
        Some((28, 8)),
        Some((28, 39)),
        Some((41, 9)),
        Some((3, 74)),
        Some((37, 27)),
        Some((187, 9)),
        Some((2, 18)),
        Some((19, 8)),
        Some((30, 39)),
        Some((44, 22)),
        Some((115, 115)),
        Some((12, 40)),
        Some((80, 46)),
        Some((2, 21)),
        Some((23, 66)),
        Some((3, 8)),
        Some((156, 9)),
        Some((49, 67)),
        Some((67, 18)),
        Some((31, 136)),
        Some((10, 11)),
        Some((64, 27)),
        Some((53, 13)),
        Some((106, 88)),
        Some((20, 76)),
        Some((90, 65)),
        Some((11, 23)),
        Some((7, 64)),
        Some((37, 26)),
        Some((2, 67)),
        Some((22, 182)),
        Some((25, 59)),
        Some((57, 15)),
        Some((16, 14)),
        Some((49, 33)),
        Some((8, 19)),
        Some((31, 89)),
        Some((10, 29)),
        Some((103, 22)),
        Some((9, 86)),
        Some((24, 21)),
        None,
    ]);
}