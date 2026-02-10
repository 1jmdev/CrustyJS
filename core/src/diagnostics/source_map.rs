#[derive(Debug, Clone, Copy)]
pub struct SourcePos {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct SourceMap {
    line_offsets: Vec<usize>,
}

impl SourceMap {
    pub fn from_source(source: &str) -> Self {
        let mut line_offsets = vec![0];
        for (idx, ch) in source.char_indices() {
            if ch == '\n' {
                line_offsets.push(idx + 1);
            }
        }
        Self { line_offsets }
    }

    pub fn byte_to_pos(&self, byte: usize) -> SourcePos {
        let line_idx = match self.line_offsets.binary_search(&byte) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        };
        let line_start = self.line_offsets.get(line_idx).copied().unwrap_or(0);
        SourcePos {
            line: line_idx + 1,
            col: byte.saturating_sub(line_start) + 1,
        }
    }
}
