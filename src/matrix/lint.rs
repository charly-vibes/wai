//! Matrix judgment-quality linting.

use super::*;

/// Result of one lint pass: structural errors (block) and methodology
/// warnings (teach, never block).
#[derive(Debug, Default, Clone)]
pub struct LintReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Subjective words that belong in the marker, not the fact text (5.7).
const JUDGMENT_WORDS: [&str; 6] = ["good", "bad", "better", "worse", "best", "worst"];

/// One filled/incomplete cell as observed by lint: whether the fact is
/// non-empty, the single marker (if exactly one), and the fact text.
type CellObs = (bool, Option<String>, String);

/// Full structural + methodology lint over the matrix directory.
pub fn lint(dir: &Path) -> Result<LintReport> {
    require_matrix(dir)?;
    let mut report = LintReport::default();
    let criteria = list_criteria(dir)?;
    let approaches = list_approaches(dir)?;
    let crit_ids: Vec<String> = criteria
        .iter()
        .map(|c| {
            c.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string()
        })
        .collect();

    // 5.4 — status quo is the first column.
    match approaches
        .first()
        .and_then(|a| a.file_name().and_then(|s| s.to_str()))
    {
        Some(STATUS_QUO) => {}
        Some(other) => report.errors.push(format!(
            "First approach column is '{other}', not '{STATUS_QUO}'. \
             Every matrix starts with the status quo — rename so it sorts first."
        )),
        None => report.errors.push(format!(
            "Matrix has no approach columns; expected '{STATUS_QUO}' first. \
             Add one with `wai matrix approach add <name>`."
        )),
    }

    // 5.1 — rectangularity: every criterion has a cell dir in every approach.
    lint_rectangularity(&approaches, &crit_ids, &mut report);

    // Per-cell encoding (5.2/5.3): fact + exactly one known marker.
    let mut cells: Vec<(PathBuf, String, bool, Option<String>, String)> = Vec::new();
    // (cell dir, criterion id, fact non-empty, marker color)
    for approach in &approaches {
        for id in &crit_ids {
            let cell = approach.join(id);
            let snapshot = read_cell_snapshot(cell.clone(), id);
            lint_cell_encoding(&cell, &snapshot, &mut report);
            cells.push(snapshot);
        }
    }

    lint_methodology(
        dir,
        &mut report,
        &criteria,
        &approaches,
        &cells,
        crit_ids.len(),
    );

    Ok(report)
}

/// Methodology warnings (5.5–5.11) — teach the methodology, never block.
fn lint_methodology(
    dir: &Path,
    report: &mut LintReport,
    criteria: &[PathBuf],
    approaches: &[PathBuf],
    cells: &[(PathBuf, String, bool, Option<String>, String)],
    crit_count: usize,
) {
    use std::collections::HashMap;

    // Index: approach name → criterion id → (filled, marker, fact text).
    let mut by_approach: HashMap<String, HashMap<String, CellObs>> = HashMap::new();
    for (cell, id, filled, marker, text) in cells {
        let approach = cell
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        by_approach
            .entry(approach)
            .or_default()
            .insert(id.clone(), (*filled, marker.clone(), text.clone()));
    }

    lint_column_bias(&by_approach, report);
    lint_indistinguishable(approaches, &by_approach, crit_count, report);
    lint_cell_text(cells, report);
    lint_criteria_questions(criteria, report);
    lint_empty_problem(dir, report);
    lint_decided_unfilled(dir, &by_approach, report);
}

/// 5.5 — all-green columns; status quo without red.
fn lint_column_bias(
    by_approach: &std::collections::HashMap<String, std::collections::HashMap<String, CellObs>>,
    report: &mut LintReport,
) {
    for (approach, cols) in by_approach {
        let filled_markers: Vec<&String> = cols
            .values()
            .filter(|(f, _, _)| *f)
            .filter_map(|(_, m, _)| m.as_ref())
            .collect();

        // 5.5 — all-green column.
        if !filled_markers.is_empty() && filled_markers.iter().all(|m| m.as_str() == "green") {
            report.warnings.push(format!(
                "All-green column '{approach}' — are you rationalizing? \
                 A column with no weaknesses usually means a criterion is missing."
            ));
        }

        // Status quo without red (Miller: show what's wrong with today).
        if approach == STATUS_QUO
            && !filled_markers.is_empty()
            && !filled_markers.iter().any(|m| m.as_str() == "red")
        {
            report.warnings.push(
                "Status quo column has no red cell — what is wrong with today? \
                 A status quo with nothing wrong means something important was missed."
                    .to_string(),
            );
        }
    }
}

/// 5.6 — undistinguished columns: every criterion filled in both with
/// identical text → the columns differ in nothing we measured.
fn lint_indistinguishable(
    approaches: &[PathBuf],
    by_approach: &std::collections::HashMap<String, std::collections::HashMap<String, CellObs>>,
    crit_count: usize,
    report: &mut LintReport,
) {
    for i in 0..approaches.len() {
        for j in (i + 1)..approaches.len() {
            let a = approaches[i]
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            let b = approaches[j]
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            let (ca, cb) = (&by_approach[a], &by_approach[b]);
            let comparable = crit_count > 0
                && !ca.is_empty()
                && ca.len() == crit_count
                && cb.len() == crit_count
                && ca.values().all(|(f, _, _)| *f)
                && cb.values().all(|(f, _, _)| *f);
            if comparable
                && ca
                    .iter()
                    .all(|(id, (_, _, ta))| cb.get(id).map(|(_, _, tb)| ta == tb).unwrap_or(false))
            {
                report.warnings.push(format!(
                    "Approaches '{a}' and '{b}' are indistinguishable — \
                     identical text in every filled cell. Likely a missing criterion."
                ));
            }
        }
    }
}

/// 5.7 judgment-in-text / 5.8 link-only, per filled cell.
fn lint_cell_text(
    cells: &[(PathBuf, String, bool, Option<String>, String)],
    report: &mut LintReport,
) {
    for (cell, _, filled, _, text) in cells {
        if !filled {
            continue;
        }
        let fact = text.to_lowercase();
        let words: Vec<&str> = fact.split_whitespace().collect();
        if words
            .iter()
            .any(|w| JUDGMENT_WORDS.contains(&w.trim_matches(|c: char| !c.is_ascii_alphanumeric())))
        {
            report.warnings.push(format!(
                "Possible judgment in fact text at '{}' — the marker carries \
                 the judgment; facts.md should state how, not whether it's good.",
                cell.display()
            ));
        }
        if (fact.starts_with("http://") || fact.starts_with("https://"))
            && !fact.contains(char::is_whitespace)
        {
            report.warnings.push(format!(
                "Link-only cell '{}': a bare URL is seeing nothing — \
                 summarize the finding first, then link.",
                cell.display()
            ));
        }
    }
}

/// 5.9 — criteria phrased as questions.
fn lint_criteria_questions(criteria: &[PathBuf], report: &mut LintReport) {
    for criterion in criteria {
        if let Ok(content) = std::fs::read_to_string(criterion)
            && content.trim_end().ends_with('?')
        {
            let id = criterion
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            report.warnings.push(format!(
                "Criterion '{id}' is phrased as a question — keep '?' for open \
                 questions (found via `wai search`), criteria state what is measured."
            ));
        }
    }
}

/// 5.10 — empty problem.md.
fn lint_empty_problem(dir: &Path, report: &mut LintReport) {
    if let Ok(problem) = std::fs::read_to_string(dir.join("problem.md")) {
        let body = problem
            .lines()
            .filter(|l| !l.trim().is_empty() && l.trim() != "# Problem")
            .count();
        if body == 0 {
            report
                .warnings
                .push("problem.md is empty — what decision are you trying to make?".to_string());
        }
    }
}

/// Decided with unfilled cells + 5.11 stale decision.
fn lint_decided_unfilled(
    dir: &Path,
    by_approach: &std::collections::HashMap<String, std::collections::HashMap<String, CellObs>>,
    report: &mut LintReport,
) {
    let decision = read_decision(dir);
    if !is_decided(dir, &decision) {
        return;
    }
    let approach_name = decision.approach_name().unwrap_or_default();
    if let Some(cols) = by_approach.get(approach_name) {
        let unfilled = cols.values().filter(|(f, _, _)| !*f).count();
        if unfilled > 0 {
            report.warnings.push(format!(
                "Decided with unfilled cells: '{approach_name}' has {unfilled} \
                 incomplete cell(s). Deciding on blanks is shopping, not deliberating."
            ));
        }
    }
    lint_staleness(dir, report, &decision);
}

// Decided with unfilled cells + 5.11 stale decision.
/// Unparseable timestamps are inconclusive → warning, never an error.
/// 5.11 — matrix files strictly newer than the recorded decision timestamp.
/// Unparseable timestamps are inconclusive → warning, never an error.
fn lint_staleness(dir: &Path, report: &mut LintReport, decision: &Decision) {
    use chrono::DateTime;
    let Some(ts) = decision.decided_at.as_deref() else {
        return;
    };
    let Ok(decided) = DateTime::parse_from_rfc3339(ts) else {
        report.warnings.push(
            "Staleness check inconclusive: decision timestamp is not parseable — \
             re-run `wai matrix decide` to re-record it."
                .to_string(),
        );
        return;
    };
    let decided: DateTime<Utc> = decided.into();
    let mut newer: Vec<String> = Vec::new();
    for root in [dir.join("approaches"), dir.join("criteria")] {
        for entry in walkdir::WalkDir::new(&root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file()
                && let Some(mtime) = entry.metadata().ok().and_then(|m| m.modified().ok())
                && chrono::DateTime::<Utc>::from(mtime) > decided
            {
                newer.push(entry.path().display().to_string());
            }
        }
    }
    if !newer.is_empty() {
        report.warnings.push(format!(
            "Stale decision: {} file(s) newer than the decision ({}). \
             Deliberation resumed — re-decide with `wai matrix decide` or revert: {}",
            newer.len(),
            ts,
            newer.first().map(|s| s.as_str()).unwrap_or("")
        ));
    }
}

/// 5.1 — every criterion must have a cell dir in every approach.
fn lint_rectangularity(approaches: &[PathBuf], crit_ids: &[String], report: &mut LintReport) {
    for approach in approaches {
        for id in crit_ids {
            if !approach.join(id).is_dir() {
                report.errors.push(format!(
                    "Non-rectangular matrix: missing cell directory '{}'. \
                     Create it with `mkdir -p` or re-run \
                     `wai matrix criterion add`.",
                    approach.join(id).display()
                ));
            }
        }
    }
}

/// 5.2/5.3 — one cell: fact + exactly one known marker; unknown markers and
/// half-filled cells are structural errors.
fn lint_cell_encoding(
    cell: &Path,
    snapshot: &(PathBuf, String, bool, Option<String>, String),
    report: &mut LintReport,
) {
    if !cell.is_dir() {
        return;
    }
    let (_, _, filled, marker, _) = snapshot;
    let known = known_markers(cell, report);

    if *filled {
        lint_marker_count(cell, &known, report);
    } else if !known.is_empty() || marker.is_some() || cell.join("fact.md").exists() {
        // marker without fact, or an empty fact.md file — both break
        // the encoding. A bare empty cell dir is just incomplete.
        report.errors.push(format!(
            "Incomplete cell '{}': missing or empty fact.md {}.",
            cell.display(),
            if !known.is_empty() || marker.is_some() {
                "but a judgment marker is present"
            } else {
                "(file exists but is empty)"
            }
        ));
    }
}

/// Collect marker file names in a cell; unknown file names are errors.
fn known_markers(cell: &Path, report: &mut LintReport) -> Vec<String> {
    let mut known: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(cell) {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "fact.md" {
                continue;
            }
            if is_marker(&name) {
                known.push(name);
            } else {
                report.errors.push(format!(
                    "Unknown judgment marker '{}' in '{}' — \
                     use exactly one of: {}.",
                    name,
                    cell.display(),
                    MARKERS.join(", ")
                ));
            }
        }
    }
    known
}

/// A filled cell must carry exactly one marker.
fn lint_marker_count(cell: &Path, known: &[String], report: &mut LintReport) {
    match known.len() {
        0 => report.errors.push(format!(
            "Filled cell '{}' has no judgment marker — \
             add exactly one of: {}.",
            cell.display(),
            MARKERS.join(", ")
        )),
        1 => {}
        _ => report.errors.push(format!(
            "Cell '{}' has {} judgment markers ({}); \
             exactly one is allowed.",
            cell.display(),
            known.len(),
            known.join(", ")
        )),
    }
}

/// Snapshot of one cell for methodology analysis: (dir, criterion id,
/// fact non-empty, single known marker, trimmed fact text).
fn read_cell_snapshot(cell: PathBuf, id: &str) -> (PathBuf, String, bool, Option<String>, String) {
    if !cell.is_dir() {
        return (cell, id.to_string(), false, None, String::new());
    }
    let fact = std::fs::read_to_string(cell.join("fact.md")).unwrap_or_default();
    let filled = !fact.trim().is_empty();
    let known: Vec<String> = std::fs::read_dir(&cell)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| is_marker(n))
        .collect();
    let marker = if known.len() == 1 {
        Some(known[0].clone())
    } else {
        None
    };
    (
        cell,
        id.to_string(),
        filled,
        marker,
        fact.trim().to_string(),
    )
}
