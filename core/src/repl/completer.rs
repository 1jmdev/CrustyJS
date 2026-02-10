pub fn keywords() -> &'static [&'static str] {
    &[
        "let",
        "const",
        "function",
        "if",
        "else",
        "while",
        "for",
        "of",
        "return",
        "true",
        "false",
        "null",
        "undefined",
        "typeof",
        "try",
        "catch",
        "finally",
        "throw",
        "new",
        "class",
        "extends",
        "super",
        "instanceof",
    ]
}

pub fn suggest_for(input: &str) -> Vec<&'static str> {
    if input.ends_with("Math.") {
        return vec![
            "abs", "ceil", "floor", "max", "min", "pow", "random", "round", "sqrt", "trunc",
        ];
    }
    if input.ends_with("JSON.") {
        return vec!["parse", "stringify"];
    }
    if input.ends_with("Object.") {
        return vec!["assign", "entries", "keys", "values"];
    }
    if input.ends_with("Date.") {
        return vec!["now"];
    }

    keywords()
        .iter()
        .copied()
        .filter(|kw| kw.starts_with(input))
        .collect()
}
