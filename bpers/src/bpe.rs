use thiserror::Error;

use crate::{Pair, Token, Vocabulary};

#[derive(Error, Debug)]
pub enum EncodingError {
    #[error("Input contains character '{char}' (code {code}) which is not in the vocabulary.")]
    CharNotInVocab { char: String, code: u32 },
    #[error("Invalid UTF-32 character (code: {code})")]
    InvalidChar { code: u32 },
    #[error("Unknown token with code {code}")]
    UnknownToken { code: u32 },
}

/// Encodes an input string into a sequence of token IDs using a pre-learned vocabulary.
///
/// This function applies the merge rules defined in the vocabulary greedily.
///
/// # Arguments
/// * `input` - The string to encode.
/// * `vocab` - A reference to the `Vocabulary` containing the learned merge rules.
///
/// # Returns
/// A `Vec<u32>` representing the encoded token sequence, or an error if unknown characters are encountered.
pub fn encode(input: &str, vocab: &Vocabulary) -> Result<Vec<u32>, EncodingError> {
    let mut tokens: Vec<u32> = input.chars().map(|c| c as u32).collect();

    for &token_id in &tokens {
        if !vocab.id_to_token.contains_key(&token_id) {
            return Err(EncodingError::CharNotInVocab {
                char: char::from_u32(token_id)
                    .map_or_else(|| "Invalid UTF-32".to_string(), |c| c.to_string()),
                code: token_id,
            });
        }
    }

    loop {
        let mut best_pair: Option<(usize, Pair, u32)> = None; // (index, pair, merged_id)

        for i in 0..tokens.len().saturating_sub(1) {
            let current_pair = Pair::new(tokens[i], tokens[i + 1]);
            if let Some(&merged_id) = vocab.token_pair_to_id.get(&current_pair) {
                if best_pair.is_none() || merged_id < best_pair.unwrap().2 {
                    best_pair = Some((i, current_pair, merged_id));
                }
            }
        }

        if best_pair.is_none() {
            break;
        }

        let (_, pair_to_merge, merged_id) = best_pair.expect("cant be None");
        let mut updated_tokens = Vec::with_capacity(tokens.len());
        let mut i = 0;
        while i < tokens.len() {
            if i + 1 < tokens.len()
                && tokens[i] == pair_to_merge.left
                && tokens[i + 1] == pair_to_merge.right
            {
                updated_tokens.push(merged_id);
                i += 2;
            } else {
                updated_tokens.push(tokens[i]);
                i += 1;
            }
        }
        tokens = updated_tokens;
    }

    Ok(tokens)
}

/// Decodes a sequence of token IDs back into a string using the vocabulary.
///
/// # Arguments
/// * `token_ids` - A slice of token IDs (`u32`) to decode.
/// * `vocab` - A reference to the `Vocabulary` used for encoding.
///
/// # Returns
/// The decoded `String`, or an error if an unknown token ID is encountered or
/// if a token ID cannot be represented as a valid character.
pub fn decode(token_ids: &[u32], vocab: &Vocabulary) -> Result<String, EncodingError> {
    let mut decoded_chars: Vec<char> = Vec::new();

    for &id in token_ids {
        let mut decoding_stack: Vec<u32> = vec![id];
        while let Some(current_id) = decoding_stack.pop() {
            match vocab.id_to_token.get(&current_id) {
                Some(Token::Lonely(lonely)) => match std::char::from_u32(lonely.0) {
                    Some(c) => decoded_chars.push(c),
                    None => {
                        return Err(EncodingError::InvalidChar { code: lonely.0 });
                    }
                },
                Some(Token::Pair(pair)) => {
                    // Push right then left, so left gets processed first (LIFO)
                    decoding_stack.push(pair.right);
                    decoding_stack.push(pair.left);
                }
                None => {
                    return Err(EncodingError::UnknownToken { code: current_id });
                }
            }
        }
    }

    Ok(decoded_chars.into_iter().collect())
}
