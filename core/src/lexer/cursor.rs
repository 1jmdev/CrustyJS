/// Character-level reader over the source string.
pub struct Cursor<'src> {
    source: &'src [u8],
    pos: usize,
}

impl<'src> Cursor<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            source: source.as_bytes(),
            pos: 0,
        }
    }

    /// Current byte position in the source.
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Peek at the current character without advancing.
    pub fn peek(&self) -> Option<u8> {
        self.source.get(self.pos).copied()
    }

    /// Peek at the next character (one ahead of current).
    pub fn peek_next(&self) -> Option<u8> {
        self.source.get(self.pos + 1).copied()
    }

    /// Advance one character and return it.
    pub fn advance(&mut self) -> Option<u8> {
        let ch = self.source.get(self.pos).copied()?;
        self.pos += 1;
        Some(ch)
    }

    /// Advance if the current character matches `expected`.
    pub fn match_char(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    /// Return a slice of the source from `start` to the current position.
    pub fn slice_from(&self, start: usize) -> &'src str {
        std::str::from_utf8(&self.source[start..self.pos]).expect("source should be valid UTF-8")
    }

    /// Whether the cursor has reached the end.
    pub fn is_at_end(&self) -> bool {
        self.pos >= self.source.len()
    }
}
