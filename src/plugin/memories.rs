//! Memory fetch/store helpers and main-worktree detection.

use super::*;

pub const MEMORIES_BUDGET: usize = 10_000;

/// Fetch all bd memories by shelling out to `bd memories`.
///
/// Returns `None` if beads is not detected (no `.beads/` directory), if `bd`
/// is not on PATH, or if the command fails. Callers must degrade gracefully.
pub fn fetch_memories(project_root: &Path) -> Option<String> {
    if !project_root.join(".beads").exists() {
        return None;
    }
    let output = Command::new("bd")
        .arg("memories")
        .current_dir(project_root)
        .output()
        .ok()?;
    if !output.status.success() || output.stdout.is_empty() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    if text.len() <= MEMORIES_BUDGET {
        Some(text)
    } else {
        // Truncate at a char boundary to stay within budget.
        let mut cut = MEMORIES_BUDGET;
        while cut > 0 && !text.is_char_boundary(cut) {
            cut -= 1;
        }
        Some(text[..cut].to_string())
    }
}

/// Fetch bd memories filtered by `query`. Returns raw stdout or `None` if bd is unavailable.
pub fn fetch_memories_for_query(project_root: &Path, query: &str) -> Option<String> {
    if !project_root.join(".beads").exists() {
        return None;
    }
    let output = std::process::Command::new("bd")
        .args(["memories", query])
        .current_dir(project_root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() { None } else { Some(text) }
}

/// Store a short insight in bd by shelling out to `bd remember "<text>"`.
///
/// Returns `Ok(())` on success, `Err` if beads is not detected or the command
/// fails. Callers should warn the user on error but must not panic.
pub fn store_memory(project_root: &Path, text: &str) -> Result<(), String> {
    if !project_root.join(".beads").exists() {
        return Err("beads not detected (.beads/ directory not found)".to_string());
    }
    let status = Command::new("bd")
        .arg("remember")
        .arg(text)
        .current_dir(project_root)
        .status()
        .map_err(|e| format!("failed to run bd: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("bd remember exited with {status}"))
    }
}

/// Detect if the current working directory is a git worktree (not the main worktree).
/// Returns the main worktree root path if we're in an additional worktree, otherwise None.
pub fn detect_main_worktree_root(project_root: &Path) -> Option<std::path::PathBuf> {
    let git_dir = std::process::Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(project_root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())?;

    let git_common_dir = std::process::Command::new("git")
        .args(["rev-parse", "--git-common-dir"])
        .current_dir(project_root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())?;

    // If --git-dir != --git-common-dir, we're in an additional worktree.
    if git_dir == git_common_dir {
        return None;
    }

    // Main worktree root is the parent of --git-common-dir.
    // git-common-dir is the .git directory (e.g. /path/to/main/.git).
    let common_dir_path = if std::path::Path::new(&git_common_dir).is_absolute() {
        std::path::PathBuf::from(&git_common_dir)
    } else {
        project_root.join(&git_common_dir)
    };

    common_dir_path.parent().map(|p| p.to_path_buf())
}
