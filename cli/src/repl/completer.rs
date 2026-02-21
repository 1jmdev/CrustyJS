use rustyline::completion::Pair;

pub fn complete_line(line: &str, pos: usize) -> (usize, Vec<Pair>) {
    let safe_pos = pos.min(line.len());
    let prefix = &line[..safe_pos];

    if let Some((start, members)) = member_completion(prefix) {
        return (start, pairs(&members));
    }

    let start = word_start(prefix);
    let needle = &prefix[start..];
    let words = keywords()
        .iter()
        .chain(globals().iter())
        .copied()
        .filter(|kw| kw.starts_with(needle))
        .collect::<Vec<_>>();

    (start, pairs(&words))
}

fn member_completion(prefix: &str) -> Option<(usize, Vec<&'static str>)> {
    let dot = prefix.rfind('.')?;
    let object_part = &prefix[..dot];
    let object_start = word_start(object_part);
    let object_name = &object_part[object_start..];
    let member_prefix = &prefix[dot + 1..];

    let members = members_for(object_name)?;
    let filtered = members
        .iter()
        .copied()
        .filter(|name| name.starts_with(member_prefix))
        .collect::<Vec<_>>();

    Some((dot + 1, filtered))
}

fn members_for(object_name: &str) -> Option<&'static [&'static str]> {
    match object_name {
        "Math" => Some(&[
            "abs", "ceil", "floor", "max", "min", "pow", "random", "round", "sqrt", "trunc",
        ]),
        "JSON" => Some(&["parse", "stringify"]),
        "Object" => Some(&["assign", "entries", "keys", "values"]),
        "Date" => Some(&["now"]),
        "console" => Some(&["log", "error", "warn", "info"]),
        _ => None,
    }
}

fn pairs(values: &[&str]) -> Vec<Pair> {
    values
        .iter()
        .map(|v| Pair {
            display: (*v).to_string(),
            replacement: (*v).to_string(),
        })
        .collect()
}

fn word_start(prefix: &str) -> usize {
    prefix
        .char_indices()
        .rev()
        .find(|(_, ch)| !is_ident_char(*ch))
        .map_or(0, |(idx, ch)| idx + ch.len_utf8())
}

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
}

fn keywords() -> &'static [&'static str] {
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
        "await",
        "async",
        "switch",
        "case",
        "break",
        "continue",
    ]
}

fn globals() -> &'static [&'static str] {
    &[
        "console",
        "Math",
        "JSON",
        "Object",
        "Date",
        "Number",
        "String",
        "Boolean",
        "Array",
        "Promise",
        "setTimeout",
        "setInterval",
    ]
}
