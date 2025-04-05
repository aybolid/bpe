use std::u8;

#[derive(Debug, Clone, Copy)]
pub struct TokenPair {
    pub left: u32,
    pub right: Option<u32>,
}

impl TokenPair {
    pub fn new(left: u32, right: u32) -> Self {
        Self {
            left,
            right: Some(right),
        }
    }

    pub fn from_ascii(left: u8) -> Self {
        Self {
            left: left as u32,
            right: None,
        }
    }
}

pub fn gen_ascii_prelude() -> Box<[TokenPair; 256]> {
    let mut prelude = Box::new([TokenPair::from_ascii(0); 256]);

    for (i, pair) in prelude.iter_mut().enumerate().skip(1) {
        *pair = TokenPair::from_ascii(i as u8)
    }

    prelude
}
