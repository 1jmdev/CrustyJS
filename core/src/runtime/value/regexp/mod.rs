mod engine;

pub use engine::MatchResult;

use regex::Regex;

use crate::runtime::gc::{Trace, Tracer};

#[derive(Debug, Clone, Default)]
pub struct RegExpFlags {
    pub global: bool,
    pub ignore_case: bool,
    pub multiline: bool,
    pub dotall: bool,
    pub unicode: bool,
    pub sticky: bool,
}

impl RegExpFlags {
    pub fn from_str(flags: &str) -> Result<Self, String> {
        let mut f = Self::default();
        for ch in flags.chars() {
            match ch {
                'g' => {
                    if f.global {
                        return Err("duplicate flag 'g'".to_string());
                    }
                    f.global = true;
                }
                'i' => {
                    if f.ignore_case {
                        return Err("duplicate flag 'i'".to_string());
                    }
                    f.ignore_case = true;
                }
                'm' => {
                    if f.multiline {
                        return Err("duplicate flag 'm'".to_string());
                    }
                    f.multiline = true;
                }
                's' => {
                    if f.dotall {
                        return Err("duplicate flag 's'".to_string());
                    }
                    f.dotall = true;
                }
                'u' => {
                    if f.unicode {
                        return Err("duplicate flag 'u'".to_string());
                    }
                    f.unicode = true;
                }
                'y' => {
                    if f.sticky {
                        return Err("duplicate flag 'y'".to_string());
                    }
                    f.sticky = true;
                }
                c => return Err(format!("invalid regex flag '{c}'")),
            }
        }
        Ok(f)
    }

    pub fn to_flag_string(&self) -> String {
        let mut s = String::new();
        if self.global {
            s.push('g');
        }
        if self.ignore_case {
            s.push('i');
        }
        if self.multiline {
            s.push('m');
        }
        if self.dotall {
            s.push('s');
        }
        if self.unicode {
            s.push('u');
        }
        if self.sticky {
            s.push('y');
        }
        s
    }
}

/// A compiled JS regular expression.
#[derive(Debug, Clone)]
pub struct JsRegExp {
    pub pattern: String,
    pub flags: RegExpFlags,
    pub last_index: usize,
    compiled: Regex,
}

impl JsRegExp {
    pub fn new(pattern: &str, flags: RegExpFlags) -> Result<Self, String> {
        let compiled = compile_regex(pattern, &flags)?;
        Ok(Self {
            pattern: pattern.to_string(),
            flags,
            last_index: 0,
            compiled,
        })
    }

    pub fn compiled(&self) -> &Regex {
        &self.compiled
    }

    pub fn flag_string(&self) -> String {
        self.flags.to_flag_string()
    }
}

/// Build a Rust `Regex` from a JS pattern + flags.
fn compile_regex(pattern: &str, flags: &RegExpFlags) -> Result<Regex, String> {
    let mut rust_pattern = String::new();
    let has_inline = flags.ignore_case || flags.multiline || flags.dotall;
    if has_inline {
        rust_pattern.push_str("(?");
        if flags.ignore_case {
            rust_pattern.push('i');
        }
        if flags.multiline {
            rust_pattern.push('m');
        }
        if flags.dotall {
            rust_pattern.push('s');
        }
        rust_pattern.push(')');
    }
    rust_pattern.push_str(pattern);

    Regex::new(&rust_pattern).map_err(|e| format!("invalid regex: {e}"))
}

impl std::fmt::Display for JsRegExp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "/{}/{}", self.pattern, self.flag_string())
    }
}

impl Trace for JsRegExp {
    fn trace(&self, _tracer: &mut Tracer) {}
}
