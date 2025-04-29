pub struct BytePairEncoding {
    sorted_replacements: Vec<(u8, (u8, u8))>,
    expanded_replacements: [Option<Vec<u8>>; 256],
}

impl BytePairEncoding {
    pub fn from_replacements(sorted_replacements: Vec<(u8, (u8, u8))>) -> Self {
        let mut replacements: [Option<(u8, u8)>; 256] = [const { None }; 256];
        
        for &(replace_with, pair) in &sorted_replacements {
            replacements[replace_with as usize] = Some(pair);
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
            sorted_replacements,
            expanded_replacements,
        }
    }
    
    pub fn encode(&self, to_encode: &[u8]) -> Vec<u8> {
        // MaybeTODO: Do this without copying
        let mut to_replace = Vec::from(to_encode);
        let mut replaced = Vec::with_capacity(to_replace.len());

        for &(replace_with, pair) in &self.sorted_replacements {
            let mut skip_next = false;
            for right_index in 1..to_replace.len() {
                if skip_next {
                    skip_next = false;
                    continue;
                }
                let peek_pair = (to_replace[right_index-1], to_replace[right_index]);
                if peek_pair == pair {
                    replaced.push(replace_with);
                    skip_next = true;
                } else {
                    replaced.push(peek_pair.0);
                }
            }
            if !skip_next {
                replaced.push(*to_replace.last().unwrap());
            }
            std::mem::swap(&mut to_replace, &mut replaced);
            replaced.clear();
        }
        
        to_replace
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