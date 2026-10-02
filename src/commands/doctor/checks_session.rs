// Session-hook doctor checks: Claude Code and Pi agent session hooks.
//
// Extracted from mod.rs verbatim (tidy-first move, no behavior change).

use genesis::doctor::CheckStatus;

use super::WaiCheckEntry;
use std::path::{Path, PathBuf};

/// Check that Claude Code's global settings wire `wai status` into the
/// SessionStart hook. Skips (Pass) when Claude Code is not installed.
pub(super) fn check_claude_session_hook() -> Vec<WaiCheckEntry> {
    let Some(settings_path) = claude_settings_path() else {
        return vec![claude_entry(
            CheckStatus::Pass,
            "Could not determine home directory — skipping Claude Code hook check",
        )];
    };
    if !settings_path.exists() {
        return vec![claude_entry(
            CheckStatus::Pass,
            "Claude Code not installed — skipping hook check",
        )];
    }
    let content = match std::fs::read_to_string(&settings_path) {
        Ok(c) => c,
        Err(e) => {
            return vec![claude_entry(
                CheckStatus::Warn,
                format!("Cannot read ~/.claude/settings.json: {e}"),
            )];
        }
    };
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            return vec![claude_entry(
                CheckStatus::Warn,
                format!("~/.claude/settings.json is not valid JSON: {e}"),
            )];
        }
    };
    if claude_settings_have_hook(&json) {
        vec![claude_entry(
            CheckStatus::Pass,
            "`wai status` is in the SessionStart hook",
        )]
    } else {
        vec![claude_hook_missing_entry(settings_path)]
    }
}

/// Path to Claude Code's global settings file, when the home dir is known.
fn claude_settings_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|home| home.join(".claude").join("settings.json"))
}

/// Build a Claude Code session-hook entry with the standard name.
fn claude_entry(status: CheckStatus, message: impl std::fmt::Display) -> WaiCheckEntry {
    WaiCheckEntry {
        name: "Claude Code session hook".to_string(),
        status,
        message: message.to_string(),
        fix: None,
        fix_fn: None,
    }
}

/// Whether any SessionStart hook command contains "wai status".
fn claude_settings_have_hook(json: &serde_json::Value) -> bool {
    json.get("hooks")
        .and_then(|h| h.get("SessionStart"))
        .and_then(|s| s.as_array())
        .map(|entries| {
            entries.iter().any(|entry| {
                entry
                    .get("hooks")
                    .and_then(|h| h.as_array())
                    .map(|hooks| {
                        hooks.iter().any(|hook| {
                            hook.get("command")
                                .and_then(|c| c.as_str())
                                .map(|cmd| cmd.contains("wai status"))
                                .unwrap_or(false)
                        })
                    })
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// Warn that the hook is missing; the fix inserts it into settings.json.
fn claude_hook_missing_entry(settings_path: std::path::PathBuf) -> WaiCheckEntry {
    WaiCheckEntry {
        name: "Claude Code session hook".to_string(),
        status: CheckStatus::Warn,
        message: "`wai status` not found in ~/.claude/settings.json SessionStart hooks"
            .to_string(),
        fix: Some(
            r#"Add to ~/.claude/settings.json hooks.SessionStart: {"matcher":"","hooks":[{"type":"command","command":"wai status 2>/dev/null || true"}]}"#
                .to_string(),
        ),
        fix_fn: Some(Box::new(move |_project_root| {
            install_claude_session_hook(&settings_path)
        })),
    }
}

/// Insert the `wai status` SessionStart hook into Claude Code settings.
fn install_claude_session_hook(settings_path: &std::path::Path) -> miette::Result<()> {
    use miette::IntoDiagnostic;

    let content = std::fs::read_to_string(settings_path).into_diagnostic()?;
    let mut json: serde_json::Value = serde_json::from_str(&content).into_diagnostic()?;

    let new_hook = serde_json::json!({
        "matcher": "",
        "hooks": [{"type": "command", "command": "wai status 2>/dev/null || true"}]
    });

    // Ensure hooks.SessionStart exists as an array, then push
    let session_start = json
        .get_mut("hooks")
        .and_then(|h| h.get_mut("SessionStart"))
        .and_then(|s| s.as_array_mut());

    if let Some(arr) = session_start {
        arr.push(new_hook);
    } else {
        // Build hooks.SessionStart from scratch, preserving other hooks
        let hooks = json.get_mut("hooks").and_then(|h| h.as_object_mut());

        if let Some(hooks_obj) = hooks {
            hooks_obj.insert("SessionStart".to_string(), serde_json::json!([new_hook]));
        } else {
            json["hooks"] = serde_json::json!({
                "SessionStart": [new_hook]
            });
        }
    }

    let updated = serde_json::to_string_pretty(&json).into_diagnostic()?;
    std::fs::write(settings_path, updated).into_diagnostic()?;
    Ok(())
}

// ─── pi session hook ─────────────────────────────────────────────────────────

/// Recommended fix text: a minimal pi extension that runs `wai prime` on
/// `session_start`. Shown when the check warns.
const PI_WAI_PRIME_EXTENSION_FIX: &str = r#"Create .pi/extensions/wai-prime.ts (or ~/.pi/agent/extensions/wai-prime.ts) with:

import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
export default function (pi: ExtensionAPI) {
  pi.on("session_start", async () => { await pi.exec("wai", ["prime"]); });
}"#;

/// True when any of the given pi extension file contents registers a
/// `session_start` handler that runs `wai prime` (or `wai status`).
///
/// Matches two forms an extension may use to invoke wai:
/// - shell-string: a `wai prime` / `wai status` substring (e.g. `bash -c "wai prime`)
/// - argv: quoted `"wai"` alongside `"prime"` / `"status"` (e.g. `pi.exec("wai", ["prime"])`)
///
/// Pure scanner so the logic can be unit-tested without touching the filesystem.
pub(super) fn pi_extensions_run_wai_prime(contents: &[String]) -> bool {
    contents.iter().any(|c| {
        if !c.contains("session_start") {
            return false;
        }
        let shell_form = c.contains("wai prime") || c.contains("wai status");
        let argv_form =
            c.contains("\"wai\"") && (c.contains("\"prime\"") || c.contains("\"status\""));
        shell_form || argv_form
    })
}

/// Collect auto-discovered pi extension files from a directory:
/// `*.ts` and `*/index.ts`. Returns paths that exist and are readable.
fn collect_pi_extension_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("ts") {
            files.push(path);
        } else if path.is_dir() {
            let idx = path.join("index.ts");
            if idx.is_file() {
                files.push(idx);
            }
        }
    }
    files
}

/// Check whether the current project runs `wai prime` at pi's `session_start`.
///
/// Fires only when the project opts into pi via a `.pi/` directory (project-local
/// config or extensions). Systems without pi, or projects that don't use pi, see
/// no output from this check — it is omitted rather than warning, so a clean
/// `wai init` workspace stays green regardless of the host's global pi install.
///
/// When pi is in use, scans `.pi/extensions/` for a `session_start` extension
/// running `wai prime`/`wai status` and warns with an actionable fix otherwise.
pub(super) fn check_pi_session_hook(project_root: &Path) -> Vec<WaiCheckEntry> {
    let pi_dir = project_root.join(".pi");
    if !pi_dir.exists() {
        return Vec::new();
    }

    let ext_dir = pi_dir.join("extensions");
    let files = collect_pi_extension_files(&ext_dir);
    let contents: Vec<String> = files
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect();

    if pi_extensions_run_wai_prime(&contents) {
        vec![WaiCheckEntry {
            name: "Pi session hook".to_string(),
            status: CheckStatus::Pass,
            message: "`wai prime` runs at pi `session_start` (via .pi/extensions)".to_string(),
            fix: None,
            fix_fn: None,
        }]
    } else {
        vec![WaiCheckEntry {
            name: "Pi session hook".to_string(),
            status: CheckStatus::Warn,
            message: "No pi `session_start` extension runs `wai prime` in .pi/extensions"
                .to_string(),
            fix: Some(PI_WAI_PRIME_EXTENSION_FIX.to_string()),
            fix_fn: None,
        }]
    }
}

/// Read a file''s content excluding the WAI managed block (between <!-- WAI:START --> and <!-- WAI:END -->).
/// Returns empty string if the file doesn't exist.
pub(super) fn read_non_managed_block_content(path: &Path) -> String {
    let Ok(content) = std::fs::read_to_string(path) else {
        return String::new();
    };
    // Strip managed block
    if let Some(start) = content.find("<!-- WAI:START -->") {
        let after_start = start + "<!-- WAI:START -->".len();
        if let Some(end) = content[after_start..].find("<!-- WAI:END -->") {
            let end_pos = after_start + end + "<!-- WAI:END -->".len();
            let before = &content[..start];
            let after = &content[end_pos..];
            return format!("{}\n{}", before, after);
        }
    }
    content
}
