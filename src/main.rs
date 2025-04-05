use std::collections::HashMap;

use bpers;

fn main() {
    let input = "aaabdaaabac";
    let mut map = bpers::PairMap::new(HashMap::new());
    map.learn(input, 456);
}
