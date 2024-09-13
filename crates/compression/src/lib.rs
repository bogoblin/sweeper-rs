use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::js_sys::{Array, Uint8Array};
use huffman_derive::huffman_derive;
use crate::PublicTile::*;

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

impl Into<u8> for PublicTile {
    fn into(self) -> u8 {
        match self {
            Hidden => 0,
            Flag => 1<<4,
            Exploded => 1<<6 + 1<<5,
            Adjacent0 => 1<<6 + 0,
            Adjacent1 => 1<<6 + 1,
            Adjacent2 => 1<<6 + 2,
            Adjacent3 => 1<<6 + 3,
            Adjacent4 => 1<<6 + 4,
            Adjacent5 => 1<<6 + 5,
            Adjacent6 => 1<<6 + 6,
            Adjacent7 => 1<<6 + 7,
            Adjacent8 => 1<<6 + 8,
            Newline => 255,
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
                current_line.push(tile.into())
            }
        }
    }
    result
}
