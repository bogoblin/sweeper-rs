use std::collections::HashMap;

pub struct BytePairEncoding {
    expanded_replacements: [Option<Vec<u8>>; 256],
    pairs: HashMap<(u8, u8), u8>,
}

impl BytePairEncoding {
    pub fn from_replacements(replacements: [Option<(u8, u8)>; 256]) -> Self {
        let mut pairs: HashMap<(u8, u8), u8> = Default::default();
        for (i, &r) in replacements.iter().enumerate() {
            match r {
                None => {}
                Some(pair) => {
                    pairs.insert(pair, i as u8);
                }
            }
        }
        
        let mut expanded_replacements: [Option<Vec<u8>>; 256] = [const { None }; 256];
        for i in 0..=255 {
            match replacements[i as usize] {
                None => {}
                Some((left, right)) => {
                    let mut full_replacement = vec![];
                    for byte in [left, right] {
                        match &expanded_replacements[byte as usize] {
                            None => {
                                full_replacement.push(byte);
                            }
                            Some(replacement_vec) => {
                                full_replacement.append(&mut replacement_vec.clone())
                            }
                        }
                    }
                    expanded_replacements[i as usize] = Some(full_replacement);
                }
            }
        }
        
        Self {
            expanded_replacements,
            pairs
        }
    }
    
    pub fn encode(&self, to_encode: &[u8]) -> Vec<u8> {
        let mut encoded = vec![];

        for &byte in to_encode {
            if let Some(last) = encoded.last_mut() {
                if let Some(&replace_with) = self.pairs.get(&(*last, byte)) {
                    *last = replace_with;
                    continue;
                }
            }
            encoded.push(byte);
        }

        encoded
    }
    
    fn decode_byte(&self, to_decode: u8) -> Vec<u8> {
        self.expanded_replacements[to_decode as usize].clone()
            .unwrap_or_else(|| vec![to_decode])
    }
    
    pub fn decode(&self, to_decode: &[u8]) -> Vec<u8> {
        let mut decoded = vec![];

        for &byte in to_decode {
            decoded.append(&mut self.decode_byte(byte));
        }
        
        decoded
    }
}