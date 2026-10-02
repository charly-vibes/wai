use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use serde::Serialize;
use std::path::Path;

use genesis::doctor::{CheckStatus, DoctorCheck, DoctorReport, DoctorSummary};
use genesis::suite_linter::{LintResult, Severity};

use crate::context::current_context;
use crate::output::print_envelope_doctor;

use super::require_project;

mod checks_basic;
mod checks_blocks;
mod checks_pipeline;
mod checks_session;
mod checks_sync;
mod checks_workspace;

use checks_blocks::{check_agent_instructions, check_managed_block_staleness};
use checks_pipeline::{
    check_artifact_locks, check_dont_drift_signals, check_pipeline_definitions,
    check_pipeline_utilization,
};
use checks_session::{check_claude_session_hook, check_pi_session_hook};
use checks_workspace::{check_agent_tool_coverage, check_skills_in_repo};

/// Internal check result with a fix closure.
///
/// The public doctor API uses `genesis::doctor::CheckEntry` / `DoctorReport`;
/// this internal wrapper carries the optional fix closure that wai applies
/// via `--fix`. It converts losslessly to a genesis `CheckEntry`.
pub(super) struct WaiCheckEntry {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    pub fix: Option<String>,
    #[allow(clippy::type_complexity)]
    pub fix_fn: Option<Box<dyn FnOnce(&Path) -> Result<()>>>,
}

/// Adapter wrapping a wai check function as a genesis::doctor::DoctorCheck.
///
/// The adapter converts WaiCheckEntry results to LintResults for the
/// DoctorRunner, skipping pass entries and mapping Warn/Fail to
/// Warning/Error severity.
struct WaiCheckAdapter {
    name: &'static str,
    description: &'static str,
    func: fn(&Path) -> Vec<WaiCheckEntry>,
}

impl DoctorCheck for WaiCheckAdapter {
    fn name(&self) -> &'static str {
        self.name
    }

    fn description(&self) -> &'static str {
        self.description
    }

    fn run(
        &self,
        repo_root: &Path,
    ) -> std::result::Result<Vec<LintResult>, Box<dyn std::error::Error>> {
        let results = (self.func)(repo_root);
        let lint_results: Vec<LintResult> = results
            .into_iter()
            .filter(|w| w.status != CheckStatus::Pass)
            .map(|w| {
                let severity = match w.status {
                    CheckStatus::Pass => unreachable!(),
                    CheckStatus::Warn => Severity::Warning,
                    CheckStatus::Fail => Severity::Error,
                };
                let mut lr = LintResult::new(w.message, severity);
                if let Some(fix) = w.fix {
                    lr = LintResult::with_fix(lr.message, lr.severity, &fix);
                }
                lr
            })
            .collect();
        Ok(lint_results)
    }
}

/// A doctor check registration: (name, description, check function).
type CheckEntry = (&'static str, &'static str, fn(&Path) -> Vec<WaiCheckEntry>);

/// The doctor check registry — single source of truth for the check set.
const DOCTOR_CHECKS: &[CheckEntry] = &[
    (
        "directory-structure",
        "Check that all .wai/ PARA directories exist",
        |root| checks_basic::check_directories(root),
    ),
    ("config", "Check that config.toml is valid", |root| {
        vec![checks_basic::check_config(root)]
    }),
    (
        "version",
        "Check that workspace version matches binary",
        |root| vec![checks_basic::check_version(root)],
    ),
    (
        "plugin-tools",
        "Check that detected plugin tools are available on PATH",
        |root| checks_basic::check_plugin_tools(root),
    ),
    (
        "agent-config-sync",
        "Check that agent config projections are in sync",
        |root| checks_sync::check_agent_config_sync(root),
    ),
    (
        "skills-in-repo",
        "Check that skills directory exists",
        |root| check_skills_in_repo(root),
    ),
    (
        "agent-tool-coverage",
        "Check for known agent tool directories",
        |root| check_agent_tool_coverage(root),
    ),
    (
        "project-state",
        "Check project state file integrity",
        |root| checks_basic::check_project_state(root),
    ),
    (
        "custom-plugins",
        "Check custom plugin definitions",
        |root| checks_basic::check_custom_plugins(root),
    ),
    (
        "agent-instructions",
        "Check that AGENTS.md / CLAUDE.md contain required managed blocks",
        |root| check_agent_instructions(root),
    ),
    (
        "managed-block-staleness",
        "Check that managed blocks are up to date",
        |root| check_managed_block_staleness(root),
    ),
    (
        "pipeline-definitions",
        "Check pipeline definitions for issues",
        |root| check_pipeline_definitions(root),
    ),
    ("readme-badge", "Check that README has wai badge", |root| {
        checks_basic::check_readme_badge(root)
    }),
    (
        "badge-block-consistency",
        "Check that badge and managed block agree",
        |root| checks_basic::check_badge_managed_block_consistency(root),
    ),
    ("suite-gates", "Check suite gate configuration", |root| {
        checks_basic::check_suite_gates(root)
    }),
    (
        "claude-session-hook",
        "Check Claude Code session hook for wai status",
        |_| check_claude_session_hook(),
    ),
    (
        "wai-project-env",
        "Check WAI_PROJECT environment variable",
        |root| checks_basic::check_wai_project_env(root),
    ),
    (
        "artifact-locks",
        "Check artifact lock file integrity",
        |root| check_artifact_locks(root),
    ),
    (
        "dont-drift-signals",
        "Check dont drift signal files",
        |root| check_dont_drift_signals(root),
    ),
    (
        "pipeline-utilization",
        "Check pipeline run utilization",
        |root| check_pipeline_utilization(root),
    ),
    ("pi-session-hook", "Check Pi agent session hook", |root| {
        check_pi_session_hook(root)
    }),
];

/// Build a DoctorRunner from all registered wai check functions.
pub(crate) fn build_doctor_runner() -> genesis::doctor::DoctorRunner {
    let checks: Vec<Box<dyn DoctorCheck>> = DOCTOR_CHECKS
        .iter()
        .map(|(name, description, func)| {
            Box::new(WaiCheckAdapter {
                name,
                description,
                func: *func,
            }) as Box<dyn DoctorCheck>
        })
        .collect();
    genesis::doctor::DoctorRunner::new(checks)
}

/// Collect every doctor `WaiCheckEntry` for the given project root.
///
/// This is the single source of truth for the check set — both `wai doctor`
/// (via [`run`]) and the inline health summaries (via [`health_summary`]) run
/// exactly the same diagnostics, so the inline line never disagrees with the
/// full report.
fn collect_checks(project_root: &Path) -> Vec<WaiCheckEntry> {
    DOCTOR_CHECKS
        .iter()
        .flat_map(|(_, _, check)| check(project_root))
        .collect()
}

/// Run the doctor checks and return a compact warn/fail report.
///
/// Used by `wai status` and `wai prime` to surface a one-line health summary
/// when the workspace is not fully green. Returns `is_healthy() == true` when
/// nothing is wrong, so callers can stay silent in that case.
pub fn health_summary(project_root: &Path) -> DoctorReport {
    let runner = build_doctor_runner();
    runner
        .run(project_root, false)
        .unwrap_or_else(|_| DoctorReport::new("wai", vec![]))
}

pub fn run(fix: bool) -> Result<()> {
    let project_root = require_project()?;
    let context = current_context();

    let checks = collect_checks(&project_root);

    let summary = DoctorSummary {
        pass: checks
            .iter()
            .filter(|c| c.status == CheckStatus::Pass)
            .count(),
        warn: checks
            .iter()
            .filter(|c| c.status == CheckStatus::Warn)
            .count(),
        fail: checks
            .iter()
            .filter(|c| c.status == CheckStatus::Fail)
            .count(),
    };

    // Handle fix mode vs diagnostic mode
    if fix {
        // In fix mode, show diagnostics first (if human mode), then apply fixes
        if !context.json {
            render_human(&checks, &summary)?;
        }
        apply_fixes(&project_root, checks, &context)?;
    } else {
        // In diagnostic mode, use DoctorRunner for JSON, render_human for tty
        if context.json {
            let runner = build_doctor_runner();
            let report = runner
                .run(&project_root, false)
                .map_err(|e| miette::miette!("{}", e))?;
            crate::output::print_json(&report.to_envelope(env!("CARGO_PKG_VERSION")))?;
        } else {
            render_human(&checks, &summary)?;
        }

        if summary.fail > 0 {
            std::process::exit(1);
        }
    }

    Ok(())
}

fn apply_fixes(
    project_root: &Path,
    mut checks: Vec<WaiCheckEntry>,
    context: &crate::context::CliContext,
) -> Result<()> {
    use crate::error::WaiError;

    // Refuse in safe mode
    if context.safe {
        return Err(WaiError::SafeModeViolation {
            action: "apply doctor fixes".to_string(),
        }
        .into());
    }

    // Filter to fixable checks
    let fixable_checks: Vec<WaiCheckEntry> =
        checks.drain(..).filter(|c| c.fix_fn.is_some()).collect();

    if fixable_checks.is_empty() {
        if !context.json {
            use cliclack::log;
            log::info("No fixable issues found").into_diagnostic()?;
        }
        return Ok(());
    }

    if !confirm_fixes(fixable_checks.len(), context)? {
        return Ok(());
    }

    let (fixes_applied, fixes_failed) = run_fixes(project_root, fixable_checks);
    report_fix_results(context, &fixes_applied, &fixes_failed)?;

    // Exit with appropriate code
    if !fixes_failed.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}

/// Confirm fix application unless --yes, --no-input, or --json.
fn confirm_fixes(count: usize, context: &crate::context::CliContext) -> Result<bool> {
    if context.json || context.no_input || context.yes {
        return Ok(true);
    }
    if context.json {
        return Ok(true);
    }
    use cliclack::confirm;
    confirm(format!("Apply {count} fix(es)?"))
        .interact()
        .into_diagnostic()
}

/// Run every fixable check's fix closure, collecting per-fix outcomes.
fn run_fixes(
    project_root: &Path,
    fixable_checks: Vec<WaiCheckEntry>,
) -> (Vec<FixResult>, Vec<FixResult>) {
    let mut fixes_applied = Vec::new();
    let mut fixes_failed = Vec::new();

    for mut check in fixable_checks {
        if let Some(fix_fn) = check.fix_fn.take() {
            match fix_fn(project_root) {
                Ok(()) => fixes_applied.push(FixResult {
                    name: check.name.clone(),
                    success: true,
                    error: None,
                }),
                Err(e) => fixes_failed.push(FixResult {
                    name: check.name.clone(),
                    success: false,
                    error: Some(e.to_string()),
                }),
            }
        }
    }

    (fixes_applied, fixes_failed)
}

/// Render fix outcomes: JSON envelope in JSON mode, cliclack logs otherwise.
fn report_fix_results(
    context: &crate::context::CliContext,
    fixes_applied: &[FixResult],
    fixes_failed: &[FixResult],
) -> Result<()> {
    if context.json {
        #[derive(Serialize)]
        struct FixPayload {
            fixes_applied: Vec<FixResult>,
            fixes_failed: Vec<FixResult>,
        }

        let payload = FixPayload {
            fixes_applied: fixes_applied.to_vec(),
            fixes_failed: fixes_failed.to_vec(),
        };
        print_envelope_doctor(payload)?;
        return Ok(());
    }

    use cliclack::log;
    println!();
    for fix in fixes_applied {
        log::success(format!("Fixed: {}", fix.name)).into_diagnostic()?;
    }
    for fix in fixes_failed {
        log::error(format!(
            "Failed to fix {}: {}",
            fix.name,
            fix.error.as_ref().unwrap_or(&"unknown error".to_string())
        ))
        .into_diagnostic()?;
    }
    println!();

    use cliclack::outro;
    if fixes_failed.is_empty() {
        outro("All fixes applied. Re-run 'wai doctor' to verify.").into_diagnostic()?;
    } else {
        outro("Some fixes failed. Re-run 'wai doctor' to check status.").into_diagnostic()?;
    }
    Ok(())
}

#[derive(Serialize, Clone)]
struct FixResult {
    name: String,
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

fn render_human(checks: &[WaiCheckEntry], summary: &DoctorSummary) -> Result<()> {
    use cliclack::outro;

    println!();
    println!("  {} Workspace Health", "◆".cyan());
    println!(
        "  {} For repo hygiene and agent workflow conventions, run 'wai way'",
        "·".dimmed()
    );
    println!();

    for check in checks {
        let icon = match check.status {
            CheckStatus::Pass => "✓".green().to_string(),
            CheckStatus::Warn => "⚠".yellow().to_string(),
            CheckStatus::Fail => "✗".red().to_string(),
        };
        println!("  {} {}: {}", icon, check.name.bold(), check.message);
        if let Some(ref fix) = check.fix {
            println!("    {} {}", "→".dimmed(), fix.dimmed());
        }
    }

    println!();
    let summary_line = format!(
        "{} passed, {} warnings, {} failed",
        summary.pass, summary.warn, summary.fail
    );
    if summary.fail > 0 {
        outro(summary_line.red().to_string()).into_diagnostic()?;
    } else if summary.warn > 0 {
        outro(summary_line.yellow().to_string()).into_diagnostic()?;
    } else {
        outro(summary_line.green().to_string()).into_diagnostic()?;
    }

    Ok(())
}
#[cfg(test)]
mod tests {
    use super::checks_pipeline::drift_signals_to_check_results;
    use super::checks_session::pi_extensions_run_wai_prime;
    use super::*;
    use tempfile::TempDir;

    /// Helper: create a minimal wai workspace with `.wai/projects/` directory.
    fn setup_workspace() -> TempDir {
        let tmp = TempDir::new().expect("create tempdir");
        let projects = tmp.path().join(".wai").join("projects").join("test-proj");
        std::fs::create_dir_all(&projects).expect("create projects dir");
        tmp
    }

    /// Helper: write an artifact file and a matching lock sidecar.
    fn write_artifact_and_lock(
        dir: &Path,
        artifact_name: &str,
        content: &str,
        run_id: &str,
        tamper: bool,
    ) {
        use super::super::pipeline::artifact_hash;

        let artifact_path = dir.join(artifact_name);
        std::fs::write(&artifact_path, content).expect("write artifact");

        let hash = if tamper {
            "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_string()
        } else {
            artifact_hash(&artifact_path).expect("hash artifact")
        };

        let lock = toml::to_string_pretty(&super::super::pipeline::ArtifactLock {
            artifact: artifact_name.to_string(),
            locked_at: "2026-01-01T00:00:00Z".to_string(),
            lock_hash: hash,
            pipeline_run: run_id.to_string(),
            pipeline_step: "step-1".to_string(),
        })
        .expect("serialize lock");

        let lock_name = format!("{}.{}.lock", artifact_name, run_id);
        std::fs::write(dir.join(lock_name), lock).expect("write lock");
    }

    #[test]
    fn no_lock_files_returns_empty() {
        let tmp = setup_workspace();
        let results = check_artifact_locks(tmp.path());
        assert!(results.is_empty());
    }

    #[test]
    fn valid_lock_returns_pass() {
        let tmp = setup_workspace();
        let proj_dir = tmp.path().join(".wai/projects/test-proj");
        write_artifact_and_lock(
            &proj_dir,
            "research.md",
            "# Research\nFindings here\n",
            "run-1",
            false,
        );

        let results = check_artifact_locks(tmp.path());
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, CheckStatus::Pass);
        assert!(results[0].message.contains("1 locked artifacts verified"));
    }

    #[test]
    fn tampered_artifact_returns_warn() {
        let tmp = setup_workspace();
        let proj_dir = tmp.path().join(".wai/projects/test-proj");
        write_artifact_and_lock(&proj_dir, "design.md", "# Design\nChoices\n", "run-2", true);

        let results = check_artifact_locks(tmp.path());
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, CheckStatus::Warn);
        assert!(results[0].message.contains("Hash mismatch"));
    }

    #[test]
    fn missing_artifact_returns_warn() {
        let tmp = setup_workspace();
        let proj_dir = tmp.path().join(".wai/projects/test-proj");
        // Write only the lock, not the artifact
        let lock = toml::to_string_pretty(&super::super::pipeline::ArtifactLock {
            artifact: "ghost.md".to_string(),
            locked_at: "2026-01-01T00:00:00Z".to_string(),
            lock_hash: "sha256:abc".to_string(),
            pipeline_run: "run-3".to_string(),
            pipeline_step: "step-1".to_string(),
        })
        .expect("serialize lock");
        std::fs::write(proj_dir.join("ghost.md.run-3.lock"), lock).expect("write lock");

        let results = check_artifact_locks(tmp.path());
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, CheckStatus::Warn);
        assert!(results[0].message.contains("Locked artifact missing"));
    }

    #[test]
    fn mixed_valid_and_tampered() {
        let tmp = setup_workspace();
        let proj_dir = tmp.path().join(".wai/projects/test-proj");
        write_artifact_and_lock(&proj_dir, "good.md", "valid content", "run-4", false);
        write_artifact_and_lock(&proj_dir, "bad.md", "tampered content", "run-4", true);

        let results = check_artifact_locks(tmp.path());
        // Should only contain the mismatch warning (pass is suppressed when mismatches exist)
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, CheckStatus::Warn);
        assert!(results[0].message.contains("Hash mismatch"));
        assert!(results[0].message.contains("bad.md"));
    }

    // --- dont drift signal tests ---

    #[test]
    fn drift_signals_empty_json_array_returns_pass() {
        let results = drift_signals_to_check_results(b"[]");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, CheckStatus::Pass);
        assert!(results[0].message.contains("No dont rejection signals"));
    }

    #[test]
    fn drift_signals_with_one_signal_returns_warn() {
        let json = serde_json::to_vec(&serde_json::json!([{
            "schema_version": "1.0",
            "signal_kind": "dont-rejection",
            "source_tool": "dont",
            "rule_name": "ungrounded",
            "timestamp": "2026-06-10T12:00:00Z",
            "violation_count": 2,
            "violations": [],
            "openspec_hint": "spec drift detected"
        }]))
        .unwrap();
        let results = drift_signals_to_check_results(&json);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, CheckStatus::Warn);
        assert!(results[0].message.contains("1 dont rejection signal"));
        assert!(results[0].message.contains("ungrounded"));
    }

    #[test]
    fn drift_signals_malformed_json_returns_empty() {
        let results = drift_signals_to_check_results(b"not json");
        assert!(results.is_empty());
    }

    #[test]
    fn check_dont_drift_signals_skips_when_env_absent() {
        // Relies on WAI_DONT_SIGNALS not being set in the test environment.
        if std::env::var("WAI_DONT_SIGNALS").is_ok() {
            return;
        }
        let tmp = TempDir::new().unwrap();
        let results = check_dont_drift_signals(tmp.path());
        assert!(results.is_empty());
    }

    // --- pi session hook tests ---

    #[test]
    fn pi_extensions_detect_wai_prime_in_session_start() {
        let contents = vec![
            "import type { ExtensionAPI } from \"@earendil-works/pi-coding-agent\";

export default function (pi: ExtensionAPI) {
  pi.on(\"session_start\", async (_e, _ctx) => {
    await pi.exec(\"wai\", [\"prime\"]);
  });
}
"
            .to_string(),
        ];
        assert!(pi_extensions_run_wai_prime(&contents));
    }

    #[test]
    fn pi_extensions_detect_wai_status_in_session_start() {
        let contents = vec![
            "pi.on(\"session_start\", async () => { await pi.exec(\"wai\", [\"status\"]); });\n"
                .to_string(),
        ];
        assert!(pi_extensions_run_wai_prime(&contents));
    }

    #[test]
    fn pi_extensions_ignore_session_start_without_wai() {
        let contents = vec![
            "pi.on(\"session_start\", async (_e, ctx) => { ctx.ui.notify(\"hi\"); });\n"
                .to_string(),
        ];
        assert!(!pi_extensions_run_wai_prime(&contents));
    }

    #[test]
    fn pi_extensions_ignore_wai_without_session_start() {
        // An extension that runs `wai prime` on a different event (e.g. a command)
        // does not satisfy the session-start check.
        let contents = vec![
            "pi.registerCommand(\"orient\", async () => { await pi.exec(\"wai\", [\"prime\"]); });\n".to_string(),
        ];
        assert!(!pi_extensions_run_wai_prime(&contents));
    }

    #[test]
    fn check_pi_session_hook_omitted_when_no_pi_dir() {
        // No `.pi/` in the project → the check is omitted (pi not in use here).
        let tmp = TempDir::new().unwrap();
        let results = check_pi_session_hook(tmp.path());
        assert!(results.is_empty());
    }

    #[test]
    fn check_pi_session_hook_warns_when_pi_present_without_wai_extension() {
        let tmp = TempDir::new().unwrap();
        // Project opts into pi but has no wai-prime session_start extension.
        std::fs::create_dir_all(tmp.path().join(".pi/extensions")).unwrap();
        std::fs::write(
            tmp.path().join(".pi/extensions/other.ts"),
            "pi.on(\"session_start\", async (_e, ctx) => { ctx.ui.notify(\"hi\"); });\n",
        )
        .unwrap();
        let results = check_pi_session_hook(tmp.path());
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, CheckStatus::Warn);
        assert!(
            results[0]
                .fix
                .as_deref()
                .unwrap_or("")
                .contains("session_start")
        );
    }

    #[test]
    fn check_pi_session_hook_passes_when_wai_prime_extension_present() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".pi/extensions")).unwrap();
        std::fs::write(
            tmp.path().join(".pi/extensions/wai-prime.ts"),
            "pi.on(\"session_start\", async () => { await pi.exec(\"wai\", [\"prime\"]); });\n",
        )
        .unwrap();
        let results = check_pi_session_hook(tmp.path());
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, CheckStatus::Pass);
    }

    // ── provenance footer drift ──────────────────────────────────────

    use crate::managed_block::inject_managed_block;

    /// Inject the managed block and return the path + the inner text
    /// between the WAI markers.
    fn setup_injected_block() -> (TempDir, std::path::PathBuf, String) {
        let tmp = setup_workspace();
        let path = tmp.path().join("AGENTS.md");
        inject_managed_block(&path, &[], &[], &[]).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        let start = content.find("<!-- WAI:START -->").unwrap() + "<!-- WAI:START -->".len();
        let end = content.find("<!-- WAI:END -->").unwrap();
        (tmp, path, content[start..end].to_string())
    }

    fn staleness_entries(path: &std::path::Path, filename: &str) -> Vec<WaiCheckEntry> {
        check_managed_block_staleness(path.parent().unwrap())
            .into_iter()
            .filter(|e| e.name == format!("Managed block staleness: {}", filename))
            .collect()
    }

    #[test]
    fn footer_version_change_does_not_flag_stale() {
        let (tmp, path, inner) = setup_injected_block();
        // Hand-edit the footer's version field only (simulates a wai release
        // that bumps the footer text without changing content).
        let footer_line = inner
            .lines()
            .find(|l| l.contains("<!-- provenance: "))
            .expect("footer present");
        let version_start = footer_line.find("version=").unwrap() + "version=".len();
        let version_end = footer_line[version_start..]
            .find(|c: char| c.is_whitespace())
            .map(|i| version_start + i)
            .unwrap_or(footer_line.len());
        let edited = inner.replace(&footer_line[version_start..version_end], "99.99.99");
        let content = std::fs::read_to_string(&path)
            .unwrap()
            .replace(&inner, &edited);
        std::fs::write(&path, content).unwrap();

        let entries = staleness_entries(&path, "AGENTS.md");
        assert!(
            entries.is_empty(),
            "version-only footer change must not flag staleness, got: {:?}",
            entries.iter().map(|e| &e.message).collect::<Vec<_>>()
        );
        drop(tmp);
    }

    #[test]
    fn footer_sha_corruption_flags_drift() {
        let (tmp, path, inner) = setup_injected_block();
        // Corrupt only the sha field — body untouched. The footer hash no
        // longer matches the body: post-init tampering signal.
        let corrupted = inner.replace("sha=", "sha=deadbeef");
        let content = std::fs::read_to_string(&path)
            .unwrap()
            .replace(&inner, &corrupted);
        std::fs::write(&path, content).unwrap();

        let entries = staleness_entries(&path, "AGENTS.md");
        assert_eq!(entries.len(), 1, "corrupted footer sha must flag drift");
        assert_eq!(entries[0].status, CheckStatus::Warn);
        assert!(entries[0].message.contains("drift"));
        drop(tmp);
    }

    #[test]
    fn edited_block_body_flags_stale() {
        let (tmp, path, inner) = setup_injected_block();
        // Edit the body content (keep footer intact).
        let edited = inner.replace("wai sync", "wai sync EDITED");
        assert_ne!(edited, inner, "edit target must exist in block");
        let content = std::fs::read_to_string(&path)
            .unwrap()
            .replace(&inner, &edited);
        std::fs::write(&path, content).unwrap();

        let entries = staleness_entries(&path, "AGENTS.md");
        assert_eq!(entries.len(), 1, "edited body must flag staleness");
        assert_eq!(entries[0].status, CheckStatus::Warn);
        drop(tmp);
    }

    #[test]
    fn freshly_injected_block_is_not_stale() {
        let (tmp, path, _inner) = setup_injected_block();
        let entries = staleness_entries(&path, "AGENTS.md");
        assert!(
            entries.is_empty(),
            "fresh injection must not be stale, got: {:?}",
            entries.iter().map(|e| &e.message).collect::<Vec<_>>()
        );
        drop(tmp);
    }
}
