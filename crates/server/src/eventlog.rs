use serde_with::base64::Standard;
use serde_with::base64::Base64;
use serde_with::formats::Unpadded;
use serde_with::serde_as;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use futures_util::{Stream, StreamExt};
use log::*;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::io;
use tokio_util::codec::{FramedRead, LinesCodec};
use world::{ChunkMines, ChunkPosition, Position};
use world::Event;

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

impl SourcedEvent {
    pub(crate) fn from_event(event: &Event) -> SourcedEvent {
        match event {
            Event::Clicked { at, .. } => {
                SourcedEvent::Click(*at)
            }
            Event::DoubleClicked { at, .. } => {
                SourcedEvent::DoubleClick(*at)
            }
            Event::Flag { at, .. } => {
                SourcedEvent::Flag(*at)
            }
            Event::Unflag { at, .. } => {
                SourcedEvent::Unflag(*at)
            }
        }
    }
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
    pub reader: FramedRead<File, LinesCodec>,
}

impl EventLogReader {
    pub async fn open(file_path: PathBuf) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .open(file_path).await?;
        let reader = FramedRead::new(file, LinesCodec::new());
        Ok(Self { reader })
    }
    
    pub fn events(self) -> impl Stream<Item = EventReadResult> {
        self.reader.map(|line| EventReadResult::parse(line.ok()))
    }
}

pub enum EventReadResult {
    Ok(SourcedEvent),
    Invalid(String),
    Eof,
}

impl EventReadResult {
    pub fn parse(line: Option<String>) -> Self {
        match line {
            None => { EventReadResult::Eof }
            Some(line) => {
                match serde_json::from_str(&line) {
                    Ok(event) => {
                        trace!("parse");
                        EventReadResult::Ok(event)
                    },
                    Err(_) => EventReadResult::Invalid(line),
                }
            }
        }
    }
}