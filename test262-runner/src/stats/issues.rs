use std::cmp::Reverse;

use colored::Colorize;

use super::Analysis;

pub(super) fn print_common_failures(analysis: &Analysis) {
    let mut failures: Vec<_> = analysis
        .failure_messages
        .iter()
        .map(|(msg, count)| (msg.clone(), *count))
        .collect();
    failures.sort_by_key(|(_, count)| Reverse(*count));
    if failures.is_empty() {
        return;
    }

    println!("\n{}", "Most common failure messages:".bold());
    for (message, count) in failures.into_iter().take(10) {
        println!("  {:>4}x {}", count, message);
    }
}

pub(super) fn print_common_skips(analysis: &Analysis) {
    let mut skips: Vec<_> = analysis
        .skip_reasons
        .iter()
        .map(|(reason, count)| (reason.clone(), *count))
        .collect();
    skips.sort_by_key(|(_, count)| Reverse(*count));
    if skips.is_empty() {
        return;
    }

    println!("\n{}", "Most common skip reasons:".bold());
    for (reason, count) in skips.into_iter().take(5) {
        println!("  {:>4}x {}", count, reason);
    }
}
