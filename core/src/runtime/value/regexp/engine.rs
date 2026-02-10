use super::JsRegExp;

/// Result of a single regex exec/match operation.
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// The full match string.
    pub full_match: String,
    /// Capture group strings (index 0 = full match).
    pub captures: Vec<Option<String>>,
    /// Start index of the match in the input.
    pub index: usize,
}

impl JsRegExp {
    /// Execute the regex against the string, starting at `last_index`
    /// for global/sticky regexps. Returns `None` on no match.
    pub fn exec(&mut self, input: &str) -> Option<MatchResult> {
        let start = if self.flags.global || self.flags.sticky {
            self.last_index
        } else {
            0
        };

        if start > input.len() {
            if self.flags.global || self.flags.sticky {
                self.last_index = 0;
            }
            return None;
        }

        let haystack = &input[start..];
        let captures = self.compiled().captures(haystack)?;

        let full = captures.get(0).unwrap();
        if self.flags.sticky && full.start() != 0 {
            self.last_index = 0;
            return None;
        }

        let match_start = start + full.start();
        let match_end = start + full.end();

        if self.flags.global || self.flags.sticky {
            self.last_index = match_end;
        }

        let caps: Vec<Option<String>> = captures
            .iter()
            .map(|m| m.map(|m| m.as_str().to_string()))
            .collect();

        Some(MatchResult {
            full_match: full.as_str().to_string(),
            captures: caps,
            index: match_start,
        })
    }

    /// Test whether the regex matches the string (updates lastIndex
    /// for global/sticky).
    pub fn test(&mut self, input: &str) -> bool {
        self.exec(input).is_some()
    }

    /// Find all matches (for global flag). Returns list of match
    /// strings.
    pub fn match_all(&mut self, input: &str) -> Vec<String> {
        let mut results = Vec::new();
        self.last_index = 0;
        loop {
            match self.exec(input) {
                Some(m) => {
                    results.push(m.full_match);
                    // Prevent infinite loop on zero-length match
                    if self.last_index == 0 {
                        break;
                    }
                }
                None => break,
            }
        }
        self.last_index = 0;
        results
    }
}
