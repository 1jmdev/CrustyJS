use std::borrow::Cow;

use owo_colors::OwoColorize;

const KEYWORDS: [&str; 15] = [
    "let",
    "const",
    "function",
    "if",
    "else",
    "while",
    "for",
    "return",
    "true",
    "false",
    "null",
    "undefined",
    "try",
    "catch",
    "throw",
];

pub fn highlight_line(line: &str) -> Cow<'_, str> {
    let mut out = line.to_string();
    out = out.replace("console", &"console".cyan().to_string());
    out = out.replace("Math", &"Math".cyan().to_string());
    for kw in KEYWORDS {
        out = out.replace(kw, &kw.blue().bold().to_string());
    }
    Cow::Owned(out)
}

pub fn highlight_prompt(prompt: &str) -> Cow<'_, str> {
    if prompt == "> " {
        return Cow::Owned(format!("{} ", ">".bright_green().bold()));
    }
    if prompt == "... " {
        return Cow::Owned(format!("{} ", "...".yellow().bold()));
    }
    Cow::Borrowed(prompt)
}

pub fn highlight_hint(hint: &str) -> Cow<'_, str> {
    Cow::Owned(hint.bright_black().to_string())
}
