use miette::Result;
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::io::{IsTerminal, Write as _};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use walkdir::WalkDir;

use crate::config::{STATE_FILE, projects_dir};
use crate::state::ProjectState;

/// Maximum number of concurrent `bd stats --json` subprocesses.
const MAX_PARALLEL_BD: usize = 8;

struct Row {
    name: String,
    workspace: PathBuf,
    phase: String,
    counts: Option<(u64, u64)>,
}

pub fn run(root: Option<PathBuf>, depth: Option<usize>, timeout_secs: u64) -> Result<()> {
    // Resolve root
    let explicit_root = root.is_some();
    let root = match root {
        Some(r) => {
            if !r.exists() {
                miette::bail!("Root path does not exist: {}", r.display());
            }
            r
        }
        None => {
            dirs::home_dir().ok_or_else(|| miette::miette!("Could not determine home directory"))?
        }
    };

    // Resolve depth
    let max_depth = depth.unwrap_or(3);
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);

    // Run workspace discovery with timeout + progress indicator
    let mut workspaces = discover_workspaces(&root, max_depth, deadline);

    // When using default root, also check ancestor directories of cwd so workspaces
    // deeper than max_depth are never silently omitted.
    if !explicit_root && let Ok(cwd) = std::env::current_dir() {
        for ancestor in find_ancestor_workspaces(&cwd) {
            if !workspaces.contains(&ancestor) {
                workspaces.push(ancestor);
            }
        }
    }

    let beads_results = fetch_beads_parallel(&workspaces);
    let mut rows = build_rows(&workspaces, &beads_results);
    let any_beads = beads_results.values().any(|c| c.is_some());

    // Sort rows by project name
    rows.sort_by(|a, b| a.name.cmp(&b.name));

    if rows.is_empty() {
        println!("No wai workspaces found under {}", root.display());
        return Ok(());
    }

    render_rows(&rows, any_beads);
    Ok(())
}

/// Fetch beads stats for all workspaces in parallel, capped at MAX_PARALLEL_BD.
fn fetch_beads_parallel(workspaces: &[PathBuf]) -> HashMap<PathBuf, Option<(u64, u64)>> {
    let semaphore = Arc::new(Semaphore::new(MAX_PARALLEL_BD));
    let results = Arc::new(Mutex::new(HashMap::new()));
    let handles: Vec<_> = workspaces
        .iter()
        .filter(|ws| ws.join(".beads").exists())
        .map(|ws| {
            let ws = ws.clone();
            let results = Arc::clone(&results);
            let semaphore: Arc<Semaphore> = Arc::clone(&semaphore);
            std::thread::spawn(move || {
                // Acquire a permit before running bd — this caps concurrency.
                let _permit = semaphore.acquire();
                let counts = fetch_beads_counts(&ws);
                results.lock().unwrap().insert(ws, counts);
            })
        })
        .collect();
    for handle in handles {
        let _ = handle.join();
    }
    Arc::try_unwrap(results).unwrap().into_inner().unwrap()
}

/// Enumerate projects (with phase + beads counts) per workspace.
fn build_rows(
    workspaces: &[PathBuf],
    beads_results: &HashMap<PathBuf, Option<(u64, u64)>>,
) -> Vec<Row> {
    let mut rows: Vec<Row> = Vec::new();
    for workspace in workspaces {
        let counts = beads_results.get(workspace).copied().flatten();

        // Enumerate project names from .wai/projects/ subdirectories
        let proj_dir = projects_dir(workspace);
        if !proj_dir.exists() {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&proj_dir) else {
            continue;
        };

        let mut project_names: Vec<String> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();
        project_names.sort();

        for name in project_names {
            let state_path = proj_dir.join(&name).join(STATE_FILE);
            let phase = match ProjectState::load(&state_path) {
                Ok(state) => state.current.to_string(),
                Err(_) => "unknown".to_string(),
            };
            rows.push(Row {
                name,
                workspace: workspace.clone(),
                phase,
                counts,
            });
        }
    }
    rows
}

fn render_rows(rows: &[Row], any_beads: bool) {
    // Duplicate name detection
    let mut name_freq: HashMap<&str, usize> = HashMap::new();
    for row in rows {
        *name_freq.entry(row.name.as_str()).or_insert(0) += 1;
    }

    let home = dirs::home_dir();

    let display_names: Vec<String> = rows
        .iter()
        .map(|row| display_name_for(row, &name_freq, &home))
        .collect();

    // Compute column widths based on raw (uncolored) strings
    let max_name = display_names.iter().map(|s| s.len()).max().unwrap_or(0);
    let max_phase = rows.iter().map(|r| r.phase.len()).max().unwrap_or(0);

    // Render table
    for (i, row) in rows.iter().enumerate() {
        let name_col = format!("{:<width$}", display_names[i], width = max_name);
        let phase_col = format_phase_col(&row.phase, max_phase);

        if any_beads {
            let counts_str = match row.counts {
                Some((open, ready)) => format!("{} open, {} ready", open, ready),
                None => String::new(),
            };
            println!("{}  {}  {}", name_col, phase_col, counts_str);
        } else {
            println!("{}  {}", name_col, phase_col);
        }
    }
}

fn display_name_for(row: &Row, name_freq: &HashMap<&str, usize>, home: &Option<PathBuf>) -> String {
    if name_freq.get(row.name.as_str()).copied().unwrap_or(0) <= 1 {
        return row.name.clone();
    }
    let ws_str = match home {
        Some(h) => {
            if let Ok(rel) = row.workspace.strip_prefix(h) {
                format!("~/{}", rel.display())
            } else {
                row.workspace.display().to_string()
            }
        }
        None => row.workspace.display().to_string(),
    };
    format!("{} ({})", row.name, ws_str)
}

/// Walk the filesystem from `root` up to `max_depth` levels, returning paths of
/// directories that contain `.wai/config.toml`. Hidden directories are skipped
/// during traversal; `.wai/` itself is never recursed into.
///
/// The scan stops early when `deadline` is reached and emits a warning to stderr.
/// A progress indicator is shown on stderr when the scan takes longer than 1 second.
fn discover_workspaces(root: &Path, max_depth: usize, deadline: Instant) -> Vec<PathBuf> {
    let mut workspaces = Vec::new();
    let mut dirs_scanned: u64 = 0;
    let mut progress_shown = false;
    let start = Instant::now();
    // Only show progress when stderr is a terminal.
    let stderr_is_tty = std::io::stderr().is_terminal();

    for entry in WalkDir::new(root)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter()
        .filter_entry(is_visible_entry)
    {
        // Check deadline before processing each entry.
        if Instant::now() >= deadline {
            report_scan_timeout(&start, workspaces.len(), stderr_is_tty, progress_shown);
            break;
        }

        let Ok(entry) = entry else { continue };
        if !entry.file_type().is_dir() {
            continue;
        }

        dirs_scanned += 1;
        progress_shown = maybe_show_progress(stderr_is_tty, progress_shown, start, dirs_scanned);

        let path = entry.path();
        if path.join(".wai").join("config.toml").exists() {
            workspaces.push(path.to_path_buf());
        }
    }

    // Clear the progress line when done so the table renders cleanly.
    if stderr_is_tty && progress_shown {
        eprint!("\r\x1b[K");
        let _ = std::io::stderr().flush();
    }

    workspaces
}

// Always include the root entry; skip hidden directories during traversal.
fn is_visible_entry(e: &walkdir::DirEntry) -> bool {
    if e.depth() == 0 {
        return true;
    }
    if e.file_type().is_dir() {
        let name = e.file_name().to_string_lossy();
        if name.starts_with('.') {
            return false;
        }
    }
    true
}

// Clear progress line, then warn about partial results (matters for CI logs too).
fn report_scan_timeout(start: &Instant, found: usize, stderr_is_tty: bool, progress_shown: bool) {
    if stderr_is_tty && progress_shown {
        eprint!("\r\x1b[K");
        let _ = std::io::stderr().flush();
    }
    eprintln!(
        "  warning: scan timed out after {}s — showing {} workspaces found so far \
         (use --timeout to increase)",
        start.elapsed().as_secs(),
        found
    );
}

// Show or update progress indicator after 1 second; returns updated visibility.
fn maybe_show_progress(
    stderr_is_tty: bool,
    progress_shown: bool,
    start: Instant,
    dirs_scanned: u64,
) -> bool {
    let shown = progress_shown || (stderr_is_tty && start.elapsed() >= Duration::from_secs(1));
    if stderr_is_tty && shown {
        eprint!("\r  Scanning... {} dirs", dirs_scanned);
        let _ = std::io::stderr().flush();
    }
    shown
}

/// Walk up from `cwd` and collect every ancestor directory that contains `.wai/config.toml`.
fn find_ancestor_workspaces(cwd: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let mut dir: &Path = cwd;
    loop {
        if dir.join(".wai").join("config.toml").exists() {
            result.push(dir.to_path_buf());
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }
    result
}

/// Invoke `bd stats --json` in `workspace` and parse open/ready counts.
/// Returns None on any error (bd not installed, non-zero exit, parse failure).
fn fetch_beads_counts(workspace: &Path) -> Option<(u64, u64)> {
    let output = std::process::Command::new("bd")
        .args(["stats", "--json"])
        .current_dir(workspace)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    let summary = json.get("summary")?;
    let open = summary.get("open_issues")?.as_u64()?;
    let ready = summary.get("ready_issues")?.as_u64()?;
    Some((open, ready))
}

/// Format a phase string with color, right-padded to `max_width` inside brackets.
fn format_phase_col(phase: &str, max_width: usize) -> String {
    let colored = match phase {
        "research" => phase.yellow().to_string(),
        "design" => phase.magenta().to_string(),
        "plan" => phase.blue().to_string(),
        "implement" => phase.green().to_string(),
        "review" => phase.cyan().to_string(),
        "archive" | "unknown" => phase.dimmed().to_string(),
        _ => phase.dimmed().to_string(),
    };
    // Pad based on raw length so ANSI codes don't affect alignment
    let padding = " ".repeat(max_width.saturating_sub(phase.len()));
    format!("[{}{}]", colored, padding)
}

/// A simple counting semaphore built on top of stdlib primitives.
///
/// `acquire()` blocks until a permit is available and returns a guard that
/// releases the permit on drop.  This caps the number of concurrent callers
/// to the value passed to `Semaphore::new`.
struct Semaphore {
    mutex: Mutex<usize>,
    condvar: std::sync::Condvar,
    capacity: usize,
}

impl Semaphore {
    /// Create a new semaphore with the given capacity.
    ///
    /// # Panics
    /// Panics if `capacity` is 0, which would cause every `acquire()` to deadlock.
    fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Semaphore capacity must be > 0");
        Self {
            mutex: Mutex::new(0),
            condvar: std::sync::Condvar::new(),
            capacity,
        }
    }

    fn acquire(&self) -> SemaphoreGuard<'_> {
        let mut count = self.mutex.lock().unwrap();
        while *count >= self.capacity {
            count = self.condvar.wait(count).unwrap();
        }
        *count += 1;
        SemaphoreGuard { semaphore: self }
    }
}

struct SemaphoreGuard<'a> {
    semaphore: &'a Semaphore,
}

impl Drop for SemaphoreGuard<'_> {
    fn drop(&mut self) {
        let mut count = self.semaphore.mutex.lock().unwrap();
        *count -= 1;
        self.semaphore.condvar.notify_one();
    }
}
