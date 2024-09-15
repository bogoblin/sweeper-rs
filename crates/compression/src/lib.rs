use crate::PublicTile::*;
use huffman_derive::huffman_derive;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::js_sys::{Array, Uint8Array};
use world::Tile;

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
enum PublicTile {
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

impl Into<PublicTile> for Tile {
    fn into(self) -> PublicTile {
        if self.is_revealed() {
            if self.is_mine() {
                Exploded
            } else {
                match self.adjacent() { 
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
            if self.is_flag() {
                Flag
            } else {
                Hidden
            }
        }
    }
}

impl Into<Tile> for PublicTile {
    fn into(self) -> Tile {
        match self {
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

#[wasm_bindgen]
pub fn decompress(compressed: Uint8Array) -> Array {
    let tiles = PublicTile::from_huffman_bytes(compressed.to_vec());
    let result = Array::new();
    let mut current_line: Vec<u8> = vec![];
    for tile in tiles {
        match *tile {
            Newline => {
                result.push(Uint8Array::from(&current_line[..]).as_ref());
                current_line = vec![];
            },
            tile => {
                let tile: Tile = tile.into();
                current_line.push(tile.into());
            }
        }
    }
    result
}
