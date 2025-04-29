use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::OpenOptions;
use std::io::Read;
use world::{PublicTile, Tile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chunk_file = OpenOptions::new()
        .read(true)
        .open("chunks")?;

    let mut bytes: Vec<u8> = chunk_file.bytes().map_while(|byte| {
        match byte {
            Ok(byte) => Some(byte),
            Err(_) => None
        }
    }).collect();
    let original_bytes = bytes.clone();
    
    let mut counts = PairCounts::from_bytes(&bytes[..]);
    
    let mut replacements = vec![];
    loop {
        let replace_with = counts.find_next_unused_byte();
        if replace_with == 255 { break }
        
        // println!("{counts}");
        let (best_pair, count) = counts.best_pair();
        let (a, b) = best_pair;
        println!("Replacing ({a}, {b}) with {replace_with}");
        replacements.push(PairEncoding {
            pair: best_pair,
            replace_with,
            count,
        });
        bytes = counts.replace(bytes, best_pair, replace_with);
        
        // MaybeTODO: speed up the implementation by avoiding this step
        counts = PairCounts::from_bytes(&bytes[..]);
    }

    for x in replacements {
        let (a, b) = x.pair;
        let r = x.replace_with;
        println!("({r}, ({a}, {b}))");
    }

    // let mut check_bytes = vec![];
    // for byte in bytes {
    //     check_bytes.append(&mut lookup_table[byte as usize].clone().unwrap_or_else(|| vec![byte]));
    // }
    
    // assert_eq!(check_bytes, original_bytes);
    
    Ok(())
}

#[derive(Eq, PartialEq)]
struct PairEncoding {
    pair: (u8, u8),
    replace_with: u8,
    count: usize,
}

impl PartialOrd for PairEncoding {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for PairEncoding {
    fn cmp(&self, other: &Self) -> Ordering {
        self.count.partial_cmp(&other.count)
            .unwrap_or_else(|| self.replace_with.cmp(&other.replace_with))
    }
}

struct PairCounts {
    pairs: HashMap<(u8, u8), usize>,
    used_bytes: [bool; 256],
}

impl PairCounts {
    fn from_bytes(bytes: &[u8]) -> Self {
        let mut result = Self {
            pairs: Default::default(),
            used_bytes: [false; 256],
        };
        for i in 0..=255 {
            // We always want to be able to represent any PublicTile, so never replace those even if
            // there aren't any of them in the training data:
            result.used_bytes[Tile::from(PublicTile::from(Tile(i))).0 as usize] = true;
        }
        
        let mut last_byte = None;
        for &byte in bytes {
            result.used_bytes[byte as usize] = true;
            match last_byte {
                Some(last_byte) => {
                    result.add_pair(last_byte, byte);
                },
                None => {}
            }
            last_byte = Some(byte);
        }
        
        result
    }

    fn find_next_unused_byte(&self) -> u8 {
        for i in 0..=255 {
            if !self.used_bytes[i] {
                return i as u8;
            }
        }
        u8::MAX
    }

    fn add_pair(&mut self, a: u8, b: u8) {
        if a == 255 || b == 255 { return; }
        *self.pairs.entry((a, b)).or_insert(0) += 1;
    }
    
    fn remove_pair(&mut self, a: u8, b: u8) {
        if a == 255 || b == 255 { return; }
        *self.pairs.entry((a, b)).or_insert(0) -= 1;
    }
    
    fn best_pair(&self) -> ((u8, u8), usize) {
        let mut best = None;

        for (&pair, &count) in &self.pairs {
            match best {
                None => {
                    best = Some((pair, count))
                }
                Some((_, count_to_beat)) => {
                    if count > count_to_beat {
                        best = Some((pair, count));
                    }
                }
            }
        }
        
        best.unwrap()
    }

    fn replace(&mut self, bytes: Vec<u8>, pair: (u8, u8), replace_with: u8) -> Vec<u8> {
        let mut result = Vec::with_capacity(bytes.len());
        
        let mut skip_next = false;
        for right_index in 1..bytes.len() {
            if skip_next { 
                skip_next = false;
                continue;
            }
            
            let a = bytes[right_index-1];
            let b = bytes[right_index];
            if (a, b) == pair {
                skip_next = true;
                result.push(replace_with);
                // self.remove_pair(a, b);

                if right_index >= 2 {
                    if let Some(&before) = result.last() {
                        // self.remove_pair(before, a);
                        self.add_pair(before, replace_with);
                    }
                }   
                if let Some(&after) = bytes.get(right_index + 1) {
                    // self.remove_pair(b, after);
                    self.add_pair(replace_with, after);
                }
            } else {
                result.push(a);
            }
        }
        if !skip_next {
            result.push(*bytes.last().unwrap())
        }
        
        result
    }
}

impl Display for PairCounts {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut sorted = self.pairs.iter().collect::<Vec<_>>();
        sorted.sort_by(|(_, a_count), (_, b_count)| {
            b_count.cmp(a_count)
        });
        for ((a, b), count) in sorted {
            writeln!(f, "({a}, {b}) => {count}")?;
        }
        Ok(())
    }
}