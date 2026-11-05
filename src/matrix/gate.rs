//! Design-gate checks, progress summary, and handoff section.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateCheck {
    /// Proceed.
    Pass,
    /// Proceed, with a warning (e.g. inconclusive staleness after a clone).
    Warn(String),
    /// Transition blocked; message is self-healing (names files, points at
    /// `wai matrix decide`).
    Block(String),
}

/// Design → plan phase gate (6.1). `None` when the project has no matrix —
/// projects without a matrix are never gated.
pub fn design_gate(dir: &Path) -> Option<GateCheck> {
    if !dir.join("problem.md").exists() {
        return None;
    }

    if problem_is_empty(dir) {
        return Some(GateCheck::Block(
            "Phase gate blocked: problem.md is empty — what decision are you \
             trying to make? Fill it in, then re-run `wai phase next`."
                .to_string(),
        ));
    }

    // Scaffolded template names no approach → not decided.
    let decision = read_decision(dir);
    if !is_decided(dir, &decision) {
        return Some(GateCheck::Block(
            "Phase gate blocked: no decision has been made in the decision matrix. \
             Record one with `wai matrix decide <approach> <rationale>`."
                .to_string(),
        ));
    }

    // Staleness (content timestamp from decision.md, not mtimes of decision.md).
    let ts = decision.decided_at.as_deref().unwrap_or_default();
    let decided = chrono::DateTime::parse_from_rfc3339(ts)
        .ok()
        .map(chrono::DateTime::<Utc>::from);
    let Some(decided) = decided else {
        return Some(GateCheck::Warn(
            "Matrix staleness inconclusive (decision timestamp missing or \
             unparseable — e.g. fresh clone). Proceeding; run `wai matrix lint` \
             to double-check."
                .to_string(),
        ));
    };

    let newest = newest_matrix_file(dir);

    // Compare on whole seconds: the recorded timestamp is second-precision,
    // so sub-second local mtimes must not read as "newer" (ties are ambiguous
    // → warn, never a false block).
    match newest {
        Some((t, path)) if t.timestamp() > decided.timestamp() => Some(GateCheck::Block(format!(
            "Phase gate blocked: matrix files newer than the decision ({ts}) — \
             deliberation resumed and is not finished. Offending file: {path}. \
             Re-decide with `wai matrix decide <approach> <rationale>` or revert the edit."
        ))),
        Some((t, _)) if t.timestamp() == decided.timestamp() => Some(GateCheck::Warn(
            "Matrix staleness inconclusive: files share the decision timestamp \
             (fresh clone?). Proceeding; run `wai matrix lint` to double-check."
                .to_string(),
        )),
        _ => Some(GateCheck::Pass),
    }
}

/// Design-phase progress snapshot for `wai status` (6.2).
#[derive(Debug, Clone)]
pub struct MatrixProgress {
    /// First content line of problem.md (the A1).
    pub problem: String,
    pub filled: usize,
    pub total: usize,
    /// Relative path of the next unfilled cell, if any.
    pub next_cell: Option<String>,
}

/// `None` when the project has no matrix.
pub fn progress(dir: &Path) -> Option<MatrixProgress> {
    if !dir.join("problem.md").exists() {
        return None;
    }
    let problem = std::fs::read_to_string(dir.join("problem.md"))
        .unwrap_or_default()
        .lines()
        .map(|l| l.trim())
        .find(|l| !l.is_empty() && *l != "# Problem")
        .unwrap_or("")
        .to_string();

    let criteria = list_criteria(dir).ok()?;
    let approaches = list_approaches(dir).ok()?;
    let mut filled = 0usize;
    let mut total = 0usize;
    let mut next_cell = None;
    for approach in &approaches {
        for criterion in &criteria {
            total += 1;
            let id = criterion
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            let fact =
                std::fs::read_to_string(approach.join(id).join("fact.md")).unwrap_or_default();
            if fact.trim().is_empty() {
                if next_cell.is_none() {
                    next_cell = Some(format!(
                        "approaches/{}/{}",
                        approach
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or_default(),
                        id
                    ));
                }
            } else {
                filled += 1;
            }
        }
    }
    Some(MatrixProgress {
        problem,
        filled,
        total,
        next_cell,
    })
}

/// Handoff section for `wai close` (6.3): the reasoning, not just the
/// conclusion. `None` when the project has no matrix.
pub fn handoff_section(dir: &Path, project: &str) -> Option<String> {
    if !dir.join("problem.md").exists() {
        return None;
    }
    let progress = progress(dir)?;
    let decision = read_decision(dir);
    let mut section = String::from("\n## Decision Matrix\n\n");
    section.push_str(&format!("- Problem: {}\n", progress.problem));
    match decision.approach_name() {
        Some(name) => {
            section.push_str(&format!(
                "- Decision: {} — {}\n",
                name,
                decision.rationale.as_deref().unwrap_or("")
            ));
        }
        None => section.push_str("- Decision: not decided yet\n"),
    }
    section.push_str(&format!(
        "- Matrix: .wai/projects/{project}/designs/matrix/ \
         ({}/{} cells filled)\n",
        progress.filled, progress.total
    ));
    if let Some(doc) = decision.design_doc.as_deref()
        && !doc.is_empty()
        && !doc.starts_with("(none")
    {
        section.push_str(&format!("- Design doc: {doc}\n"));
    }
    Some(section)
}

/// Newest file under approaches/ or criteria/ (mtime, path).
fn newest_matrix_file(dir: &Path) -> Option<(chrono::DateTime<Utc>, String)> {
    let mut newest: Option<(chrono::DateTime<Utc>, String)> = None;
    for root in [dir.join("approaches"), dir.join("criteria")] {
        for entry in walkdir::WalkDir::new(&root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file()
                && let Some(mtime) = entry.metadata().ok().and_then(|m| m.modified().ok())
            {
                let mtime = chrono::DateTime::<Utc>::from(mtime);
                if newest.as_ref().map(|(t, _)| mtime > *t).unwrap_or(true) {
                    newest = Some((mtime, entry.path().display().to_string()));
                }
            }
        }
    }
    newest
}

/// True when problem.md exists but has no substantive body lines.
fn problem_is_empty(dir: &Path) -> bool {
    let Ok(problem) = std::fs::read_to_string(dir.join("problem.md")) else {
        return false;
    };
    problem
        .lines()
        .filter(|l| !l.trim().is_empty() && l.trim() != "# Problem")
        .count()
        == 0
}
