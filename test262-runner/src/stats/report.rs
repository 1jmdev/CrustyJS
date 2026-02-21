use super::Analysis;
use super::issues::{print_common_failures, print_common_skips};
use super::sections::print_sections;
use colored::Colorize;

pub fn print_analysis(analysis: &Analysis) {
    println!("\n{}", "Analysis (--analyze)".bold().cyan());
    print_sections(analysis);
    print_common_failures(analysis);
    print_common_skips(analysis);
}
