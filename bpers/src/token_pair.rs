#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Token {
    Lonely(Lonely),
    Pair(Pair),
}

impl Token {
    pub fn new(left: u32, right: Option<u32>) -> Self {
        match right {
            Some(right) => Self::Pair(Pair { left, right }),
            None => Self::Lonely(Lonely(left)),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Lonely(pub u32);

impl Lonely {
    pub fn new(n: u32) -> Self {
        Self(n)
    }

    pub fn as_token(&self) -> Token {
        Token::Lonely(*self)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Pair {
    pub left: u32,
    pub right: u32,
}

impl Pair {
    pub fn new(left: u32, right: u32) -> Self {
        Self { left, right }
    }

    pub fn as_token(&self) -> Token {
        Token::Pair(*self)
    }
}
