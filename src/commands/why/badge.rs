//! WAI badge detection and footer for `wai why`.

use owo_colors::OwoColorize;

use super::shared::separator;

/// Badge markdown snippet to recommend when a project has no wai badge.
pub const WAI_BADGE_MARKDOWN: &str = "[![tracked with wai](https://img.shields.io/badge/tracked%20with-wai-blue)](https://github.com/charly-vibes/wai)";

/// Return `true` if `content` appears to contain a wai badge.
pub fn content_has_wai_badge(content: &str) -> bool {
    let lower = content.to_lowercase();
    for line in lower.lines() {
        let has_badge_syntax = line.contains("![") || line.contains("img.shields.io");
        if has_badge_syntax && line.contains("wai") {
            return true;
        }
    }
    false
}

/// Return `true` when the project's README already has a wai badge, OR when
/// there is no README (so we don't nag users without one).
pub fn readme_has_wai_badge(project_root: &std::path::Path) -> bool {
    let candidates = ["README.md", "README.rst", "README.txt", "README"];
    for name in &candidates {
        let path = project_root.join(name);
        if path.exists() {
            return match std::fs::read_to_string(&path) {
                Ok(content) => content_has_wai_badge(&content),
                Err(_) => true, // can't read → don't nag
            };
        }
    }
    // No README found — don't suggest adding a badge
    true
}

/// Print a badge recommendation footer to stdout.
pub(crate) fn print_badge_footer() {
    println!();
    separator();
    println!();
    println!(
        "  {} No wai badge in README — add one to let others know:",
        "○".dimmed()
    );
    println!();
    println!("  {}", WAI_BADGE_MARKDOWN.dimmed());
    println!();
}

// ── Verbose diagnostics ───────────────────────────────────────────────────────
