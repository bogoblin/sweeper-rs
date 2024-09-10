pub struct BitReader {
    bytes: Vec<u8>,
    byte_index: usize,
    offset_within_byte: u8, // Should only be 0-7
}

impl BitReader {
    pub fn from(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            byte_index: 0,
            offset_within_byte: 0
        }
    }

    pub fn read_byte(&mut self) -> Option<bool> {
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

pub struct BitWriter {
    bytes: Vec<u8>,
    offset_within_byte: u8, // Should only be 0-7
}

impl BitWriter {
    pub fn new() -> Self {
        Self {
            bytes: vec![],
            offset_within_byte: 0,
        }
    }

    pub fn write_bit(&mut self, bit: bool) {
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
    
    pub fn to_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

pub trait HuffmanCode {
    fn encode(&self, encode_to: &mut BitWriter);
    fn decode(decode_from: &mut BitReader) -> Option<Box<Self>>;
}