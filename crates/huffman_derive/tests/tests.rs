use huffman_derive::huffman_derive;
use crate::HuffTest::*;

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
    let decoded = HuffTest::from_huffman_bytes(bytes);
    println!("{decoded:?}");
}
