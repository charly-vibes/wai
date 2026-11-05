use miette::Result;
use owo_colors::OwoColorize;

// ─── preflight ─────────────────────────────────────────────────

/// One environment check for `wai pipeline preflight`: run `tool <args>` in
/// the current directory and report the outcome. Spawn failure means the
/// tool is not installed; a nonzero exit means it is installed but broken.
fn preflight_check(name: &str, args: &[&str], fix: &str) -> std::result::Result<String, String> {
    match std::process::Command::new(name).args(args).output() {
        Ok(out) if out.status.success() => {
            let version = String::from_utf8_lossy(&out.stdout);
            let first = version.lines().next().unwrap_or(name).trim();
            Ok(first.to_string())
        }
        Ok(out) => Err(format!(
            "✗ {name}: exit {} (args: {args:?}) — {fix}",
            out.status.code().unwrap_or(-1)
        )),
        Err(e) => Err(format!("✗ {name}: not runnable ({e}) — {fix}")),
    }
}

/// Verify the tool environment of the current directory before a spawn
/// (ticket wai-gvqj). Week 2026-10-13..19 logged 7 opaque lefthook
/// exit-2 failures, 11 wrong-prefix bd failures, and 17 testaruda failures
/// in spawned worktrees — all discoverable in seconds before spawning.
/// Anti-goals: no auto-install at spawn, no network calls — fail fast only.
pub fn cmd_preflight() -> Result<()> {
    let checks: [(&str, Vec<&str>, &str); 3] = [
        (
            "lefthook",
            vec!["--version"],
            "orchestrator pre-commit will fail mid-run. Fix: install lefthook \
             (brew install lefthook | npm install -g lefthook | just setup)",
        ),
        (
            "bd",
            vec!["list"],
            "beads state unreadable in this worktree. Fix: run `bd doctor` and \
             verify .beads/ is present and initialized here",
        ),
        (
            "testaruda",
            vec!["init"],
            "test selection unavailable; `just test` will fail at init. Fix: \
             cargo install --locked testaruda (init is idempotent — re-running \
             is safe; a stale store is recreated on first `just test`)",
        ),
    ];

    println!(
        "Preflight: verifying tool environment ({})…",
        std::env::current_dir().unwrap_or_default().display()
    );

    let failures: Vec<String> = checks
        .iter()
        .filter_map(|(tool, args, fix)| preflight_check(tool, args, fix).err())
        .collect();
    report_preflight_outcome(&failures, checks.len())
}

/// Print the preflight summary; returns Err when any check failed.
fn report_preflight_outcome(failures: &[String], total: usize) -> Result<()> {
    if failures.is_empty() {
        println!("{} preflight OK — environment ready for spawn", "✓".green());
        return Ok(());
    }

    println!();
    println!(
        "  {} Preflight failed ({} of {} checks):",
        "✗".red(),
        failures.len(),
        total
    );
    println!();
    for f in failures {
        println!("    {f}");
    }
    println!();
    println!(
        "  {} Fix the above before spawning into this environment",
        "→".cyan()
    );
    Err(miette::miette!(
        "preflight failed: {} check(s) broken",
        failures.len()
    ))
}

// ─── wait ─────────────────────────────────────────────────────────────────────

/// Validate the PR argument: non-empty and not flag-like, so `gh pr checks`
/// receives a positional PR/branch and never a forged option.
fn validate_pr_arg(pr: &str) -> Result<()> {
    if pr.is_empty() || pr.starts_with('-') {
        return Err(miette::miette!(
            "invalid PR argument '{pr}': must be a PR number or branch name"
        ));
    }
    Ok(())
}

/// Condense multi-line `gh pr checks` output into one bounded line for the
/// timeout summary's last observed state.
fn condense_state(stdout: &str) -> String {
    let one_line = stdout
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" | ");
    if one_line.is_empty() {
        return "unknown".to_string();
    }
    let mut line: String = one_line.chars().take(120).collect();
    if line.chars().count() < one_line.chars().count() {
        line.push('…');
    }
    line
}

/// Format the time budget in minutes as a human-readable duration.
fn format_elapsed(minutes: u64) -> String {
    if minutes >= 60 && minutes.is_multiple_of(60) {
        format!("{}h", minutes / 60)
    } else {
        format!("{minutes}m")
    }
}

/// Canonical blocking wait for a PR's checks (ticket wai-63h9). Blocks via
/// gh's own watch mode — no poll loop is written by the agent or by wai.
/// Emits exactly ONE summary line and exits: 0 on pass (including
/// zero-checks PRs), nonzero on check failure or timeout exhaustion. On
/// timeout, a single re-query reports the last observed state so the agent
/// never re-polls to reconstruct it. Week 2026-10-13..19 measured 539
/// `sleep >=60s` poll commands (~53h blocked wall-clock); this primitive
/// replaces those improvised loops.
pub fn cmd_wait(pr: &str, timeout_minutes: u64) -> Result<()> {
    use wait_timeout::ChildExt;

    validate_pr_arg(pr)?;
    let budget = std::time::Duration::from_secs(timeout_minutes.saturating_mul(60));

    // gh's watch mode does the waiting; wai only supervises it against the
    // time budget. Output is captured (not streamed) because the primitive's
    // contract is exactly one summary line.
    let mut child = std::process::Command::new("gh")
        .args(["pr", "checks", pr, "--watch"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            miette::miette!("failed to run gh: {e} — is gh installed and authenticated?")
        })?;

    // Drains must run CONCURRENTLY with the wait: gh's watch redraws its
    // table, and output that fills the OS pipe buffer would block gh on
    // write forever if we only drained after the wait (ro5u finding 1).
    // Each drain thread blocks on EOF — no polling; kill closes the pipes
    // so the drains terminate in the timeout branch too.
    let stdout_pipe = child.stdout.take();
    let stderr_pipe = child.stderr.take();
    let stdout_reader = std::thread::spawn(move || read_pipe(stdout_pipe));
    let stderr_reader = std::thread::spawn(move || read_pipe(stderr_pipe));

    // Deadline supervision WITHOUT a sleep-poll loop: wait_timeout blocks
    // the OS waitpid on a deadline. On exhaustion: kill, reap, then ONE
    // re-query for the last observed state.
    let outcome = match child.wait_timeout(budget) {
        Ok(Some(status)) => {
            let stdout = stdout_reader.join().unwrap_or_default();
            let stderr = stderr_reader.join().unwrap_or_default();
            crate::pipeline_wait::classify(status.code(), &stdout, &stderr)
        }
        Ok(None) => {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout_reader.join();
            let _ = stderr_reader.join();
            let last_state = requery_last_state(pr);
            crate::pipeline_wait::WaitOutcome::Timeout {
                elapsed: format_elapsed(timeout_minutes),
                last_state,
            }
        }
        Err(e) => return Err(miette::miette!("gh watch failed: {e}")),
    };

    emit_wait_outcome(pr, &outcome)
}

/// Drain a child's output pipe to a string, blocking until EOF (the child
/// exited or was killed and its pipes closed). Never polls.
fn read_pipe<R: std::io::Read>(pipe: Option<R>) -> String {
    let mut buf = String::new();
    if let Some(mut p) = pipe {
        let _ = p.read_to_string(&mut buf);
    }
    buf
}

/// One re-query of `gh pr checks <pr>` (no --watch) after a timeout, to
/// report the last observed state in the summary line. Failure to re-query
/// degrades to "unknown" — never an extra poll loop.
fn requery_last_state(pr: &str) -> String {
    match std::process::Command::new("gh")
        .args(["pr", "checks", pr])
        .output()
    {
        Ok(out) => condense_state(&String::from_utf8_lossy(&out.stdout)),
        Err(_) => "unknown".to_string(),
    }
}

/// Print the one summary line and map the outcome to the process exit:
/// pass (including zero-checks) exits 0; failure and timeout exit nonzero
/// (the primitive's exit code IS its API for scripts and agents).
fn emit_wait_outcome(pr: &str, outcome: &crate::pipeline_wait::WaitOutcome) -> Result<()> {
    use crate::pipeline_wait;
    let pr_num: u64 = pr.parse().unwrap_or(0);
    println!("{}", pipeline_wait::summary_line(pr_num, outcome));
    match pipeline_wait::exit_code(outcome) {
        0 => Ok(()),
        code => std::process::exit(code),
    }
}
