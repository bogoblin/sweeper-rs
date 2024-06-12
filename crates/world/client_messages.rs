use crate::Position;

pub enum ClientMessage {
    Welcome,
    Click(Position),
    Flag(Position),
    DoubleClick(Position),
}