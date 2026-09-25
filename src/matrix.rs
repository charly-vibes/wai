//! Decision matrix — core logic.
//!
//! The matrix is a plain directory tree under
//! `.wai/projects/<project>/designs/matrix/` (see openspec change
//! `add-decision-matrix`). The filesystem is the source of truth; this module
//! implements the multi-directory operations (`init`, criterion/approach add,
//! `decide`) plus the shared model (naming, markers, decision parsing) used by
//! the lint, render, and phase-gate code paths.
//!
//! Cell encoding: each cell is `approaches/<approach>/<criterion>/` containing
//! `fact.md` (the aspect — facts, not judgment) and exactly one judgment
//! marker file named [`neutral`, `green`, `yellow`, `red`]. A cell is
//! incomplete iff `fact.md` is missing or empty; `neutral` is a legitimate
//! judgment, distinct from incomplete.

use chrono::Utc;
use miette::{IntoDiagnostic, Result};
use std::path::{Path, PathBuf};

/// The always-first column: the approach you already have.
pub const STATUS_QUO: &str = "01-status-quo";

/// Judgment marker file names — the four colors of the methodology.
pub const MARKERS: [&str; 4] = ["neutral", "green", "yellow", "red"];

/// `true` when a directory entry name is a judgment marker file.
pub fn is_marker(name: &str) -> bool {
    MARKERS.contains(&name)
}

/// Fixed location of the matrix for a project.
pub fn matrix_dir(project_root: &Path, project: &str) -> PathBuf {
    project_root
        .join(".wai")
        .join("projects")
        .join(project)
        .join("designs")
        .join("matrix")
}

/// Slugify a user-provided name into a directory/file-safe identifier:
/// lowercase, whitespace/underscores → hyphens, keep `[a-z0-9-]`, collapse
/// and trim hyphens.
pub fn slugify(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut prev_dash = true; // trim leading
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if (c.is_whitespace() || c == '_' || c == '-') && !prev_dash {
            out.push('-');
            prev_dash = true;
        }
        // everything else is dropped
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

/// Largest `NN-` prefix currently used inside `dir`, or 0 when empty/missing.
fn max_prefix(dir: &Path) -> u32 {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    entries
        .filter_map(|e| e.ok())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter_map(|n| n.split('-').next().and_then(|p| p.parse::<u32>().ok()))
        .max()
        .unwrap_or(0)
}

/// Sorted criterion definition files (`criteria/NN-slug.md`).
pub fn list_criteria(dir: &Path) -> Result<Vec<PathBuf>> {
    let criteria = dir.join("criteria");
    let mut out = Vec::new();
    if !criteria.is_dir() {
        return Ok(out);
    }
    for entry in std::fs::read_dir(&criteria).into_diagnostic()? {
        let path = entry.into_diagnostic()?.path();
        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}

/// Sorted approach directories (`approaches/NN-slug/`).
pub fn list_approaches(dir: &Path) -> Result<Vec<PathBuf>> {
    let approaches = dir.join("approaches");
    let mut out = Vec::new();
    if !approaches.is_dir() {
        return Ok(out);
    }
    for entry in std::fs::read_dir(&approaches).into_diagnostic()? {
        let path = entry.into_diagnostic()?.path();
        if path.is_dir() {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}

/// Scaffold a fresh matrix. Fails when a matrix already exists (one matrix
/// per project — edit the existing one instead).
pub fn init(dir: &Path, problem: &str) -> Result<()> {
    if dir.exists() {
        miette::bail!(
            "Matrix already exists at '{}'. There is one matrix per project — \
             edit problem.md to re-anchor it, or delete the directory to start over.",
            dir.display()
        );
    }
    std::fs::create_dir_all(dir.join("criteria")).into_diagnostic()?;
    std::fs::create_dir_all(dir.join("approaches").join(STATUS_QUO)).into_diagnostic()?;

    std::fs::write(dir.join("problem.md"), format!("# Problem\n\n{problem}\n"))
        .into_diagnostic()?;
    std::fs::write(
        dir.join("approaches")
            .join(STATUS_QUO)
            .join("_description.md"),
        "# Status quo\n\n\
         Keep things as they are. Every matrix starts with this column —\n\
         it must show what is wrong with today, not only what works.\n",
    )
    .into_diagnostic()?;
    std::fs::write(
        dir.join("decision.md"),
        "# Decision\n\n\
         Selected approach: (none)\n\
         Rationale: (none)\n\
         Decided at: (none)\n\
         Design doc: (none)\n",
    )
    .into_diagnostic()?;
    Ok(())
}

/// Add a criterion: `criteria/NN-slug.md` plus an empty cell directory in
/// every existing approach (rectangular by construction).
pub fn add_criterion(dir: &Path, name: &str) -> Result<PathBuf> {
    require_matrix(dir)?;
    let slug = slugify(name);
    if slug.is_empty() {
        miette::bail!("Criterion name must contain at least one letter or digit.");
    }
    let n = max_prefix(&dir.join("criteria")) + 1;
    let id = format!("{n:02}-{slug}");

    let def = dir.join("criteria").join(format!("{id}.md"));
    std::fs::write(
        &def,
        format!("# {slug}\n\n(what does this criterion measure?)\n"),
    )
    .into_diagnostic()?;

    for approach in list_approaches(dir)? {
        std::fs::create_dir_all(approach.join(&id)).into_diagnostic()?;
    }
    Ok(def)
}

/// Add an approach: `approaches/NN-slug/` with `_description.md` plus an
/// empty cell directory for every existing criterion.
pub fn add_approach(dir: &Path, name: &str) -> Result<PathBuf> {
    require_matrix(dir)?;
    let slug = slugify(name);
    if slug.is_empty() {
        miette::bail!("Approach name must contain at least one letter or digit.");
    }
    let n = max_prefix(&dir.join("approaches")) + 1;
    let id = format!("{n:02}-{slug}");

    let path = dir.join("approaches").join(&id);
    std::fs::create_dir_all(&path).into_diagnostic()?;
    std::fs::write(
        path.join("_description.md"),
        format!("# {slug}\n\n(describe this approach)\n"),
    )
    .into_diagnostic()?;
    for criterion in list_criteria(dir)? {
        let id = criterion
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        std::fs::create_dir_all(path.join(&id)).into_diagnostic()?;
    }
    Ok(path)
}

fn require_matrix(dir: &Path) -> Result<()> {
    if !dir.join("problem.md").exists() {
        miette::bail!(
            "No matrix found at '{}'. Initialize one first with `wai matrix init <problem>`.",
            dir.display()
        );
    }
    Ok(())
}

/// UTC timestamp written inside `decision.md` by `decide`. Rendered in RFC
/// 3339 with second precision so gate comparisons are deterministic.
pub fn now_utc() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}
