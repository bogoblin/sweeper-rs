use world::Position;

#[derive(Debug)]
pub enum ClientMessage {
    Connected,
    Click(Position),
    Flag(Position),
    DoubleClick(Position),
}