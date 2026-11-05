//! Shared helpers for `wai add` artifact assembly and persistence.

use chrono::Utc;
use miette::{IntoDiagnostic, Result};
use std::path::Path;

use super::tags;

/// Read artifact body from `--content` or `--file`.
pub(super) fn get_content(content: Option<&str>, file: Option<&str>) -> Result<String> {
    if let Some(path) = file {
        return std::fs::read_to_string(path).into_diagnostic();
    }
    if let Some(text) = content {
        return Ok(text.to_string());
    }
    Err(miette::miette!(
        "Provide content or use --file to import from a file"
    ))
}

/// Derive a unique `<date>-<slug>.md` filename inside `dir`.
pub(super) fn make_unique_filename(dir: &Path, body: &str) -> String {
    let slug = slug::slugify(body.chars().take(50).collect::<String>());
    let date = Utc::now().format("%Y-%m-%d");
    let base = format!("{}-{}.md", date, slug);
    let mut filename = base.clone();
    let mut counter = 2;
    while dir.join(&filename).exists() {
        filename = format!("{}-{}-{}.md", date, slug, counter);
        counter += 1;
    }
    filename
}

/// Split a comma-separated `--tracks` argument into a list.
pub(super) fn parse_tracks_arg(tracks: Option<&str>) -> Vec<String> {
    match tracks {
        None => vec![],
        Some(s) => s
            .split(',')
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect(),
    }
}

/// Attach the `pipeline-addendum:<step>` tag when `--corrects` points at a
/// pipeline-tagged artifact.
pub(super) fn process_corrects(corrects: Option<&str>, tags: &mut Vec<String>) -> Result<()> {
    if let Some(path) = corrects
        && let Some(step_id) = tags::resolve_pipeline_step_from_artifact(path)
    {
        tags::warn_if_unlocked(path, &step_id)?;
        let addendum_tag = format!("pipeline-addendum:{}", step_id);
        if !tags.contains(&addendum_tag) {
            tags.push(addendum_tag);
        }
    }
    Ok(())
}

/// Render one frontmatter line per `Some(entry)`; entries are pre-rendered
/// `key: value` strings. Returns empty string when no fields are present.
fn render_frontmatter(entries: Vec<Option<String>>) -> String {
    let lines: Vec<String> = entries.into_iter().flatten().collect();
    if lines.is_empty() {
        return String::new();
    }
    let mut out = String::from("---\n");
    for line in lines {
        out.push_str(&line);
        out.push('\n');
    }
    out.push_str("---\n\n");
    out
}

/// Assemble the full file content for research/plan/design artifacts:
/// optional frontmatter (tags, bead, corrects, tracks) followed by the body.
pub(super) fn assemble_artifact(
    body: &str,
    all_tags: &[String],
    bead: Option<&String>,
    corrects: Option<&String>,
    tracks: &[String],
) -> String {
    let tags_line = (!all_tags.is_empty()).then(|| format!("tags: [{}]", all_tags.join(", ")));
    let bead_line = bead.map(|b| format!("bead: {}", b));
    let corrects_line = corrects.map(|c| format!("corrects: {}", c));
    let tracks_lines = (!tracks.is_empty())
        .then(|| {
            let mut s = String::from("tracks:");
            for t in tracks {
                s.push_str(&format!("\n  - {}", t));
            }
            s
        });

    let mut file_content = render_frontmatter(vec![tags_line, bead_line, corrects_line, tracks_lines]);
    file_content.push_str(body);
    file_content.push('\n');
    file_content
}

/// Write the artifact to disk, adding the freshness sidecar when tracks exist.
pub(super) fn write_artifact(
    dir: &Path,
    filename: &str,
    file_content: &str,
    project_root: &Path,
    tracks: &[String],
) -> Result<()> {
    std::fs::create_dir_all(dir).into_diagnostic()?;
    let artifact_path = dir.join(filename);
    std::fs::write(&artifact_path, file_content).into_diagnostic()?;
    if !tracks.is_empty() {
        crate::freshness::write_sidecar(&artifact_path, project_root, tracks);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn bead_field_written_to_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        let research_dir = dir.path().join("research");
        std::fs::create_dir_all(&research_dir).unwrap();

        let bead_id = "wai-p94k".to_string();
        let body = "some research notes";
        let filename = super::make_unique_filename(&research_dir, body);

        let file_content = super::assemble_artifact(body, &[], Some(&bead_id), None, &[]);

        std::fs::write(research_dir.join(&filename), &file_content).unwrap();
        let written = std::fs::read_to_string(research_dir.join(&filename)).unwrap();
        assert!(
            written.contains("bead: wai-p94k"),
            "bead field missing from frontmatter"
        );
        assert!(written.contains("---"), "frontmatter delimiters missing");
        assert!(
            !written.contains("tags:"),
            "tags should not appear when not provided"
        );
    }

    #[test]
    fn bead_and_tags_both_written_when_both_provided() {
        let bead_id = "wai-abc".to_string();
        let tags = ["research".to_string(), "design".to_string()];
        let file_content = super::assemble_artifact("notes\n", &tags, Some(&bead_id), None, &[]);

        assert!(file_content.contains("tags: [research, design]"));
        assert!(file_content.contains("bead: wai-abc"));
    }

    #[test]
    fn no_frontmatter_when_neither_tags_nor_bead() {
        let file_content = super::assemble_artifact("notes\n", &[], None, None, &[]);
        assert!(
            !file_content.contains("---"),
            "no frontmatter should be written"
        );
    }

    #[test]
    fn corrects_field_written_to_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        let research_dir = dir.path().join("research");
        std::fs::create_dir_all(&research_dir).unwrap();

        let corrects_path = "research/2026-04-01-original.md".to_string();
        let body = "corrected notes";
        let filename = super::make_unique_filename(&research_dir, body);

        let file_content = super::assemble_artifact(body, &[], None, Some(&corrects_path), &[]);

        std::fs::write(research_dir.join(&filename), &file_content).unwrap();
        let written = std::fs::read_to_string(research_dir.join(&filename)).unwrap();
        assert!(
            written.contains("corrects: research/2026-04-01-original.md"),
            "corrects field missing from frontmatter"
        );
        assert!(written.contains("---"), "frontmatter delimiters missing");
    }

    #[test]
    fn tracks_written_as_list_in_frontmatter() {
        let tracks = vec!["a".to_string(), "b".to_string()];
        let file_content = super::assemble_artifact("body", &[], None, None, &tracks);
        assert!(file_content.contains("tracks:\n  - a\n  - b"));
    }
}
