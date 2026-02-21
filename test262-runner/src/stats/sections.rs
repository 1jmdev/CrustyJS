use std::cmp::Reverse;

use colored::Colorize;

use super::{Analysis, SectionStats};

pub(super) fn print_sections(analysis: &Analysis) {
    let mut sections: Vec<_> = analysis
        .sections
        .iter()
        .map(|(name, stats)| (name.clone(), *stats))
        .collect();
    sections.sort_by_key(|(_, stats)| Reverse(stats.total));

    println!("\n{}", "Largest sections by volume:".bold());
    for (name, stats) in sections.iter().take(8) {
        println!(
            "  {:28} {:>6} tests | pass {:>6} | fail {:>6} | skip {:>6}",
            name,
            stats.total,
            rate(stats.passed, stats.total),
            rate(stats.failed, stats.total),
            rate(stats.skipped, stats.total)
        );
    }

    let min_tests = 25;
    let mut ranked: Vec<_> = sections
        .into_iter()
        .filter(|(_, stats)| stats.total >= min_tests)
        .collect();
    ranked.sort_by(compare_pass_rate);
    if ranked.is_empty() {
        return;
    }

    println!(
        "\n{} {min_tests} tests):",
        "Worst sections (by pass rate, >=".bold()
    );
    for (name, stats) in ranked.iter().take(8) {
        println!(
            "  {:28} pass {:>6} ({} / {})",
            name,
            rate(stats.passed, stats.total),
            stats.passed,
            stats.total
        );
    }

    println!(
        "\n{} {min_tests} tests):",
        "Best sections (by pass rate, >=".bold()
    );
    for (name, stats) in ranked.iter().rev().take(8) {
        println!(
            "  {:28} pass {:>6} ({} / {})",
            name,
            rate(stats.passed, stats.total),
            stats.passed,
            stats.total
        );
    }
}

fn compare_pass_rate(a: &(String, SectionStats), b: &(String, SectionStats)) -> std::cmp::Ordering {
    let a_rate = a.1.passed as f64 / a.1.total as f64;
    let b_rate = b.1.passed as f64 / b.1.total as f64;
    a_rate
        .partial_cmp(&b_rate)
        .unwrap_or(std::cmp::Ordering::Equal)
        .then_with(|| b.1.total.cmp(&a.1.total))
}

fn rate(numerator: usize, denominator: usize) -> String {
    if denominator == 0 {
        return "n/a".to_string();
    }
    format!("{:.1}%", numerator as f64 * 100.0 / denominator as f64)
}
