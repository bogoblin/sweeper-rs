use crate::ServerMessage;
use std::usize;

pub struct ServerMessageBundle(Vec<ServerMessage>);

struct MessageLength(usize);

impl MessageLength {
    fn to_bytes(&self) -> Vec<u8> {
        if self.0 == 0 {
            return vec![0];
        }
        let mut length_remaining = self.0;
        let mut result = vec![];
        result.push((length_remaining % 128) as u8);
        length_remaining /= 128;
        while length_remaining > 0 {
            result.push((length_remaining % 128 + 128) as u8);
            length_remaining /= 128;
        }
        
        result.reverse();
        
        result
    }
}

impl MessageLength {
    pub fn read_from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), ()> {
        let mut length: usize = 0;
        let mut slice_end = 0;
        for &byte in bytes {
            slice_end += 1;
            length *= 128;
            length += (byte % 128) as usize;
            if byte < 128 {
                break
            }
        }
        Ok((Self(length), &bytes[0..slice_end]))
    }
}

impl From<Vec<ServerMessage>> for ServerMessageBundle {
    fn from(value: Vec<ServerMessage>) -> Self {
        Self(value)
    }
}

impl ServerMessageBundle {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = vec![b'b'];
        for message in &self.0 {
            let mut serialized: Vec<u8> = message.into();

            let length = MessageLength(serialized.len());
            result.append(&mut length.to_bytes());

            result.append(&mut serialized);
        }
        result
    }
}

impl ServerMessageBundle {
    pub fn from_compressed(value: &[u8]) -> Result<Self, ()> {
        if let Some(header) = value.get(0) {
            if *header != b'b' { return Err(()); }
            let mut result = vec![];
            let mut read_position = 1;
            while read_position < value.len() {
                let (MessageLength(length), length_slice) = MessageLength::read_from_bytes(&value[read_position..])?;
                read_position += length_slice.len();
                let end = read_position + length;
                if end > value.len() {
                    return Err(());
                }
                let message_bytes = &value[read_position..end];
                read_position = end;
                let message = ServerMessage::from_compressed(message_bytes).map_err(|_| ())?;
                result.push(message);
            }
            Ok(Self(result))
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use quickcheck_macros::quickcheck;
    use crate::updates::server_message_bundle::{MessageLength, ServerMessageBundle};
    use crate::{Chunk, ChunkPosition, ChunkTiles, ServerMessage};

    #[quickcheck]
    fn encode_decode_message_length(length: usize) {
        let encoded = MessageLength(length).to_bytes();
        let (decoded_length, slice) = MessageLength::read_from_bytes(&encoded).unwrap();
        assert_eq!(decoded_length.0, length);
        assert_eq!(slice, &encoded);
    }

    #[test]
    fn bundle_and_compress_one_chunk() {
        let message = ServerMessage::Chunk(Chunk::from_position_and_tiles(ChunkPosition::new(0, 0), ChunkTiles::default()));
        let compressed: Vec<u8> = ServerMessageBundle(vec![message.clone()]).to_bytes();
        let decompressed = ServerMessageBundle::from_compressed(&compressed).unwrap();
        assert_eq!(decompressed.0.len(), 1);
        assert_eq!(decompressed.0[0], message);
    }

    #[quickcheck]
    fn bundling_and_compression(messages: Vec<ServerMessage>) {
        let compressed: Vec<u8> = ServerMessageBundle(messages.clone()).to_bytes();
        if let Ok(ServerMessageBundle(decompressed)) = ServerMessageBundle::from_compressed(&compressed[..]) {
            for i in 0..messages.len() {
                assert_eq!(messages[i], decompressed[i]);
            }
        }
    }
}