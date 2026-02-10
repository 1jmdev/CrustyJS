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

    pub fn advance_by(&mut self, n: usize) {
        self.pos = self.pos.saturating_add(n).min(self.source.len());
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

    pub fn whitespace_len(&self) -> Option<usize> {
        let b0 = *self.source.get(self.pos)?;
        match b0 {
            b' ' | b'\t' | b'\r' | b'\n' | 0x0B | 0x0C => Some(1),
            0xC2 => {
                if self.source.get(self.pos + 1) == Some(&0xA0) {
                    Some(2)
                } else {
                    None
                }
            }
            0xE2 => {
                if self.source.get(self.pos + 1) == Some(&0x80) {
                    match self.source.get(self.pos + 2) {
                        Some(0xA8) | Some(0xA9) => Some(3),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn line_terminator_len(&self) -> Option<usize> {
        let b0 = *self.source.get(self.pos)?;
        match b0 {
            b'\r' | b'\n' => Some(1),
            0xE2 => {
                if self.source.get(self.pos + 1) == Some(&0x80) {
                    match self.source.get(self.pos + 2) {
                        Some(0xA8) | Some(0xA9) => Some(3),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
