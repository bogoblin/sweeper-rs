use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::Position;

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    Connected,
    Click(Position),
    Flag(Position),
    DoubleClick(Position),
}

impl ClientMessage {
    pub fn decode(data: Value) -> Option<ClientMessage> {
        serde_json::from_value(data).ok()
    }
}