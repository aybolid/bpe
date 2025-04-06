use bpers;

fn main() {
    let input = std::fs::read_to_string("src/main.rs").unwrap();
    // let input = "aa aa aa aa aa aa";
    let mut vocabulary = bpers::Vocabulary::new();
    let tokenized = vocabulary.learn(&input, 99999);
    println!(
        "input len: {}; tokenized len: {}",
        input.len(),
        tokenized.len()
    );

    println!("{:#?}", vocabulary.id_to_token);

    let encoded = bpers::encode(&input, &vocabulary);
    println!("{:?}", tokenized);
    println!("{:?}", encoded);
    println!("encoded size: {}", encoded.len());

    let decoded = bpers::decode(&encoded, &vocabulary);
    println!("{decoded}");
    println!("decoded size: {}", decoded.len());
}
