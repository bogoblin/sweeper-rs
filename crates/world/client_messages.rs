use crate::Position;

pub enum ClientMessage {
    Click(Position),
    Flag(Position),
    DoubleClick(Position),
}