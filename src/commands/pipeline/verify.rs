//! Artifact lock verification, hashing, and lock file I/O.

use miette::{IntoDiagnostic, Result};
use std::fs;
use std::path::Path;

use crate::commands::require_project;

use super::model::ArtifactLock;

pub(super) fn cmd_verify() -> Result<()> {
    let project_root = require_project()?;
    let projects_dir = crate::config::projects_dir(&project_root);

    // Walk all .lock files under .wai/projects/
    let mut lock_count: usize = 0;
    let mut mismatches: Vec<String> = Vec::new();

    for entry in walkdir::WalkDir::new(&projects_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("lock") {
            continue;
        }

        let lock = read_artifact_lock(path)?;
        let artifact_path = path.parent().unwrap().join(&lock.artifact);

        if !artifact_path.exists() {
            mismatches.push(format!(
                "  {} — artifact missing (expected at {})",
                lock.artifact,
                artifact_path.display()
            ));
            lock_count += 1;
            continue;
        }

        let actual_hash = artifact_hash(&artifact_path)?;
        if actual_hash != lock.lock_hash {
            mismatches.push(format!(
                "  {} — expected {}, got {}",
                lock.artifact, lock.lock_hash, actual_hash
            ));
        }

        lock_count += 1;
    }

    report_verify_outcome(lock_count, &mismatches);
    Ok(())
}

/// Emit the verify summary; exits the process with status 1 on mismatch.
fn report_verify_outcome(lock_count: usize, mismatches: &[String]) {
    if lock_count == 0 {
        cliclack::log::info("No locked artifacts found.").into_diagnostic().ok();
        return;
    }

    if mismatches.is_empty() {
        let _ = cliclack::log::success(format!("All {} locked artifacts verified.", lock_count))
            .into_diagnostic();
    } else {
        let header = format!(
            "{} of {} locked artifacts failed verification:",
            mismatches.len(),
            lock_count
        );
        let detail = mismatches.join("\n");
        let _ = cliclack::log::error(format!("{}\n{}", header, detail)).into_diagnostic();
        std::process::exit(1);
    }
}

// ─── Artifact lock helpers ────────────────────────────────────────────────────

/// Substitute `{topic}` in a prompt string with the given topic value.
///
/// If the prompt contains no `{topic}` placeholder, the prompt is returned unchanged.
pub fn render_prompt(prompt: &str, topic: &str) -> String {
    prompt.replace("{topic}", topic)
}

/// Compute SHA-256 hash of an artifact file, normalizing line endings to LF.
///
/// Returns the hash as a hex string prefixed with "sha256:".
#[allow(dead_code)] // Used by pipeline lock/verify commands (wai-2lwo, wai-rdp5)
pub fn artifact_hash(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let content = fs::read_to_string(path).into_diagnostic()?;
    let normalized = content.replace("\r\n", "\n");
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    let hash = hasher.finalize();
    Ok(format!("sha256:{:x}", hash))
}

/// Write a `.lock` sidecar file for an artifact.
///
/// The lock file is named `<artifact>.<run-id>.lock` and placed alongside the artifact.
/// Returns the path to the lock file.
#[allow(dead_code)] // Used by pipeline lock commands (wai-yh5v, wai-bzrp)
pub fn write_artifact_lock(
    artifact_path: &Path,
    run_id: &str,
    step_id: &str,
) -> Result<std::path::PathBuf> {
    let hash = artifact_hash(artifact_path)?;
    let lock = ArtifactLock {
        artifact: artifact_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string(),
        locked_at: chrono::Utc::now().to_rfc3339(),
        lock_hash: hash,
        pipeline_run: run_id.to_string(),
        pipeline_step: step_id.to_string(),
    };
    let lock_filename = format!(
        "{}.{}.lock",
        artifact_path.file_name().unwrap().to_string_lossy(),
        run_id
    );
    let lock_path = artifact_path.parent().unwrap().join(&lock_filename);
    let toml_content = toml::to_string_pretty(&lock).into_diagnostic()?;
    fs::write(&lock_path, toml_content).into_diagnostic()?;
    Ok(lock_path)
}

/// Read and parse a `.lock` sidecar file.
#[allow(dead_code)] // Used by pipeline lock/verify commands (wai-yh5v, wai-rdp5)
pub fn read_artifact_lock(lock_path: &Path) -> Result<ArtifactLock> {
    let content = fs::read_to_string(lock_path).into_diagnostic()?;
    toml::from_str(&content).into_diagnostic()
}

// ─── Unit tests ───────────────────────────────────────────────────────────────

