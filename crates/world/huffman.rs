use crate::huffman::PublicTile::*;
use crate::Tile;

enum PublicTile {
    Hidden,
    Flag,
    Exploded,
    Revealed0,
    Revealed1,
    Revealed2,
    Revealed3,
    Revealed4,
    Revealed5,
    Revealed6,
    Revealed7,
    Revealed8,
}

impl PublicTile {
    fn from(tile: &Tile) -> Self {
        if tile.is_revealed() {
            if tile.is_mine() {
                return Exploded;
            } else {
                match tile.adjacent() {
                    1 => Revealed1,
                    2 => Revealed2,
                    3 => Revealed3,
                    4 => Revealed4,
                    5 => Revealed5,
                    6 => Revealed6,
                    7 => Revealed7,
                    8 => Revealed8,
                    _ => Revealed0, // Zero
                }
            }
        } else {
            return if tile.is_flag() {
                Flag
            } else {
                Hidden
            }
        }
    }

    fn from_bits(bits: &mut BitReader) -> Option<Self> {
        return if bits.read_byte()? == false { // 0
            Some(Hidden)
        } else {
            if bits.read_byte()? == false { // 10
                if bits.read_byte()? == false { // 100
                    Some(Revealed2)
                } else { // 101
                    Some(Revealed0)
                }
            } else { // 11
                if bits.read_byte()? == false { // 110
                    Some(Revealed1)
                } else { // 111
                    if bits.read_byte()? == false { // 1110
                        if bits.read_byte()? == false { // 11100
                            Some(Exploded)
                        } else { // 11101
                            if bits.read_byte()? == false { // 111010
                                if bits.read_byte()? == false { // 1110100
                                    if bits.read_byte()? == false { // 11101000
                                        if bits.read_byte()? == false { // 111010000
                                            Some(Revealed8)
                                        } else { // 111010001
                                            Some(Revealed7)
                                        }
                                    } else { // 11101001
                                        Some(Revealed6)
                                    }
                                } else { // 1110101
                                    Some(Revealed5)
                                }
                            } else { // 111011
                                Some(Revealed4)
                            }
                        }
                    } else { // 1111
                        return if bits.read_byte()? == false { // 11110
                            Some(Revealed3)
                        } else { // 11111
                            Some(Flag)
                        }
                    }
                }
            }
        }
    }

    fn as_bits(&self) -> Vec<bool> {
        match self {
            Hidden => string_to_bits("0"),
            Flag => string_to_bits("11111"),
            Exploded => string_to_bits("11100"),
            Revealed0 => string_to_bits("101"),
            Revealed1 => string_to_bits("110"),
            Revealed2 => string_to_bits("100"),
            Revealed3 => string_to_bits("11110"),
            Revealed4 => string_to_bits("111011"),
            Revealed5 => string_to_bits("1110101"),
            Revealed6 => string_to_bits("11101001"),
            Revealed7 => string_to_bits("111010001"),
            Revealed8 => string_to_bits("111010000"),
        }
    }

    fn write_to_bits(&self, bits: &mut BitWriter) {
        for bit in self.as_bits() {
            bits.write_bit(bit);
        }
    }
}

struct BitReader {
    bytes: Vec<u8>,
    byte_index: usize,
    offset_within_byte: u8, // Should only be 0-7
}

impl BitReader {
    fn from(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            byte_index: 0,
            offset_within_byte: 0
        }
    }

    fn read_byte(&mut self) -> Option<bool> {
        if self.offset_within_byte == 8 {
            self.offset_within_byte = 0;
            self.byte_index += 1;
        }
        let byte = self.bytes.get(self.byte_index)?;
        let result = Some(byte & (1 << self.offset_within_byte) != 0);
        self.offset_within_byte += 1;
        result
    }
}

struct BitWriter {
    bytes: Vec<u8>,
    offset_within_byte: u8, // Should only be 0-7
}

impl BitWriter {
    fn new() -> Self {
        Self {
            bytes: vec![],
            offset_within_byte: 0,
        }
    }

    fn write_bit(&mut self, bit: bool) {
        if self.offset_within_byte == 0 {
            self.bytes.push(0);
        }
        if bit {
            let mut byte = self.bytes.pop().unwrap();
            byte |= 1 << self.offset_within_byte;
            self.bytes.push(byte);
        }
        self.offset_within_byte += 1;
        if self.offset_within_byte == 8 {
            self.offset_within_byte = 0;
        }
    }
}


fn string_to_bits(string: &str) -> Vec<bool> {
    string.chars().map(|char| {
        match char {
            '0' => false,
            '1' => true,
            _ => panic!("String must be 0 and 1 only")
        }
    }).collect()
}

// For sending to client - hides information about hidden tiles:
fn encode_tiles(tiles: Vec<Tile>) -> Vec<u8> {
    let mut bits = BitWriter::new();
    for tile in tiles {
        PublicTile::from(&tile).write_to_bits(&mut bits);
    }
    bits.bytes
}