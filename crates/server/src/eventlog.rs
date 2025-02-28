use serde_with::base64::Standard;
use serde_with::base64::Base64;
use serde_with::formats::Unpadded;
use serde_with::serde_as;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::io;
use world::{ChunkMines, ChunkPosition, Position};

#[serde_as]
#[derive(Serialize, Deserialize)]
pub enum SourcedEvent {
    Click(Position),
    DoubleClick(Position),
    Flag(Position),
    Unflag(Position),
    ChunkGenerated(
        ChunkPosition, 
        #[serde_as(as = "Base64<Standard, Unpadded>")]
        ChunkMines
    )
}

pub struct EventLogWriter {
    file: BufWriter<File>,
}

impl EventLogWriter {
    pub async fn new(file_path: PathBuf) -> io::Result<Self> {
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(file_path).await?;
        Ok(Self { file: BufWriter::new(file) })
    }
    
    pub async fn write(&mut self, event: SourcedEvent) -> io::Result<()> {
        let mut json = serde_json::to_string(&event)?;
        json.push('\n');
        self.file.write_all(json.as_bytes()).await?;
        Ok(())
    }
    
    pub async fn flush(&mut self) -> io::Result<()> {
        self.file.flush().await
    }
}

pub struct EventLogReader {
    file: BufReader<File>
}

impl EventLogReader {
    pub async fn open(file_path: PathBuf) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .open(file_path).await?;
        Ok(Self { file: BufReader::new(file) })
    }

    // TODO: would like to handle EOFs and invalid events differently
    pub async fn read(&mut self) -> Option<SourcedEvent> {
        let mut line = String::new();
        self.file.read_line(&mut line).await.unwrap();
        match serde_json::from_str(&line) {
            Ok(event) => Some(event),
            Err(_) => None
        }
    }
}