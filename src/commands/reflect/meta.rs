use std::path::{Path, PathBuf};

use miette::{IntoDiagnostic, Result};

/// Metadata stored in `.wai/projects/<project>/.reflect-meta`.
#[derive(Debug, Clone, PartialEq)]
pub struct ReflectMeta {
    pub last_reflected: String,
    pub session_count: u32,
}

/// Read `.reflect-meta` TOML from `project_dir`. Returns `None` if the file
/// does not exist; returns an error if the file is malformed.
pub fn read_reflect_meta(project_dir: &Path) -> Result<Option<ReflectMeta>> {
    let path = project_dir.join(".reflect-meta");
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&path).into_diagnostic()?;
    let table: toml::Table = raw.parse().into_diagnostic()?;

    let last_reflected = table
        .get("last_reflected")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let session_count = table
        .get("session_count")
        .and_then(|v| v.as_integer())
        .unwrap_or(0) as u32;

    Ok(Some(ReflectMeta {
        last_reflected,
        session_count,
    }))
}

/// Write `.reflect-meta` TOML to `project_dir`, overwriting any existing file.
pub fn write_reflect_meta(project_dir: &Path, meta: &ReflectMeta) -> Result<()> {
    let path = project_dir.join(".reflect-meta");
    let content = format!(
        "last_reflected = \"{}\"\nsession_count = {}\n",
        meta.last_reflected, meta.session_count
    );
    std::fs::write(&path, content).into_diagnostic()?;
    Ok(())
}

/// Write a reflection resource file to `.wai/resources/reflections/<date>-<project>.md`.
///
/// Prepends YAML front-matter with `date`, `project`, `sessions_analyzed`, and
/// `type: reflection`. Creates the reflections directory if it doesn't exist.
///
/// If a reflection file for the same project already exists (any date), it is
/// updated in place rather than creating a new numbered file.  There should be
/// at most one reflection file per project at any time.
///
/// * `handoff_count` — number of handoff artifacts analyzed; written as
///   `sessions_analyzed` in the YAML front-matter.
///
/// Returns the path of the file written.
pub fn write_reflect_resource(
    project_root: &Path,
    project_name: &str,
    content: &str,
    handoff_count: usize,
) -> Result<PathBuf> {
    let refl_dir = crate::config::reflections_dir(project_root);
    std::fs::create_dir_all(&refl_dir).into_diagnostic()?;

    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let name_slug = slug::slugify(project_name);

    // Look for an existing reflection file for this project (any date).
    // Convention: files are named `<date>-<slug>.md`; migrated files end in
    // `-migrated.md` and are left untouched.
    let existing_path = find_existing_reflection(&refl_dir, &name_slug);

    let path = existing_path.unwrap_or_else(|| refl_dir.join(format!("{}-{}.md", date, name_slug)));

    let mut front_matter = format!(
        "---\ndate: \"{}\"\nproject: \"{}\"\nsessions_analyzed: {}\ntype: reflection\n---\n\n",
        date, project_name, handoff_count
    );
    front_matter.push_str(content);
    std::fs::write(&path, front_matter).into_diagnostic()?;
    Ok(path)
}

/// Find an existing reflection file for `slug` inside `refl_dir`.
///
/// Matches files whose stem ends with `-<slug>` — i.e. the pattern
/// `<date>-<slug>.md` — and excludes `-migrated` files.
/// Returns the path of the first match, or `None` if none exists.
pub(super) fn find_existing_reflection(refl_dir: &Path, slug: &str) -> Option<PathBuf> {
    let suffix = format!("-{}", slug);
    let entries = std::fs::read_dir(refl_dir).ok()?;
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        // Skip migrated files.
        if stem.ends_with("-migrated") {
            continue;
        }
        if stem.ends_with(&suffix) {
            return Some(path);
        }
    }
    None
}

/// Compute the path that `write_reflect_resource` would write to, without writing.
///
/// Used for `--dry-run` to show the user what path would be created.
pub fn predict_reflect_resource_path(project_root: &Path, project_name: &str) -> PathBuf {
    let refl_dir = crate::config::reflections_dir(project_root);
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let name_slug = slug::slugify(project_name);
    // Mirror the idempotent logic: reuse an existing reflection if one exists.
    if let Some(existing) = find_existing_reflection(&refl_dir, &name_slug) {
        return existing;
    }
    refl_dir.join(format!("{}-{}.md", date, name_slug))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn tmp() -> TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    // ── ReflectMeta tests ──────────────────────────────────────────────────

    #[test]
    fn read_reflect_meta_returns_none_when_file_missing() {
        let dir = tmp();
        let result = read_reflect_meta(dir.path()).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn write_then_read_reflect_meta_round_trips() {
        let dir = tmp();
        let meta = ReflectMeta {
            last_reflected: "2026-02-24".to_string(),
            session_count: 7,
        };
        write_reflect_meta(dir.path(), &meta).unwrap();
        let read_back = read_reflect_meta(dir.path())
            .unwrap()
            .expect("should exist");
        assert_eq!(read_back.last_reflected, "2026-02-24");
        assert_eq!(read_back.session_count, 7);
    }

    #[test]
    fn write_reflect_meta_creates_valid_toml_file() {
        let dir = tmp();
        let meta = ReflectMeta {
            last_reflected: "2026-01-01".to_string(),
            session_count: 3,
        };
        write_reflect_meta(dir.path(), &meta).unwrap();
        let raw = fs::read_to_string(dir.path().join(".reflect-meta")).unwrap();
        assert!(raw.contains("last_reflected = \"2026-01-01\""));
        assert!(raw.contains("session_count = 3"));
    }

    #[test]
    fn write_reflect_meta_overwrites_existing() {
        let dir = tmp();
        let first = ReflectMeta {
            last_reflected: "2026-01-01".to_string(),
            session_count: 1,
        };
        write_reflect_meta(dir.path(), &first).unwrap();
        let second = ReflectMeta {
            last_reflected: "2026-02-24".to_string(),
            session_count: 12,
        };
        write_reflect_meta(dir.path(), &second).unwrap();
        let read_back = read_reflect_meta(dir.path()).unwrap().unwrap();
        assert_eq!(read_back.last_reflected, "2026-02-24");
        assert_eq!(read_back.session_count, 12);
    }

    // ── write_reflect_resource ────────────────────────────────────────────────

    // 6.1: writes correct path with YAML front-matter
    #[test]
    fn write_reflect_resource_creates_file_with_front_matter() {
        let dir = tmp();
        write_reflect_resource(dir.path(), "my-proj", "body text", 3).unwrap();

        let refl_dir = crate::config::reflections_dir(dir.path());
        let entries: Vec<_> = fs::read_dir(&refl_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(entries.len(), 1, "expected exactly one reflection file");

        let content = fs::read_to_string(entries[0].path()).unwrap();
        assert!(
            content.contains("sessions_analyzed: 3"),
            "missing sessions_analyzed"
        );
        assert!(
            content.contains("project: \"my-proj\""),
            "missing project field"
        );
        assert!(content.contains("type: reflection"), "missing type field");
        assert!(content.contains("body text"), "missing content body");
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        assert!(
            content.contains(&format!("date: \"{}\"", today)),
            "missing date field"
        );
    }

    #[test]
    fn write_reflect_resource_filename_contains_slug_and_date() {
        let dir = tmp();
        write_reflect_resource(dir.path(), "My Project", "content", 1).unwrap();

        let refl_dir = crate::config::reflections_dir(dir.path());
        let entries: Vec<_> = fs::read_dir(&refl_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(entries.len(), 1);

        let filename = entries[0].file_name();
        let name = filename.to_string_lossy();
        // slug::slugify("My Project") == "my-project"
        assert!(
            name.contains("my-project"),
            "filename should contain slugified name, got: {name}"
        );
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        assert!(
            name.contains(&today),
            "filename should contain date, got: {name}"
        );
        assert!(name.ends_with(".md"), "filename should end with .md");
    }

    #[test]
    fn write_reflect_resource_creates_dir_if_missing() {
        let dir = tmp();
        // reflections dir does not exist yet — function must create it
        let refl_dir = crate::config::reflections_dir(dir.path());
        assert!(
            !refl_dir.exists(),
            "precondition: reflections dir should not exist yet"
        );

        write_reflect_resource(dir.path(), "proj", "content", 0).unwrap();
        assert!(
            refl_dir.exists(),
            "reflections dir should have been created"
        );
        assert_eq!(
            fs::read_dir(&refl_dir).unwrap().count(),
            1,
            "expected one file written in the reflections dir"
        );
    }

    // 6.2: repeated calls update existing file in place (idempotent)
    #[test]
    fn write_reflect_resource_updates_existing_in_place() {
        let dir = tmp();

        let path1 = write_reflect_resource(dir.path(), "proj", "first", 1).unwrap();
        let path2 = write_reflect_resource(dir.path(), "proj", "second", 2).unwrap();
        let path3 = write_reflect_resource(dir.path(), "proj", "third", 3).unwrap();

        // All three calls must return the same path.
        assert_eq!(path1, path2, "second call should reuse the same file");
        assert_eq!(path2, path3, "third call should reuse the same file");

        let refl_dir = crate::config::reflections_dir(dir.path());
        let names: Vec<String> = fs::read_dir(&refl_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();

        assert_eq!(
            names.len(),
            1,
            "expected exactly one reflection file, got: {names:?}"
        );

        // The file should contain the content from the latest call.
        let content = fs::read_to_string(&path3).unwrap();
        assert!(
            content.contains("third"),
            "file should contain latest content"
        );
        assert!(
            !content.contains("first"),
            "file should not contain stale content"
        );
    }
}
