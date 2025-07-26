#[derive(Clone, Copy)]
pub struct Annotation {
    pub flags: u32,
}

impl Annotation {
    pub fn new(flags: u32) -> Self {
        Self { flags }
    }

    pub fn dedupe() -> Self {
        Self { flags: Self::DEDUPE }
    }

    pub fn key() -> Self {
        Self { flags: Self::KEY }
    }

    /// Dedupe Expression
    pub const DEDUPE: u32 = 1 << 0;
    /// Property Key
    pub const KEY: u32 = 1 << 1;

    pub fn is_dedupe(&self) -> bool {
        self.flags & Self::DEDUPE != 0
    }

    pub fn is_key(&self) -> bool {
        self.flags & Self::KEY != 0
    }

    pub const ID_NAME: &'static str = "__oveo__";
}
