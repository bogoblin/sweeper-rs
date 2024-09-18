use std::{fs, io};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use derive_more::{Display, Error};
use world::World;

pub struct Backup {
    file: PathBuf,
    last_saved: Option<Instant>
}

#[derive(Error, Debug, Display)]
pub enum BackupError {
    PostcardError(postcard::Error),
    IoError(io::Error)
}

impl Backup {
    pub fn new(file: PathBuf) -> Self {
        Self {
            file,
            last_saved: None
        }
    }

    pub fn load(&self) -> Result<World, BackupError> {
        match fs::read(self.file.as_path()) {
            Ok(saved_world) => {
                match postcard::from_bytes::<World>(saved_world.as_slice()) {
                    Ok(world) => Ok(world),
                    Err(err) => Err(BackupError::PostcardError(err))
                }
            },
            Err(err) => Err(BackupError::IoError(err))
        }
    }

    pub fn save(&mut self, world: &World) -> Result<usize, BackupError> {
        let now = Instant::now();
        let do_backup = match self.last_saved {
            None => true,
            Some(backup_time) =>
                now - backup_time > Duration::from_secs(5)
        };
        if do_backup {
            self.last_saved = Some(now);
            match postcard::to_allocvec(&world) {
                Ok(serialized) => {
                    let num_bytes = serialized.len();
                    if let Err(err) = fs::write(self.file.as_path(), serialized) {
                        Err(BackupError::IoError(err))
                    } else {
                        Ok(num_bytes)
                    }
                },
                Err(err) => Err(BackupError::PostcardError(err))
            }
        } else {
            Ok(0)
        }
    }
    
    pub fn location(&self) -> &Path {
        self.file.as_path()
    }
}