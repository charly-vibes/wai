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

// ── Deciding ─────────────────────────────────────────────────────────────────

/// A parsed `decision.md`. `approach` is the raw `Selected approach:` value
/// (before validity checking); a freshly-scaffolded template yields `(none)`.
#[derive(Debug, Clone, Default)]
pub struct Decision {
    pub approach: Option<String>,
    pub rationale: Option<String>,
    /// RFC 3339 UTC timestamp written by `decide`; `(none)` until then.
    pub decided_at: Option<String>,
    pub design_doc: Option<String>,
}

impl Decision {
    /// The recorded-approach name, when present and not the template marker.
    pub fn approach_name(&self) -> Option<&str> {
        self.approach
            .as_deref()
            .filter(|s| !s.is_empty() && !s.starts_with("(none"))
    }
}

/// Parse `decision.md`. Line-oriented: `Key: value` under `# Decision`.
/// A missing or unparseable file yields the default (never decided).
pub fn read_decision(dir: &Path) -> Decision {
    let mut decision = Decision::default();
    let Ok(content) = std::fs::read_to_string(dir.join("decision.md")) else {
        return decision;
    };
    for line in content.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "Selected approach" => decision.approach = Some(value.to_string()),
            "Rationale" => decision.rationale = Some(value.to_string()),
            "Decided at" => decision.decided_at = Some(value.to_string()),
            "Design doc" => decision.design_doc = Some(value.to_string()),
            _ => {}
        }
    }
    decision
}

/// `true` when the decision names an approach directory that exists under
/// `approaches/`. A scaffolded template names `(none)` → not decided.
pub fn is_decided(dir: &Path, decision: &Decision) -> bool {
    let Some(name) = decision.approach_name() else {
        return false;
    };
    dir.join("approaches").join(name).is_dir()
}

/// Record a decision: validate the approach exists, write `decision.md`, and
/// scaffold a design doc in `designs/` with a decision-time snapshot of the
/// winning column's facts. Returns `(decision path, design doc path)`.
pub fn decide(
    dir: &Path,
    approach_arg: &str,
    rationale: &str,
    project_rel: &str,
) -> Result<(PathBuf, PathBuf)> {
    require_matrix(dir)?;
    let approaches = list_approaches(dir)?;
    let wanted = slugify(approach_arg);
    let match_name = approaches.iter().find_map(|a| {
        let name = a.file_name()?.to_str()?;
        let stem = name.split_once('-').map(|x| x.1).unwrap_or(name);
        (stem == wanted || name == approach_arg).then(|| name.to_string())
    });
    let Some(approach_name) = match_name else {
        let valid: Vec<&str> = approaches
            .iter()
            .filter_map(|a| a.file_name().and_then(|s| s.to_str()))
            .collect();
        miette::bail!(
            "No approach '{}' in the matrix. Valid approaches: {}. \
             Add one first with `wai matrix approach add {}`.",
            approach_arg,
            valid.join(", "),
            slugify(approach_arg)
        );
    };

    let now = now_utc();
    let today = &now[..10];
    let doc_slug = approach_name
        .split_once('-')
        .map(|x| x.1)
        .unwrap_or(approach_name.as_str());
    let designs_dir = dir
        .parent()
        .ok_or_else(|| miette::miette!("Matrix directory '{}' has no parent", dir.display()))?;

    // Unique design doc: designs/<date>-<slug>.md, numbered on re-decide.
    let mut doc = designs_dir.join(format!("{today}-{doc_slug}.md"));
    let mut counter = 2;
    while doc.exists() {
        doc = designs_dir.join(format!("{today}-{doc_slug}-{counter}.md"));
        counter += 1;
    }

    let doc_rel = format!(
        ".wai/projects/{project_rel}/designs/{}",
        doc.file_name().and_then(|s| s.to_str()).unwrap_or_default()
    );

    std::fs::write(
        dir.join("decision.md"),
        format!(
            "# Decision\n\n\
             Selected approach: {approach_name}\n\
             Rationale: {rationale}\n\
             Decided at: {now}\n\
             Design doc: {doc_rel}\n"
        ),
    )
    .into_diagnostic()?;

    let snapshot = winning_column_snapshot(dir, &approach_name)?;
    std::fs::write(
        &doc,
        format!(
            "---\n\
             tags: [design]\n\
             tracks:\n\
               - .wai/projects/{project_rel}/designs/matrix\n\
             ---\n\n\
             # Design: {doc_slug}\n\n\
             Decision: {approach_name}\n\
             Date: {now}\n\
             Matrix: .wai/projects/{project_rel}/designs/matrix/\n\n\
             ## Rationale\n\n\
             {rationale}\n\n\
             ## Trade-offs\n\n\
             (describe the trade-offs accepted)\n\n\
             ## Decision-time snapshot — winning column\n\n\
             {snapshot}"
        ),
    )
    .into_diagnostic()?;

    Ok((dir.join("decision.md"), doc))
}

/// The winning column's non-empty facts at decision time, as markdown
/// sections. The matrix keeps growing; the design doc records what was true
/// when decided.
fn winning_column_snapshot(dir: &Path, approach_name: &str) -> Result<String> {
    let mut out = String::new();
    for criterion in list_criteria(dir)? {
        let id = criterion
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let fact_path = dir
            .join("approaches")
            .join(approach_name)
            .join(id)
            .join("fact.md");
        if let Ok(fact) = std::fs::read_to_string(&fact_path)
            && !fact.trim().is_empty()
        {
            out.push_str(&format!("### {id}\n\n{}\n\n", fact.trim()));
        }
    }
    if out.is_empty() {
        out.push_str("(no cells filled at decision time)\n");
    }
    Ok(out)
}

/// UTC timestamp written inside `decision.md` by `decide`. Rendered in RFC
/// 3339 with second precision so gate comparisons are deterministic.
pub fn now_utc() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

// ── Rendering ─────────────────────────────────────────────────────────────────

/// Escape HTML special characters in cell/banner text.
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Chip class + label for a marker color.
fn chip(color: &str) -> (&'static str, &'static str) {
    match color {
        "green" => ("chip-green", "green"),
        "yellow" => ("chip-yellow", "yellow"),
        "red" => ("chip-red", "red"),
        _ => ("chip-neutral", "neutral"),
    }
}

/// Pure function of the matrix directory → self-contained HTML. Same state,
/// byte-identical output. The output is a generated view: never edited,
/// never committed, always regenerable.
pub fn render(dir: &Path) -> Result<PathBuf> {
    require_matrix(dir)?;

    let problem = std::fs::read_to_string(dir.join("problem.md"))
        .unwrap_or_else(|_| "(problem.md missing)".to_string());
    let decision = read_decision(dir);

    let mut html = String::with_capacity(16 * 1024);
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
    html.push_str("<title>Decision Matrix</title>\n<style>\n");
    html.push_str(
        "body{font-family:sans-serif;margin:2rem;color:#1a1a1a;}\
         table{border-collapse:collapse;width:100%;}\
         th,td{border:1px solid #ccc;padding:.5rem;text-align:left;vertical-align:top;}\
         th.approach{background:#f5f5f5;}\
         tr.crit td:first-child{font-weight:600;background:#fafafa;white-space:nowrap;}\
         .banner{font-size:1.4rem;font-weight:700;margin-bottom:.25rem;}\
         .decision{margin:.5rem 0 1.5rem;padding:.5rem 1rem;background:#eef;\
                   border-left:4px solid #55f;}\
         .unset{color:#999;font-style:italic;background:#f7f7f7;}\
         .chip{display:inline-block;padding:.1rem .5rem;border-radius:.75rem;\
               font-size:.75rem;font-weight:600;color:#fff;margin-top:.25rem;}\
         .chip-green{background:#2a7;}\
         .chip-yellow{background:#da3;}\
         .chip-red{background:#d43;}\
         .chip-neutral{background:#888;}\
         .key{margin-top:2rem;font-size:.85rem;}\
         .key .chip{margin:0 .25rem;}\
      ",
    );
    html.push_str("\n</style>\n</head>\n<body>\n");

    // 1. problem.md as the banner, above everything else.
    for line in problem.lines() {
        let line = line.trim();
        if line.is_empty() || line == "# Problem" {
            continue;
        }
        html.push_str(&format!(
            "<div class=\"banner\">{}</div>\n",
            escape_html(line)
        ));
    }

    // 5. decision.md below the banner.
    match decision.approach_name() {
        Some(name) => {
            let rationale = decision.rationale.as_deref().unwrap_or("");
            html.push_str(&format!(
                "<div class=\"decision\">Decided: <strong>{}</strong> — {}</div>\n",
                escape_html(name),
                escape_html(rationale)
            ));
        }
        None => {
            html.push_str("<div class=\"decision\">Not decided yet.</div>\n");
        }
    }

    let criteria = list_criteria(dir)?;
    let approaches = list_approaches(dir)?;

    html.push_str("<table>\n<thead>\n<tr>\n<th></th>\n");
    // 3. approach columns, sorted (01-status-quo asserted first by lint).
    for approach in &approaches {
        let name = approach
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        html.push_str(&format!(
            "<th class=\"approach\">{}</th>\n",
            escape_html(name)
        ));
    }
    html.push_str("</tr>\n</thead>\n<tbody>\n");

    // 2. criteria sorted → ordered rows.
    for criterion in &criteria {
        let id = criterion
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        html.push_str(&format!("<tr class=\"crit\"><td>{}</td>", escape_html(id)));
        for approach in &approaches {
            html.push_str(&render_cell(approach, id));
        }
        html.push_str("</tr>\n");
    }
    html.push_str("</tbody>\n</table>\n");

    // 6. Assessment key baked in.
    html.push_str("<div class=\"key\"><strong>Assessment key:</strong> ");
    for color in MARKERS {
        let (class, label) = chip(color);
        html.push_str(&format!("<span class=\"chip {}\">{}</span>", class, label));
    }
    html.push_str(
        " — a cell with no color is <em>not yet assessed</em>; \
         neutral is a judgment (\"clear\"), not incompleteness.</div>\n",
    );
    html.push_str("</body>\n</html>\n");

    let out = dir.join("matrix.html");
    std::fs::write(&out, html).into_diagnostic()?;
    Ok(out)
}

/// One table cell: fact text + judgment chip, or the gray placeholder.
fn render_cell(approach: &Path, criterion_id: &str) -> String {
    let cell = approach.join(criterion_id);
    let fact = std::fs::read_to_string(cell.join("fact.md")).unwrap_or_default();
    if fact.trim().is_empty() {
        // 4.2 — incomplete cells are never a judgment color.
        return "<td class=\"unset\">not yet assessed</td>\n".to_string();
    }
    let marker = std::fs::read_dir(&cell)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().into_string().ok())
                .find(|n| is_marker(n))
        })
        .ok()
        .flatten();
    match marker {
        Some(color) => {
            let (class, label) = chip(&color);
            format!(
                "<td>{}<br><span class=\"chip {}\">{}</span></td>\n",
                escape_html(fact.trim()),
                class,
                label
            )
        }
        // Filled fact, no marker: show the fact, no chip (lint flags it).
        None => format!("<td>{}</td>\n", escape_html(fact.trim())),
    }
}
