//! Shared display helpers for the `why` subcommand modules.

use owo_colors::OwoColorize;

/// Print a dim horizontal separator line.
pub(crate) fn separator() {
    println!("  {}", "─".repeat(58).dimmed());
}

pub(crate) fn section_header(title: &str) {
    separator();
    println!("  {}", title.bold());
    separator();
}
