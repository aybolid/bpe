#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
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

    pub fn new_single(left: u32) -> Self {
        Self { left, right: None }
    }
}
