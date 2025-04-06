use std::time::Instant;

use foldhash::{HashMap, HashMapExt};
use indexmap::IndexMap;

use crate::{Lonely, Pair, Token};

type FoldIndexMap<K, V> = IndexMap<K, V, foldhash::fast::FixedState>;

#[derive(Debug)]
pub struct Vocabulary {
    /// A recursive map that represents learned vocabulary.
    pub id_to_token: HashMap<u32, Token>,
    pub token_pair_to_id: HashMap<Pair, u32>,
    next_token_id: u32,
}

impl Vocabulary {
    /// Creates a new `Vocabulary`.
    pub fn new() -> Self {
        Self {
            id_to_token: HashMap::new(),
            token_pair_to_id: HashMap::new(),
            next_token_id: 0,
        }
    }

    /// Learns vocabulary from a given corpus.
    ///
    /// # Arguments
    /// * `corpus` - The input text corpus.
    /// * `n_merges` - The max number of merges to perform.
    ///
    /// # Returns
    /// An artifact of the learning process. Basically, it returns a byte pair encoded `corpus`.
    pub fn learn(&mut self, corpus: &str, n_merges: u32) -> Vec<u32> {
        let mut max_char = 0;
        let mut tokens: Vec<u32> = corpus
            .chars()
            .map(|char| {
                let char_u32 = char as u32;
                if char_u32 > max_char {
                    max_char = char_u32;
                }
                char_u32
            })
            .collect();
        if self.next_token_id == 0 {
            self.next_token_id = max_char + 1;
        }

        for token in &tokens {
            if !self.id_to_token.contains_key(token) {
                let lonely = Lonely::new(*token).as_token();
                self.id_to_token.insert(*token, lonely);
            }
        }

        for n_merge in 0..n_merges {
            let start_time = Instant::now();

            // index map for deterministic ordering
            let mut adjacent_pair_freq: FoldIndexMap<Pair, usize> = FoldIndexMap::default();
            for window in tokens.windows(2) {
                let token_pair = Pair::new(window[0], window[1]);
                *adjacent_pair_freq.entry(token_pair).or_insert(0) += 1;
            }

            match adjacent_pair_freq.into_iter().max_by_key(|(_, freq)| *freq) {
                Some((most_freq_pair, pair_freq)) if pair_freq > 1 => {
                    self.id_to_token
                        .insert(self.next_token_id, most_freq_pair.as_token());
                    self.token_pair_to_id
                        .insert(most_freq_pair, self.next_token_id);

                    let mut updated_tokens = Vec::with_capacity(tokens.len());
                    let mut i = 0;
                    while i < tokens.len() {
                        if i + 1 < tokens.len()
                            && tokens[i] == most_freq_pair.left
                            && tokens[i + 1] == most_freq_pair.right
                        {
                            updated_tokens.push(self.next_token_id);
                            i += 2;
                        } else {
                            updated_tokens.push(tokens[i]);
                            i += 1;
                        }
                    }

                    tokens = updated_tokens;
                    self.next_token_id += 1;

                    if (n_merge + 1) % 10 == 0 {
                        println!("Merge #{}", n_merge + 1);
                        println!("\tMerge took:           {:?}", start_time.elapsed());
                        println!("\tTokenized input size: {}", tokens.len());
                        println!("\tVocabulary size:      {}", self.id_to_token.len());
                    }
                }
                _ => {
                    println!("No pairs with frequency > 1 after {n_merge} merges, stop learning\n");
                    break;
                }
            }
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    // https://en.wikipedia.org/wiki/Byte_pair_encoding#Example
    fn bpe_tokenize_aaabdaaabac() {
        let corpus = "aaabdaaabac";
        let mut vocabulary = Vocabulary::new();

        let max_n_merges_possible = 3;
        // passing higher n_merges doesnt matter here as only 3 merges are possible
        let tokenized = vocabulary.learn(corpus, max_n_merges_possible + (68 + 1));

        let n_uniq_chars = corpus.chars().collect::<HashSet<_>>().len();

        assert_eq!(
            vocabulary.id_to_token.len(),
            n_uniq_chars + max_n_merges_possible as usize
        );

        //    aaabdaaabac ->
        // 1. ZabdZabac ->
        // 2. ZYdZYac ->
        // 3. XdXac (len 5)
        assert_eq!(tokenized.len(), 5)
    }
}
