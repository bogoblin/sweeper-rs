use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::{Position, Rect};

#[derive(Debug, Eq, PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    Connected,
    Click(Position),
    Flag(Position),
    DoubleClick(Position),
    Query(Rect),
}

impl ClientMessage {
    pub fn decode(data: Value) -> Option<ClientMessage> {
        serde_json::from_value(data).ok()
    }
}