use serde_json::Value;
use world::Position;
use crate::client_messages::ClientMessage::*;

#[derive(Debug)]
pub enum ClientMessage {
    Connected,
    Click(Position),
    Flag(Position),
    DoubleClick(Position),
}

impl ClientMessage {
    pub(crate) fn decode(data: Value) -> Option<ClientMessage> {
        if let Value::Array(array) = data {
            match &array[..] {
                [Value::String(message_type), Value::Number(x), Value::Number(y)] => {
                    let x = x.as_f64()?.floor() as i32;
                    let y = y.as_f64()?.floor() as i32;
                    let position = Position(x, y);
                    match message_type.as_str() {
                        "click" => Some(Click(position)),
                        "flag" => Some(Flag(position)),
                        "doubleClick" => Some(DoubleClick(position)),
                        _ => None,
                    }
                },
                _ => None
            }
        } else {
            None
        }
    }
}