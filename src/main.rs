use std::fs::read_to_string;

use bpers;

fn main() {
    let input = read_to_string("bible.txt").unwrap();
    let mut vocabulary = bpers::Vocabulary::new();
    let tokenized = vocabulary.learn(&input, 99999);
    println!(
        "input len: {}; tokenized len: {}",
        input.len(),
        tokenized.len()
    )
}
