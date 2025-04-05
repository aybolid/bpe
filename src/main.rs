use bpers;

fn main() {
    let input = "aaabdaaabac";
    let mut map = bpers::PairMap::with_ascii_prelude();
    map.learn(&[input]);
}
