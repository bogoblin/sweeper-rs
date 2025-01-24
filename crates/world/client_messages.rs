use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::{Position, Rect};

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    Connected,
    Click(Position),
    Flag(Position),
    DoubleClick(Position),
    Query(Rect),
    Disconnected(String),
}

impl ClientMessage {
    pub fn decode(data: Value) -> Option<ClientMessage> {
        serde_json::from_value(data).ok()
    }
}