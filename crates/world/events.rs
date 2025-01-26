use std::i32;
use serde::{Deserialize, Serialize};
use huffman::{BitWriter, HuffmanCode};
use crate::{Position, Tile, UpdatedRect, UpdatedTile};
use crate::compression::PublicTile;

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub enum Event {
    Clicked {
        player_id: String,
        at: Position,
        updated: UpdatedRect,
    },
    DoubleClicked {
        player_id: String,
        at: Position,
        updated: UpdatedRect,
    },
    Flag {
        player_id: String,
        at: Position,
    },
    Unflag {
        player_id: String,
        at: Position,
    }
}

impl Event {
    pub fn tiles_updated(&self) -> Vec<UpdatedTile> {
        match self {
            Event::Clicked { updated, .. } |
            Event::DoubleClicked { updated, .. } => {
                let mut result = vec![];
                for (x, col) in updated.updated.iter().enumerate() {
                    for (y, tile) in col.iter().enumerate() {
                        if *tile == Tile::empty() {
                            continue
                        }
                        let position = updated.top_left + Position(x as i32, y as i32);
                        result.push(UpdatedTile {
                            position,
                            tile: tile.clone()
                        });
                    }
                }
                result
            }
            Event::Flag { at, .. } => {
                vec![UpdatedTile {
                    position: at.clone(),
                    tile: Tile::empty().with_flag()
                }]
            }
            Event::Unflag { at, .. } => {
                vec![UpdatedTile {
                    position: at.clone(),
                    tile: Tile::empty()
                }]
            }
        }
    }
}
impl Event {
    pub fn compress(&self) -> Vec<u8> {
        let mut binary = vec![];
        let (header, player_id, at, updated) = match self {
            Event::Clicked { player_id, at, updated } |
            Event::DoubleClicked { player_id, at, updated } => {
                ("c", player_id, at, Some(updated))
            }
            Event::Flag { player_id, at } => {
                ("f", player_id, at, None)
            }
            Event::Unflag { player_id, at } => {
                ("u", player_id, at, None)
            }
        };

        binary.append(&mut header.as_bytes().to_vec());
        binary.append(&mut player_id.as_bytes().to_vec());
        binary.push(0);
        binary.append(&mut at.0.to_be_bytes().to_vec());
        binary.append(&mut at.1.to_be_bytes().to_vec());
        if let Some(updated) = updated {
            let Position(x, y) = updated.top_left;
            binary.append(&mut x.to_be_bytes().to_vec());
            binary.append(&mut y.to_be_bytes().to_vec());
            let mut bw = BitWriter::new();
            let public_tiles = updated.public_tiles();
            for tile in public_tiles {
                tile.encode(&mut bw);
            }
            binary.append(&mut bw.to_bytes());
        }
        
        binary
    }
    
    pub fn from_compressed(compressed: Vec<u8>) -> Option<Event> {
        let header = String::from_utf8_lossy(&compressed[0..=0]);
        let mut index = 1;
        loop {
            let char = compressed.get(index)?;
            index += 1;
            if *char == 0 {
                break
            }
        }
        let player_id = String::from_utf8_lossy(&compressed[1..index-1]).to_string();
        let at = {
            let x = i32::from_be_bytes(*compressed[index..].first_chunk()?);
            let y = i32::from_be_bytes(*compressed[index+4..].first_chunk()?);
            index += 8;
            Position(x, y)
        };
        
        if header == "c" {
            let top_left = {
                let x = i32::from_be_bytes(*compressed[index..].first_chunk()?);
                let y = i32::from_be_bytes(*compressed[index+4..].first_chunk()?);
                index += 8;
                Position(x, y)
            };
            let mut updated = UpdatedRect::empty();
            updated.top_left = top_left;

            let tiles = PublicTile::from_huffman_bytes(compressed[index..].to_vec());
            
            let mut current_line: Vec<Tile> = vec![];
            for tile in tiles {
                match *tile {
                    PublicTile::Newline => {
                        updated.updated.push(current_line);
                        current_line = vec![];
                    },
                    tile => {
                        let tile: Tile = tile.into();
                        current_line.push(tile.into());
                    }
                }
            }
            
            return Some(Event::Clicked {
                player_id,
                at,
                updated
            });
        } else if header == "f" {
            return Some(Event::Flag {
                player_id,
                at
            });
        } else if header == "u" {
            return Some(Event::Unflag {
                player_id,
                at
            });
        }
        
        None
    }
}


#[test]
fn compression_test() -> Result<(), String> {
    let p = "Gary";
    let pos = Position(-50, 300);
    let flag = Event::Flag {
        player_id: p.to_string(),
        at: pos
    };
    let decompressed = Event::from_compressed(flag.compress()).ok_or("uh oh")?;
    match decompressed {
        Event::Flag { player_id, at } => {
            assert_eq!(player_id, p);
            assert_eq!(at, pos);
        }
        _ => panic!("Wrong event type")
    }

    let to_update = vec![
        UpdatedTile { position: Position(1, 1), tile: Tile::empty().with_revealed() },
        UpdatedTile { position: Position(3, 1), tile: Tile::empty().with_revealed() },
        UpdatedTile { position: Position(3, 3), tile: Tile::mine().with_revealed() },
        UpdatedTile { position: Position(2, 3), tile: Tile::mine().with_flag() },
    ];
    let clicked = Event::Clicked {
        player_id: p.to_string(),
        at: pos,
        updated: UpdatedRect::new(to_update.clone())
    };
    let decompressed = Event::from_compressed(clicked.compress()).ok_or("uh oh")?;
    match decompressed {
        Event::Clicked { player_id, at, updated } => {
            assert_eq!(player_id, p);
            assert_eq!(at, pos);
            assert_eq!(updated.public_tiles(), UpdatedRect::new(to_update).public_tiles());
        }
        _ => panic!("Wrong event type")
    }

    Ok(())
}