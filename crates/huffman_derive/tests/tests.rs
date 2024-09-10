use huffman_derive::huffman_derive;
use crate::HuffTest::*;
use huffman::HuffmanCode;

#[huffman_derive(
    One => 1,
    Two => 0.5f64,
    Three => 0.25f64,
    Four => 0.125f64
)]
#[derive(Debug)]
enum HuffTest {
    One, Two, Three, Four
}

#[test]
fn test() {
    let example = vec![One, Four, Three, Two, Four];
    let mut bits = BitWriter::new();
    for i in example {
        i.encode(&mut bits);
    }
    let bytes = bits.to_bytes();
    println!("{bytes:?}");
    let mut reader = BitReader::from(bytes);
    let mut decoded = vec![];
    loop {
        if let Some(thing) = HuffTest::decode(&mut reader) {
            decoded.push(*thing)
        } else {
            break;
        }
    }
    println!("{decoded:?}");
}
