//! `wai feedback` — file a well-contexted issue against wai's upstream repo.
//!
//! wai is the reference implementation of the agent-issue-reporting feature.
//! The redactor, context-bundle serializer, error-scratch, and `gh` invocation
//! with fallback ladder all live in [`genesis::feedback`]; this command is the
//! thin surface that gathers inputs, wires them through genesis, and reports
//! the result.
//!
//! See `openspec/changes/add-feedback-subcommand` for the full design.

use miette::{IntoDiagnostic, Result};
use std::path::Path;

use crate::cli::FeedbackKind;
use crate::output::print_envelope;

/// The target repository (`owner/repo`) for filed feedback.
///
/// Derived from `Cargo.toml`'s `repository` field at build time. We hardcode
/// the canonical host/path form rather than parsing the URL at runtime so the
/// dry-run `gh` line is stable and dependency-free.
const TARGET_REPO: &str = "charly-vibes/wai";

/// The tool name used for error-scratch and context-gathering.
const TOOL_NAME: &str = "wai";

/// Arguments bound from the `Feedback` CLI variant.
pub struct FeedbackArgs {
    pub kind: Option<FeedbackKind>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub from_last_error: bool,
    pub dry_run: bool,
    pub web: bool,
    pub json: bool,
    pub yes: bool,
    pub no_context: bool,
    pub redact_remote: bool,
}

/// Run the `wai feedback` command.
pub fn run(args: FeedbackArgs) -> Result<()> {
    // 1. Resolve the kind (interactive prompt when omitted and not --yes/--json).
    let kind = match args.kind {
        Some(k) => k,
        None => match prompt_kind(args.yes, args.json) {
            Some(k) => k,
            None => {
                miette::bail!(
                    "kind is required (one of: bug, friction, docs-gap, aix-gap, idea). \
                     Re-run with an explicit kind, or use --from-last-error."
                );
            }
        },
    };

    // 2. Resolve title + body, optionally from the last error scratch.
    let (title, body, last_error) = resolve_content(&args)?;

    // 3. Build the issue body: user body + optional context bundle.
    let cwd = std::env::current_dir().into_diagnostic()?;
    let full_body = build_body(&args, &body, &last_error, &cwd, kind);

    // 4. Redact the body (secrets, env values, home paths; reduce git remote).
    let home_dir = dirs::home_dir();
    let git_remote = genesis::feedback::context::gather_context(
        TOOL_NAME,
        env!("CARGO_PKG_VERSION"),
        None,
        None,
        None,
        &cwd,
    )
    .git_remote;
    // Pre-pass: reduce any credential-bearing URL embedded in the body text
    // (e.g. an agent pasting `https://<pat>@github.com/...`). genesis's redactor
    // only reduces the *configured* remote, so we scan the body for arbitrary
    // `scheme://<creds>@host` URLs and reduce each one. This is the wai-side
    // enhancement destined for donation back to genesis::feedback::redactor.
    let body_pre = reduce_embedded_remotes(&full_body);
    let redacted_body = genesis::feedback::redactor::redact(
        &body_pre,
        home_dir.as_deref(),
        if args.redact_remote {
            git_remote.as_deref()
        } else {
            None
        },
    );

    let labels: Vec<&str> = kind.labels().to_vec();

    // 5. --dry-run: print title/body/labels + exact gh line, exit 0.
    if args.dry_run {
        return dry_run(&title, &redacted_body, &labels);
    }

    // 6. Live path: delegate to genesis::feedback::gh (with fallback ladder).
    live_file(&title, &redacted_body, &labels, &args)
}

// ── content resolution ────────────────────────────────────────────────

/// Resolve the issue title and body, optionally seeded from the last error
/// scratch entry when `--from-last-error` is set.
fn resolve_content(
    args: &FeedbackArgs,
) -> Result<(
    String,
    String,
    Option<genesis::feedback::scratch::ErrorRecord>,
)> {
    let last_error = if args.from_last_error {
        genesis::feedback::scratch::read_last_error(TOOL_NAME)
    } else {
        None
    };

    if args.from_last_error && last_error.is_none() {
        miette::bail!(
            "no recent error found in the scratch. Run a wai command that fails first, \
             then re-run `wai feedback bug --from-last-error`."
        );
    }

    let title = match (&args.title, &last_error) {
        (Some(t), _) => t.clone(),
        (None, Some(rec)) => format!("[{}] {} (exit {})", rec.kind, rec.argv.join(" "), rec.exit),
        (None, None) => {
            miette::bail!(
                "--title is required (or use --from-last-error to derive one from the last error)"
            );
        }
    };

    let body = match (&args.body, &last_error) {
        (Some(b), _) => b.clone(),
        (None, Some(rec)) => {
            let mut s = "**Reproduced from the last error.**\n\n".to_string();
            s.push_str(&format!("- command: `{}`\n", rec.argv.join(" ")));
            s.push_str(&format!("- exit code: {}\n", rec.exit));
            s.push_str(&format!("- kind: {}\n", rec.kind));
            if let Some(footer) = &rec.footer {
                s.push_str(&format!("- suggestion footer: {}\n", footer));
            }
            s
        }
        (None, None) => String::new(),
    };

    Ok((title, body, last_error))
}

/// Build the full issue body: the user/error body followed by the environment
/// context bundle (unless `--no-context`).
fn build_body(
    args: &FeedbackArgs,
    body: &str,
    last_error: &Option<genesis::feedback::scratch::ErrorRecord>,
    cwd: &Path,
    kind: FeedbackKind,
) -> String {
    let mut out = String::new();
    out.push_str(&format!("## {}\n\n", kind.as_word()));
    if !body.is_empty() {
        out.push_str(body);
        out.push_str("\n\n");
    }

    if !args.no_context {
        let (command, exit_code, footer) = match last_error {
            Some(rec) => (Some(rec.argv.join(" ")), Some(rec.exit), rec.footer.clone()),
            None => (None, None, None),
        };
        let bundle = genesis::feedback::context::gather_context(
            TOOL_NAME,
            env!("CARGO_PKG_VERSION"),
            command,
            exit_code,
            footer,
            cwd,
        );
        out.push_str(&genesis::feedback::context::format_context_bundle(&bundle));
    }

    out
}

// ── dry-run ───────────────────────────────────────────────────────────

/// Print the title, body, labels, and the exact `gh` line; exit 0.
fn dry_run(title: &str, body: &str, labels: &[&str]) -> Result<()> {
    println!("title: {}", title);
    println!("labels: {}", labels.join(", "));
    println!();
    println!("body:");
    println!("{}", body);
    println!();
    println!("{}", gh_line(title, labels));
    Ok(())
}

/// The exact `gh issue create` invocation that would be run.
///
/// Body is piped via stdin (`--body-file -`). Title and labels are
/// single-quoted with embedded quotes escaped POSIX-style.
fn gh_line(title: &str, labels: &[&str]) -> String {
    let mut parts = vec![
        "gh issue create".to_string(),
        format!("--repo {}", TARGET_REPO),
        format!("--title {}", shell_quote(title)),
    ];
    for label in labels {
        parts.push(format!("--label {}", shell_quote(label)));
    }
    parts.push("--body-file -".to_string());
    parts.join(" ")
}

/// Reduce any credential-bearing URL embedded in `text` to host/path.
///
/// Matches `scheme://<credentials>@<host>...` and replaces each occurrence
/// with the reduced form produced by [`genesis::feedback::redactor::reduce_git_remote_url`].
/// This catches PATs that an agent pastes into the body even when they differ
/// from the repo's configured git remote.
fn reduce_embedded_remotes(text: &str) -> String {
    let re = regex::Regex::new(r"(?i)https?://[^\s/@]+@[^\s]+").expect("valid regex");
    re.replace_all(text, |caps: &regex::Captures| {
        genesis::feedback::redactor::reduce_git_remote_url(&caps[0])
    })
    .to_string()
}

/// Single-quote a string for shell display, escaping embedded single quotes.
fn shell_quote(s: &str) -> String {
    let escaped = s.replace('\'', "'\\''");
    format!("'{}'", escaped)
}

// ── live filing ───────────────────────────────────────────────────────

/// File the issue via genesis::feedback::gh (or a prefilled URL with --web).
fn live_file(title: &str, body: &str, labels: &[&str], args: &FeedbackArgs) -> Result<()> {
    let mut opts = genesis::feedback::gh::CreateIssueOptions::new(TARGET_REPO, title, body);
    opts.labels = labels.iter().map(|s| s.to_string()).collect();
    opts.dry_run = false;

    if args.web {
        // --web: don't invoke gh; print the prefilled URL.
        let url = build_prefilled_url(TARGET_REPO, title, body, labels);
        println!("Open this URL to file your feedback:");
        println!("{}", url);
        return Ok(());
    }

    match genesis::feedback::gh::create_issue(&opts) {
        Ok(genesis::feedback::gh::GhResult::Created { url, number }) => {
            if args.json {
                let payload = serde_json::json!({
                    "filed": true,
                    "url": url,
                    "number": number,
                    "repo": TARGET_REPO,
                });
                print_envelope(genesis::envelope::EnvelopeKind::Ok, payload, vec![], vec![])?;
            } else {
                println!("Filed issue #{}: {}", number, url);
            }
            Ok(())
        }
        Ok(genesis::feedback::gh::GhResult::FallbackUrl(url)) => {
            if args.json {
                let payload = serde_json::json!({
                    "filed": false,
                    "fallback": "url",
                    "url": url,
                    "repo": TARGET_REPO,
                });
                print_envelope(genesis::envelope::EnvelopeKind::Ok, payload, vec![], vec![])?;
            } else {
                println!("Could not file via gh. Open this URL instead:");
                println!("{}", url);
            }
            Ok(())
        }
        Ok(genesis::feedback::gh::GhResult::LocalFile(path)) => {
            if args.json {
                let payload = serde_json::json!({
                    "filed": false,
                    "fallback": "local_file",
                    "path": path.to_string_lossy(),
                    "repo": TARGET_REPO,
                });
                print_envelope(genesis::envelope::EnvelopeKind::Ok, payload, vec![], vec![])?;
            } else {
                println!(
                    "Network unavailable. Report written to:\n  {}",
                    path.display()
                );
                println!("Retry later with `gh issue create` once you're back online.");
            }
            Ok(())
        }
        Err(message) => {
            // The fallback ladder already produced an actionable message
            // (install hint, auth hint, prefilled URL, etc.).
            if args.json {
                let payload = serde_json::json!({
                    "filed": false,
                    "fallback": "error",
                    "message": message,
                    "repo": TARGET_REPO,
                });
                print_envelope(genesis::envelope::EnvelopeKind::Ok, payload, vec![], vec![])?;
                Ok(())
            } else {
                Err(miette::Report::msg(message).wrap_err("feedback filing failed"))
            }
        }
    }
}

/// Build a prefilled GitHub issue URL (mirrors genesis's private helper, kept
/// here so --web works without depending on a private genesis function).
fn build_prefilled_url(repo: &str, title: &str, body: &str, labels: &[&str]) -> String {
    format!(
        "https://github.com/{}/issues/new?title={}&body={}&labels={}",
        repo,
        urlencode(title),
        urlencode(body),
        urlencode(&labels.join(",")),
    )
}

fn urlencode(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

// ── kind prompt ───────────────────────────────────────────────────────

/// Prompt the user to pick a feedback kind. Returns `None` if prompting is
/// skipped (non-interactive / json) and no kind was supplied.
fn prompt_kind(yes: bool, json: bool) -> Option<FeedbackKind> {
    if yes || json || !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        // Non-interactive: default to Bug rather than blocking.
        return Some(FeedbackKind::Bug);
    }
    use cliclack::select;
    let choice = select("What kind of feedback?")
        .item(
            FeedbackKind::Bug,
            "bug",
            "A defect: crash, hang, or wrong output",
        )
        .item(
            FeedbackKind::Friction,
            "friction",
            "A usability or workflow papercut",
        )
        .item(
            FeedbackKind::DocsGap,
            "docs-gap",
            "Missing, stale, or misleading docs",
        )
        .item(
            FeedbackKind::AixGap,
            "aix-gap",
            "A gap in agent-instructions (managed blocks, AGENTS.md)",
        )
        .item(FeedbackKind::Idea, "idea", "A feature idea or enhancement")
        .interact()
        .ok()?;
    Some(choice)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gh_line_targets_wai_upstream_and_reads_body_from_stdin() {
        let line = gh_line("crash on init", &["bug", "feedback"]);
        assert!(line.starts_with("gh issue create --repo charly-vibes/wai"));
        assert!(line.contains("--title 'crash on init'"));
        assert!(line.contains("--label 'bug' --label 'feedback'"));
        assert!(line.ends_with("--body-file -"));
    }

    #[test]
    fn gh_line_escapes_embedded_single_quotes() {
        let line = gh_line("it's broken", &["bug"]);
        assert!(line.contains("--title 'it'\\''s broken'"));
    }

    #[test]
    fn feedback_kind_labels_always_include_feedback() {
        for k in [
            FeedbackKind::Bug,
            FeedbackKind::Friction,
            FeedbackKind::DocsGap,
            FeedbackKind::AixGap,
            FeedbackKind::Idea,
        ] {
            let labels = k.labels();
            assert!(
                labels.contains(&"feedback"),
                "kind {:?} missing feedback label",
                k
            );
        }
    }

    #[test]
    fn redactor_strips_pat_from_remote_url() {
        let pat_url = "https://ghp_SECRET1234567890@github.com/charly-vibes/wai.git";
        // Direct genesis redactor on the configured remote:
        let redacted = genesis::feedback::redactor::redact(pat_url, None, Some(pat_url));
        assert!(!redacted.contains("ghp_SECRET"));
        assert!(redacted.contains("github.com/charly-vibes/wai"));
    }

    #[test]
    fn reduce_embedded_remotes_strips_pat_in_body_text() {
        let body =
            "remote was: https://ghp_SECRET1234567890@github.com/charly-vibes/wai.git and more"
                .to_string();
        let reduced = reduce_embedded_remotes(&body);
        assert!(!reduced.contains("ghp_SECRET"));
        assert!(reduced.contains("github.com/charly-vibes/wai"));
        assert!(reduced.contains(" and more"));
    }

    #[test]
    fn redactor_preserves_monkey_type_and_keymap() {
        let body = "configuring my keymap and testing monkey_type when wai crashed";
        let redacted = genesis::feedback::redactor::redact(body, None, None);
        assert!(redacted.contains("monkey_type"));
        assert!(redacted.contains("keymap"));
    }
}
