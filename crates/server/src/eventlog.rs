use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use world::events::Event;
use world::World;

pub struct EventLog {
    
}

impl EventLog {
    pub fn open(file_path: PathBuf) -> World {
        match fs::read(file_path) {
            Ok(event_log) => {
                // Event::from_compressed()
                todo!()
            },
            Err(err) => {
                World::new()
            }
        }
    }
}