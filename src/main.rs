use bpers;

fn main() {
    let input = std::fs::read_to_string("bible.txt").unwrap();
    // let input = "aa ab ab aabb aab bb aab aab aa aa";
    let mut vocabulary = bpers::Vocabulary::new();
    let tokenized = vocabulary.learn(&input, 2000);
    println!(
        "input len: {}; tokenized len: {}",
        input.len(),
        tokenized.len()
    );

    let encoded = bpers::encode(&input, &vocabulary);
    println!("encoded size: {}", encoded.len());

    let decoded = bpers::decode(&tokenized, &vocabulary);
    // println!("{decoded}");
    println!("decoded size: {}", decoded.len());
}
