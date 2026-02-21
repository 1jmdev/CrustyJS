pub fn hint_for(line: &str, pos: usize) -> Option<String> {
    if pos < line.len() {
        return None;
    }

    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(prediction) = predict_console_log(trimmed) {
        return Some(format!("  => {prediction}"));
    }

    if trimmed == "console." {
        return Some("log".to_string());
    }
    if trimmed == "Math." {
        return Some("abs(1)".to_string());
    }
    if trimmed == "JSON." {
        return Some("parse('{}')".to_string());
    }

    None
}

fn predict_console_log(input: &str) -> Option<String> {
    let inner = input
        .strip_prefix("console.log(")?
        .strip_suffix(')')?
        .trim();

    if inner.starts_with('"') && inner.ends_with('"') && inner.len() >= 2 {
        return Some(inner[1..inner.len() - 1].to_string());
    }
    if inner.starts_with('\'') && inner.ends_with('\'') && inner.len() >= 2 {
        return Some(inner[1..inner.len() - 1].to_string());
    }
    if inner.parse::<f64>().is_ok() {
        return Some(inner.to_string());
    }
    match inner {
        "true" | "false" | "null" | "undefined" => Some(inner.to_string()),
        _ => None,
    }
}
