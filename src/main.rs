#![allow(unused_assignments)]

use clap::Parser;
use miette::Result;

mod cli;
mod commands;
mod config;
mod context;
mod error;
pub mod freshness;
mod guided_flows;
mod help;
mod json;
mod llm;
pub mod managed_block;
pub mod openspec;
mod output;
pub mod plugin;
mod state;
mod sync_core;
mod tutorial;
#[allow(dead_code)]
mod workflows;
mod workspace;

use cli::Cli;
use context::{CliContext, set_context};
use output::print_json_line;

fn main() -> Result<()> {
    miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .terminal_links(true)
                .unicode(true)
                .context_lines(2)
                .build(),
        )
    }))
    .ok();

    let args: Vec<String> = std::env::args().collect();

    // Handle --version --json before clap processes it (clap's built-in --version
    // doesn't participate in the global --json flag)
    let has_version = args.iter().any(|a| {
        a == "--version"
            || a == "-V"
            || (a.starts_with("-") && !a.starts_with("--") && a.contains('V') && !a.contains('h'))
    });
    if has_version
        && !args
            .iter()
            .any(|a| a == "--help" || a == "-h" || a == "-jh")
    {
        let has_json = args.iter().any(|a| {
            a == "--json"
                || a == "-j"
                || (a.starts_with("-j") && !a.starts_with("--") && !a.contains('h'))
        });
        if has_json {
            use genesis::envelope::Envelope;
            let envelope = Envelope::success(
                genesis::envelope::EnvelopeKind::Version,
                serde_json::json!({
                    "name": "wai",
                    "version": cli::VERSION
                }),
                vec![],
                vec![],
            );
            let _ = print_json_line(&envelope);
            return Ok(());
        }
    }

    if let Some(output) = help::try_render_help(&args) {
        print!("{}", output);
        return Ok(());
    }

    let cli = Cli::parse();
    set_context(CliContext {
        json: cli.json,
        no_input: cli.no_input,
        yes: cli.yes,
        safe: cli.safe,
        verbose: cli.verbose,
        quiet: cli.quiet,
    });
    let guide = cli::build_guide();
    let argv: Vec<String> = std::env::args().collect();
    match commands::run(cli, &guide) {
        Ok(_) => Ok(()),
        Err(err) => {
            // Error-scratch: persist the last error so `wai feedback --from-last-error`
            // can rebuild a well-contexted issue. Best-effort — never shadows the
            // real error, never changes the exit code.
            let footer = err.help().map(|h| h.to_string());
            genesis::feedback::scratch::write_scratch_best_effort(
                "wai",
                &genesis::feedback::scratch::ErrorRecord {
                    ts: scratch_timestamp(),
                    argv: argv.clone(),
                    exit: 1,
                    footer,
                    kind: "error".to_string(),
                },
            );

            let context = context::current_context();
            if context.json {
                use genesis::envelope::{Envelope, ErrorResult, RemediationEntry};
                let err_result = ErrorResult::new(
                    "E000",
                    &err.to_string(),
                    None,
                    None,
                    None,
                    vec![],
                    vec![RemediationEntry {
                        command: "wai doctor".into(),
                        description: "run workspace health check".into(),
                    }],
                )
                .expect("remediation must be non-empty");
                let _ = print_json_line(&Envelope::error(err_result, vec![]));
            } else {
                // Footer hook: when the error carries no self-healing help, offer
                // the feedback subcommand so the user can file the issue.
                if err.help().is_none() {
                    eprintln!("Feedback: wai feedback bug --from-last-error");
                }
            }
            Err(err)
        }
    }
}

/// Generate an ISO 8601 UTC timestamp without pulling in chrono at the call site.
fn scratch_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }
    let month_days = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m = 1;
    for &md in &month_days {
        if remaining < md {
            break;
        }
        remaining -= md;
        m += 1;
    }
    let d = remaining + 1;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m, d, hours, minutes, seconds
    )
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}
