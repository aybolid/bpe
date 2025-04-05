use std::collections::HashMap;

use crate::{Lonely, Pair, Token};

#[derive(Debug)]
pub struct PairMap {
    map: HashMap<u32, Token>,
}

impl PairMap {
    pub const ASCII_PRELUDE_SIZE: usize = 256;

    /// Creates a new `PairMap` with provided `HashMap`.
    ///
    /// # Arguments
    /// * `map` - The HashMap containing the token pairs.
    pub fn new(map: HashMap<u32, Token>) -> Self {
        Self { map }
    }

    /// Creates a new `PairMap` with an ASCII prelude tokens.
    pub fn with_ascii_prelude() -> Self {
        let mut pair_map = Self::new(HashMap::new());
        let prelude = gen_ascii_prelude();

        for token in prelude.into_iter() {
            pair_map.map.insert(token.0, token.as_token());
        }

        pair_map
    }

    pub fn learn(&mut self, corpus: &str, n_merges: u32) -> Vec<u32> {
        let mut tokenized_input: Vec<u32> = corpus.chars().map(|char| char as u32).collect();
        for token in &tokenized_input {
            if !self.map.contains_key(&token) {
                let lonely = Lonely::new(*token).as_token();
                self.map.insert(*token, lonely);
            }
        }

        let mut next_token_id = (self.map.len() + 1) as u32;

        for _ in 0..n_merges {
            let mut adjacent_pair_freq: HashMap<Pair, usize> = HashMap::new();

            for pair in tokenized_input.windows(2) {
                let token_pair = Pair::new(pair[0], pair[1]);

                adjacent_pair_freq
                    .entry(token_pair)
                    .and_modify(|freq| *freq += 1)
                    .or_insert(1);
            }

            let (most_freq_pair, pair_freq) = adjacent_pair_freq
                .into_iter()
                .max_by_key(|(_, freq)| *freq)
                .expect("seems impossible to me that this thing will be None :)");

            if pair_freq == 1 {
                break;
            }

            self.map.insert(next_token_id, most_freq_pair.as_token());

            let mut updated_tokens =
                Vec::with_capacity(tokenized_input.len().saturating_sub(pair_freq));

            let mut i = 0;
            while i + 1 < tokenized_input.len() {
                let left = tokenized_input[i];
                let right = tokenized_input[i + 1];
                if left == most_freq_pair.left && right == most_freq_pair.right {
                    updated_tokens.push(next_token_id);
                    i += 2;
                } else {
                    updated_tokens.push(left);
                    i += 1;
                }
            }

            if i < tokenized_input.len() {
                // if there’s one token left at the end, push it
                updated_tokens.push(tokenized_input[i]);
            }

            tokenized_input = updated_tokens;
            next_token_id += 1;
        }

        tokenized_input
    }
}

fn gen_ascii_prelude() -> Box<[Lonely; PairMap::ASCII_PRELUDE_SIZE]> {
    let mut prelude = Box::new([Lonely::new(0); PairMap::ASCII_PRELUDE_SIZE]);

    for (i, token) in prelude.iter_mut().enumerate().skip(1) {
        *token = Lonely::new(i as u32)
    }

    prelude
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn with_prelude() {
        let map = PairMap::with_ascii_prelude();
        assert_eq!(map.map.len(), PairMap::ASCII_PRELUDE_SIZE)
    }

    #[test]
    // https://en.wikipedia.org/wiki/Byte_pair_encoding#Example
    fn bpe_tokenize_aaabdaaabac() {
        let corpus = "aaabdaaabac";
        let mut map = PairMap::new(HashMap::new());

        let max_n_merges_possible = 3;
        // passing higher n_merges doesnt matter here as only 3 merges are possible
        let tokenized = map.learn(corpus, max_n_merges_possible + (68 + 1));

        let n_uniq_chars = corpus.chars().collect::<HashSet<_>>().len();

        assert_eq!(map.map.len(), n_uniq_chars + max_n_merges_possible as usize);

        //    aaabdaaabac ->
        // 1. ZabdZabac ->
        // 2. ZYdZYac ->
        // 3. XdXac (len 5)
        assert_eq!(tokenized.len(), 5)
    }
}
