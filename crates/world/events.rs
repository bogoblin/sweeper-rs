use crate::{ChunkMines, ChunkPosition, Position, Tile, UpdatedRect, UpdatedTile};
use serde::{Deserialize, Serialize};
use std::i32;

#[derive(Debug, Clone)]
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
    },
    GeneratedChunk {
        position: ChunkPosition,
        mines: ChunkMines,
    }
}

impl Event {
    pub fn tiles_updated(&self) -> Vec<UpdatedTile> {
        match self {
            Event::Clicked { updated, .. } |
            Event::DoubleClicked { updated, .. } => {
                updated.tiles_updated()
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
            Event::GeneratedChunk { .. } => { vec![] }
        }
    }
    
    pub fn player_id(&self) -> String {
        match self {
            Event::Clicked { player_id, .. } |
            Event::DoubleClicked { player_id, .. } |
            Event::Flag { player_id, .. } |
            Event::Unflag { player_id, .. } => {
                player_id.clone()
            }
            Event::GeneratedChunk { .. } => { "".to_string() }
        }
    }
    
    pub fn should_send(&self) -> bool {
        match self {
            Event::GeneratedChunk { .. } => false,
            _ => true
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
            Event::GeneratedChunk { position, .. } => {
                ("Q", &"".to_string(), &position.position(), None)
            }
        };

        binary.append(&mut header.as_bytes().to_vec());
        binary.append(&mut player_id.as_bytes().to_vec());
        binary.push(0);
        binary.append(&mut at.0.to_be_bytes().to_vec());
        binary.append(&mut at.1.to_be_bytes().to_vec());
        if let Some(updated) = updated {
            binary.append(&mut updated.into());
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
            match UpdatedRect::from_compressed(&compressed[index..]) {
                None => None,
                Some(updated) => {
                    Some(Event::Clicked {
                        player_id,
                        at,
                        updated
                    })
                }
            }
        } else if header == "f" {
            Some(Event::Flag {
                player_id,
                at
            })
        } else if header == "u" {
            Some(Event::Unflag {
                player_id,
                at
            })
        } else {
            None
        }
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