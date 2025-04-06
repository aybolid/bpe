use std::{collections::HashMap, time::Instant};

use crate::{Lonely, Pair, Token};

#[derive(Debug)]
pub struct Vocabulary {
    map: HashMap<u32, Token>,
}

impl Vocabulary {
    /// Creates a new `Vocabulary`.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Learns vocabulary from a given corpus.
    ///
    /// # Arguments
    /// * `corpus` - The input text corpus.
    /// * `n_merges` - The max number of merges to perform.
    ///
    /// # Returns
    /// An artifact of the learning process. Basically, it returns an encoded input corpus.
    pub fn learn(&mut self, corpus: &str, n_merges: u32) -> Vec<u32> {
        let mut tokens: Vec<u32> = corpus.chars().map(|char| char as u32).collect();

        // Initialize vocabulary with single characters
        for token in &tokens {
            if !self.map.contains_key(token) {
                let lonely = Lonely::new(*token).as_token();
                self.map.insert(*token, lonely);
            }
        }

        let mut next_token_id = (self.map.len() + 1) as u32;

        for n_merge in 0..n_merges {
            let start_time = Instant::now();
            let mut adjacent_pair_freq: HashMap<Pair, usize> = HashMap::new();
            for window in tokens.windows(2) {
                let token_pair = Pair::new(window[0], window[1]);
                *adjacent_pair_freq.entry(token_pair).or_insert(0) += 1;
            }

            // Find most frequent pair
            match adjacent_pair_freq.into_iter().max_by_key(|(_, freq)| *freq) {
                Some((most_freq_pair, pair_freq)) if pair_freq > 1 => {
                    self.map.insert(next_token_id, most_freq_pair.as_token());

                    let mut updated_tokens = Vec::with_capacity(tokens.len());
                    let mut i = 0;
                    while i < tokens.len() {
                        if i + 1 < tokens.len()
                            && tokens[i] == most_freq_pair.left
                            && tokens[i + 1] == most_freq_pair.right
                        {
                            updated_tokens.push(next_token_id);
                            i += 2;
                        } else {
                            updated_tokens.push(tokens[i]);
                            i += 1;
                        }
                    }

                    tokens = updated_tokens;
                    next_token_id += 1;

                    if (n_merge + 1) % 10 == 0 {
                        println!("Merge #{}", n_merge + 1);
                        println!("\tMerge took: {:>22?}", start_time.elapsed());
                        println!("\tTokenized input size: {:>12}", tokens.len());
                        println!("\tVocabulary size: {:>17}", self.map.len());
                    }
                }
                _ => break, // No pairs with frequency > 1, stop merging
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
            vocabulary.map.len(),
            n_uniq_chars + max_n_merges_possible as usize
        );

        //    aaabdaaabac ->
        // 1. ZabdZabac ->
        // 2. ZYdZYac ->
        // 3. XdXac (len 5)
        assert_eq!(tokenized.len(), 5)
    }
}
