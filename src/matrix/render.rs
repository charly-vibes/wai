//! HTML report rendering for a matrix.

use super::*;

/// Pure function of the matrix directory → self-contained HTML. Same state,
/// byte-identical output. The output is a generated view: never edited,
/// never committed, always regenerable.
pub fn render(dir: &Path) -> Result<PathBuf> {
    require_matrix(dir)?;

    let problem = std::fs::read_to_string(dir.join("problem.md"))
        .unwrap_or_else(|_| "(problem.md missing)".to_string());
    let decision = read_decision(dir);

    let mut html = String::with_capacity(16 * 1024);
    push_html_head(&mut html);
    render_banner(&mut html, &problem);
    render_decision_banner(&mut html, &decision);

    let criteria = list_criteria(dir)?;
    let approaches = list_approaches(dir)?;
    render_table(&mut html, &criteria, &approaches);
    render_assessment_key(&mut html);
    html.push_str("</body>\n</html>\n");

    let out = dir.join("matrix.html");
    std::fs::write(&out, html).into_diagnostic()?;
    Ok(out)
}

/// HTML preamble: doctype + inline stylesheet.
fn push_html_head(html: &mut String) {
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
}

/// 1. problem.md as the banner, above everything else.
fn render_banner(html: &mut String, problem: &str) {
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
}

/// 5. decision.md below the banner.
fn render_decision_banner(html: &mut String, decision: &Decision) {
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
}

/// 2./3. criteria rows × approach columns.
fn render_table(html: &mut String, criteria: &[PathBuf], approaches: &[PathBuf]) {
    html.push_str("<table>\n<thead>\n<tr>\n<th></th>\n");
    // approach columns, sorted (01-status-quo asserted first by lint).
    for approach in approaches {
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

    // criteria sorted → ordered rows.
    for criterion in criteria {
        let id = criterion
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        html.push_str(&format!("<tr class=\"crit\"><td>{}</td>", escape_html(id)));
        for approach in approaches {
            html.push_str(&render_cell(approach, id));
        }
        html.push_str("</tr>\n");
    }
    html.push_str("</tbody>\n</table>\n");
}

/// 6. Assessment key baked in.
fn render_assessment_key(html: &mut String) {
    html.push_str("<div class=\"key\"><strong>Assessment key:</strong> ");
    for color in MARKERS {
        let (class, label) = chip(color);
        html.push_str(&format!("<span class=\"chip {}\">{}</span>", class, label));
    }
    html.push_str(
        " — a cell with no color is <em>not yet assessed</em>; \
         neutral is a judgment (\"clear\"), not incompleteness.</div>\n",
    );
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

// ── Workflow integration ──────────────────────────────────────────────────
