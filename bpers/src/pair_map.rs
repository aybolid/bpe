use std::collections::HashMap;

pub const ASCII_PRELUDE_SIZE: usize = 256;

use crate::TokenPair;

#[derive(Debug)]
pub struct PairMap {
    map: HashMap<u32, TokenPair>,
}

impl PairMap {
    pub fn new(map: HashMap<u32, TokenPair>) -> Self {
        Self { map }
    }

    pub fn with_ascii_prelude() -> Self {
        let mut pair_map = Self::new(HashMap::new());
        let prelude = gen_ascii_prelude();

        for pair in prelude.into_iter() {
            pair_map.insert_pair(pair.left, pair);
        }

        pair_map
    }

    pub fn insert_pair(&mut self, key: u32, pair: TokenPair) -> Option<TokenPair> {
        self.map.insert(key, pair)
    }

    pub fn learn(&mut self, corpus: &[&str]) {
        println!("Learning started...");
    }
}

fn gen_ascii_prelude() -> Box<[TokenPair; ASCII_PRELUDE_SIZE]> {
    let mut prelude = Box::new([TokenPair::from_ascii(0); ASCII_PRELUDE_SIZE]);

    for (i, pair) in prelude.iter_mut().enumerate().skip(1) {
        *pair = TokenPair::from_ascii(i as u8)
    }

    prelude
}
