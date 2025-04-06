use crate::{Pair, Token, Vocabulary};

pub fn encode(input: &str, vocab: &Vocabulary) -> Vec<u32> {
    let mut encoded: Vec<u32> = input.chars().map(|c| c as u32).collect();

    let mut merge_occurred = true;
    while merge_occurred {
        merge_occurred = false;

        let mut i = 0;
        let mut new_encoded = Vec::with_capacity(encoded.len());

        while i < encoded.len() {
            if i == encoded.len() - 1 {
                new_encoded.push(encoded[i]);
                break;
            }

            let pair = Pair::new(encoded[i], encoded[i + 1]);
            if let Some(token_id) = vocab.token_pair_to_id.get(&pair) {
                new_encoded.push(*token_id);
                merge_occurred = true;
                i += 2;
            } else {
                new_encoded.push(encoded[i]);
                i += 1;
            }
        }

        encoded = new_encoded;
    }

    encoded
}

pub fn decode(encoded: &[u32], vocab: &Vocabulary) -> String {
    let mut encoded_vec = encoded.to_vec();
    let mut is_decoded = false;

    while !is_decoded {
        is_decoded = true;
        let mut new_encoded = Vec::with_capacity(encoded_vec.len() * 2);

        for &id in &encoded_vec {
            match vocab.id_to_token.get(&id) {
                Some(Token::Pair(pair)) => {
                    is_decoded = false;
                    new_encoded.push(pair.left);
                    new_encoded.push(pair.right);
                }
                Some(Token::Lonely(lone)) => {
                    new_encoded.push(lone.0);
                }
                None => {
                    todo!();
                }
            }
        }

        encoded_vec = new_encoded;
    }

    // Convert final code points to characters
    encoded_vec
        .iter()
        .map(|&char_u32| char::from_u32(char_u32).unwrap())
        .collect()
}
