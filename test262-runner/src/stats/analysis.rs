use std::collections::HashMap;
use std::path::Path;

use crate::runner::TestResult;

#[derive(Default, Clone, Copy)]
pub struct SectionStats {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
}

#[derive(Default)]
pub struct Analysis {
    pub sections: HashMap<String, SectionStats>,
    pub failure_messages: HashMap<String, usize>,
    pub skip_reasons: HashMap<String, usize>,
}

impl Analysis {
    pub fn record(&mut self, root: &Path, path: &Path, result: &TestResult) {
        let section = section_from_path(root, path);
        let stats = self.sections.entry(section).or_default();
        stats.total += 1;

        match result {
            TestResult::Passed => stats.passed += 1,
            TestResult::Failed(reason) => {
                stats.failed += 1;
                *self
                    .failure_messages
                    .entry(normalize_message(reason))
                    .or_insert(0) += 1;
            }
            TestResult::Skipped(reason) => {
                stats.skipped += 1;
                *self.skip_reasons.entry(reason.clone()).or_insert(0) += 1;
            }
        }
    }
}

fn section_from_path(root: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    let parts: Vec<_> = rel
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(part) => Some(part.to_string_lossy().to_string()),
            _ => None,
        })
        .collect();

    match parts.as_slice() {
        [] => "(root)".to_string(),
        [first] => first.clone(),
        [first, second, ..] => format!("{first}/{second}"),
    }
}

fn normalize_message(message: &str) -> String {
    let first_line = message.lines().next().unwrap_or_default().trim();
    let compact = first_line.split_whitespace().collect::<Vec<_>>().join(" ");
    let max_chars = 140;

    if compact.chars().count() <= max_chars {
        compact
    } else {
        let clipped: String = compact.chars().take(max_chars).collect();
        format!("{clipped}...")
    }
}
