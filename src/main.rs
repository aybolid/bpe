use std::{collections::HashMap, fs::read_to_string};

use bpers;

fn main() {
    let input = read_to_string("bpers/src/pair_map.rs").unwrap();
    let mut vocabulary = bpers::Vocabulary::new(HashMap::new());
    let tokenized = vocabulary.learn(&input, 99999);
    println!(
        "input len: {}; tokenized len: {}",
        input.len(),
        tokenized.len()
    )
}
