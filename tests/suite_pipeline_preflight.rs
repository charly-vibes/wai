//! Purpose: regression tests for `wai pipeline preflight` (ticket wai-gvqj).
//! Responsibilities: pin the fail-fast contract of the spawn preflight —
//! every missing tool is named in one pass with an actionable fix, a healthy
//! environment passes without mutating anything, and the epic-orchestrator
//! template requires preflight before spawning. Rationale: week
//! 2026-10-13..19 saw 7 opaque lefthook-exit-2, 11 wrong-prefix bd, and 17
//! testaruda failures in spawned worktrees (scripts/session-friction.py).

mod common;

use common::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Build a stub bin dir containing fake tool executables. Each entry is a
/// POSIX shell script with a shebang so it works under any PATH. Tools not
/// listed are simply absent from the dir (simulating "not installed" once
/// PATH points only at the stub dir).
fn stub_bin(dir: &Path, tools: &[(&str, i32, &str)]) -> std::path::PathBuf {
    let bin = dir.join("bin");
    fs::create_dir_all(&bin).unwrap();
    for (name, code, stdout) in tools {
        let script = bin.join(name);
        fs::write(
            &script,
            format!("#!/bin/sh\necho '{stdout}'\nexit {}\n", *code),
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
        }
    }
    bin
}

/// Run `wai pipeline preflight` with PATH restricted to the stub bin dir
/// (plus /usr/bin:/bin so the shell scripts themselves can spawn).
fn preflight_in(dir: &Path, bin: &Path) -> std::process::Output {
    hermetic_wai_cmd(dir)
        .args(["pipeline", "preflight"])
        .env("PATH", format!("{}:/usr/bin:/bin", bin.display()))
        .output()
        .expect("preflight should run")
}

/// Ticket wai-gvqj tasks 1.1: a worktree missing lefthook with broken bd
/// state must fail nonzero in ONE pass, naming every broken tool with an
/// actionable fix hint.
#[test]
fn preflight_reports_all_failures() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let bin = stub_bin(
        tmp.path(),
        &[("bd", 1, "boom"), ("testaruda", 0, "testaruda 0.5.7")],
    );

    let out = preflight_in(tmp.path(), &bin);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    assert!(
        !out.status.success(),
        "preflight must fail when lefthook is absent and bd is broken"
    );
    assert!(
        combined.contains("lefthook"),
        "preflight must name lefthook as missing, output: {combined}"
    );
    assert!(
        combined.contains("bd"),
        "preflight must name bd as broken, output: {combined}"
    );
    // All failures in one pass, not fail-on-first.
    assert!(
        combined.contains("lefthook") && combined.contains("bd"),
        "both failures must be reported in a single run, output: {combined}"
    );
}

/// Ticket wai-gvqj tasks 1.2: a healthy environment exits 0 and mutates
/// nothing (anti-goal: no auto-install, no store creation at preflight).
#[test]
fn preflight_passes_clean_env() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let bin = stub_bin(
        tmp.path(),
        &[
            ("lefthook", 0, "lefthook version 1.8.0"),
            ("bd", 0, "wai-123"),
            ("testaruda", 0, "testaruda 0.5.7"),
        ],
    );

    let before: Vec<std::path::PathBuf> = walk(tmp.path());
    let out = preflight_in(tmp.path(), &bin);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    assert!(
        out.status.success(),
        "preflight must pass in a healthy environment, output: {combined}"
    );
    let after: Vec<std::path::PathBuf> = walk(tmp.path());
    assert_eq!(
        before, after,
        "preflight must not create or modify files (no auto-install, no store init)"
    );
}

fn walk(root: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for e in entries.flatten() {
                out.push(e.path());
                if e.path().is_dir() {
                    stack.push(e.path());
                }
            }
        }
    }
    out.sort();
    out
}

/// Ticket wai-gvqj tasks 1.3: the epic-orchestrator template must require
/// `wai pipeline preflight` in the spawn step, so every worktree spawn
/// starts from a verified environment instead of discovering breakage
/// mid-run via opaque lefthook failures.
#[test]
fn orchestrator_template_requires_preflight() {
    let tmp = TempDir::new().unwrap();
    init_workspace(tmp.path());

    let init = hermetic_wai_cmd(tmp.path())
        .args(["pipeline", "init", "epic-orchestrator"])
        .output()
        .expect("init should run");
    assert!(init.status.success(), "init epic-orchestrator failed");

    let template = tmp
        .path()
        .join(".wai/resources/pipelines/epic-orchestrator.toml");
    let content =
        fs::read_to_string(&template).expect("epic-orchestrator template should exist after init");

    // Locate the spawn step section.
    let spawn_pos = content
        .find("id = \"spawn\"")
        .expect("template must have a spawn step");
    let next_step = content[spawn_pos..]
        .find("[[steps]]")
        .map(|i| spawn_pos + i)
        .unwrap_or(content.len());
    let spawn_section = &content[spawn_pos..next_step];

    assert!(
        spawn_section.contains("wai pipeline preflight"),
        "spawn step must require `wai pipeline preflight` before spawning; \
         spawn section:\n{spawn_section}"
    );
}
