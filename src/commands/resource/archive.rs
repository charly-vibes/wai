use cliclack::log;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use std::fs;
use std::io::BufReader;
use std::path::Path;

use crate::config::{SKILLS_DIR, agent_config_dir, global_skills_dir};
use crate::context::current_context;

use super::require_project;
use super::validation::validate_skill_name;

/// Validate an archive entry path.
///
/// Valid patterns:
///   `<name>/SKILL.md`           (flat skill)
///   `<cat>/<action>/SKILL.md`   (hierarchical skill)
///
/// Rejects path traversal (components starting with `..` or absolute paths).
pub(super) fn validate_archive_entry_path(path_str: &str) -> Result<()> {
    // Must end with /SKILL.md
    if !path_str.ends_with("/SKILL.md") {
        miette::bail!(
            "Invalid archive entry '{}': entries must end with '/SKILL.md'",
            path_str
        );
    }

    let components: Vec<&str> = path_str.split('/').collect();
    // Valid: ["name", "SKILL.md"] or ["cat", "action", "SKILL.md"]
    let depth = components.len();
    if !(2..=3).contains(&depth) {
        miette::bail!(
            "Invalid archive entry '{}': expected 'name/SKILL.md' or 'category/action/SKILL.md'",
            path_str
        );
    }

    // Check for path traversal in each component
    for component in &components {
        if *component == ".." || *component == "." {
            miette::bail!(
                "Invalid archive entry '{}': path traversal detected",
                path_str
            );
        }
        if component.starts_with('/') {
            miette::bail!(
                "Invalid archive entry '{}': absolute paths not allowed",
                path_str
            );
        }
    }

    // Validate skill name components (everything except the final SKILL.md)
    let name_part = components[..depth - 1].join("/");
    validate_skill_name(&name_part)?;

    Ok(())
}

/// Export named skills to a tar.gz archive.
pub(super) fn export_skills(skill_names: &[String], output_path: &str) -> Result<()> {
    let project_root = require_project()?;

    let local_skills_dir = agent_config_dir(&project_root).join(SKILLS_DIR);
    let global_dir = global_skills_dir();

    let out_file = fs::File::create(output_path).into_diagnostic()?;
    let gz = GzEncoder::new(out_file, Compression::default());
    let mut archive = tar::Builder::new(gz);

    for name in skill_names {
        validate_skill_name(name)?;

        // Find the skill (local first, then global)
        let local_path = skill_md_path(&local_skills_dir, name);
        let global_path = skill_md_path(&global_dir, name);

        let skill_md_file = if local_path.exists() {
            local_path
        } else if global_path.exists() {
            global_path
        } else {
            miette::bail!(
                "Skill '{}' not found in local project or global library",
                name
            );
        };

        // Archive path: preserve <name>/SKILL.md or <cat>/<action>/SKILL.md structure
        let archive_path = format!("{}/SKILL.md", name);

        archive
            .append_path_with_name(&skill_md_file, &archive_path)
            .into_diagnostic()?;
    }

    archive.finish().into_diagnostic()?;
    let gz_encoder = archive.into_inner().into_diagnostic()?;
    gz_encoder.finish().into_diagnostic()?;

    log::success(format!(
        "Exported {} skill{} to '{}'",
        skill_names.len(),
        if skill_names.len() == 1 { "" } else { "s" },
        output_path
    ))
    .into_diagnostic()?;

    Ok(())
}

/// Import skills from a tar.gz archive into the current project.
pub(super) fn import_skills_archive(archive_path: &str, yes: bool) -> Result<()> {
    // Merge local --yes with global --yes so both
    // `wai resource import archive f --yes` and `wai --yes resource import archive f` work.
    let yes = yes || current_context().yes;
    let project_root = require_project()?;
    crate::context::require_safe_mode("import skills from archive")?;

    let local_skills_dir = agent_config_dir(&project_root).join(SKILLS_DIR);
    fs::create_dir_all(&local_skills_dir).into_diagnostic()?;

    import_archive_into_dir(archive_path, &local_skills_dir, yes)
}

/// Extract validated skill entries from a tar.gz archive.
///
/// Returns a list of `(archive_path, content)` pairs. Validates all paths
/// before returning — callers can trust the results are safe to write.
pub(crate) fn read_archive_entries(archive_path: &str) -> Result<Vec<(String, Vec<u8>)>> {
    let archive_file = fs::File::open(archive_path).into_diagnostic()?;
    let gz = GzDecoder::new(BufReader::new(archive_file));
    let mut archive = tar::Archive::new(gz);

    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    for entry in archive.entries().into_diagnostic()? {
        let mut entry = entry.into_diagnostic()?;
        let entry_path = entry.path().into_diagnostic()?;
        let path_str = entry_path.to_string_lossy().to_string();

        if entry.header().entry_type().is_dir() {
            continue;
        }

        validate_archive_entry_path(&path_str)?;

        let mut content = Vec::new();
        std::io::copy(&mut entry, &mut content).into_diagnostic()?;
        entries.push((path_str, content));
    }
    Ok(entries)
}

/// Write validated archive entries into a skills directory.
///
/// With `yes=true` overwrites without prompting; otherwise prompts per skill.
/// Returns `(imported, overwritten, skipped)` counts.
pub(crate) fn write_archive_entries(
    entries: Vec<(String, Vec<u8>)>,
    skills_dir: &Path,
    yes: bool,
) -> Result<(usize, usize, usize)> {
    let mut imported = 0;
    let mut overwritten = 0;
    let mut skipped = 0;

    for (path_str, content) in entries {
        let skill_name = path_str.trim_end_matches("/SKILL.md");
        let dst_dir = skills_dir.join(skill_name);
        let dst_file = dst_dir.join("SKILL.md");

        if dst_file.exists() {
            if yes {
                fs::create_dir_all(&dst_dir).into_diagnostic()?;
                fs::write(&dst_file, &content).into_diagnostic()?;
                overwritten += 1;
            } else {
                let confirmed =
                    cliclack::confirm(format!("Skill '{}' already exists. Overwrite?", skill_name))
                        .interact()
                        .into_diagnostic()?;

                if confirmed {
                    fs::create_dir_all(&dst_dir).into_diagnostic()?;
                    fs::write(&dst_file, &content).into_diagnostic()?;
                    overwritten += 1;
                } else {
                    skipped += 1;
                }
            }
        } else {
            fs::create_dir_all(&dst_dir).into_diagnostic()?;
            fs::write(&dst_file, &content).into_diagnostic()?;
            imported += 1;
        }
    }
    Ok((imported, overwritten, skipped))
}

/// Core implementation: import from archive into a given skills directory.
pub(crate) fn import_archive_into_dir(
    archive_path: &str,
    skills_dir: &Path,
    yes: bool,
) -> Result<()> {
    let entries = read_archive_entries(archive_path)?;

    if entries.is_empty() {
        println!();
        println!("  {} No skills found in archive", "○".dimmed());
        println!();
        return Ok(());
    }

    let (imported, overwritten, skipped) = write_archive_entries(entries, skills_dir, yes)?;

    println!();
    if imported > 0 {
        log::success(format!(
            "Imported {} new skill{}",
            imported,
            if imported == 1 { "" } else { "s" }
        ))
        .into_diagnostic()?;
    }
    if overwritten > 0 {
        log::success(format!(
            "Overwrote {} existing skill{}",
            overwritten,
            if overwritten == 1 { "" } else { "s" }
        ))
        .into_diagnostic()?;
    }
    if skipped > 0 {
        log::info(format!(
            "Skipped {} skill{} (kept existing)",
            skipped,
            if skipped == 1 { "" } else { "s" }
        ))
        .into_diagnostic()?;
    }
    if imported > 0 || overwritten > 0 {
        log::info("Remember to run `wai sync` to update agent config").into_diagnostic()?;
    }
    println!();

    Ok(())
}

/// Resolve the SKILL.md path for a named skill within a skills directory.
fn skill_md_path(skills_dir: &Path, name: &str) -> std::path::PathBuf {
    skills_dir.join(name).join("SKILL.md")
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::Compression;
    use flate2::read::GzDecoder;
    use flate2::write::GzEncoder;
    use std::io::BufReader;

    fn make_skill_dir(dir: &Path, name: &str, content: &str) {
        // name may be hierarchical (cat/action)
        let skill_dir = dir.join(name);
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), content).unwrap();
    }

    // ── archive path validation ──────────────────────────────────────────────

    #[test]
    fn test_valid_archive_paths() {
        assert!(validate_archive_entry_path("my-skill/SKILL.md").is_ok());
        assert!(validate_archive_entry_path("issue/gather/SKILL.md").is_ok());
        assert!(validate_archive_entry_path("a/SKILL.md").is_ok());
    }

    #[test]
    fn test_archive_path_traversal_rejected() {
        let result = validate_archive_entry_path("../etc/passwd");
        assert!(result.is_err());
        let result = validate_archive_entry_path("../bad/SKILL.md");
        assert!(result.is_err());
        let result = validate_archive_entry_path("good/../SKILL.md");
        assert!(result.is_err());
    }

    #[test]
    fn test_archive_path_must_end_with_skill_md() {
        let result = validate_archive_entry_path("my-skill/README.md");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("SKILL.md"));
    }

    #[test]
    fn test_archive_path_depth_too_deep() {
        let result = validate_archive_entry_path("a/b/c/SKILL.md");
        assert!(result.is_err());
    }

    #[test]
    fn test_archive_path_absolute() {
        // An absolute path component should be rejected via skill name validation
        let result = validate_archive_entry_path("/etc/SKILL.md");
        assert!(result.is_err());
    }

    // ── export → import round-trip ───────────────────────────────────────────

    #[test]
    fn test_export_import_round_trip() {
        let src_skills = tempfile::tempdir().unwrap();
        let dst_skills = tempfile::tempdir().unwrap();
        let archive_dir = tempfile::tempdir().unwrap();

        let content = "---\nname: gather\ndescription: Gather skill\n---\n\nInstructions.\n";
        make_skill_dir(src_skills.path(), "gather", content);

        // Export
        let archive_path = archive_dir.path().join("skills.tar.gz");
        let archive_path_str = archive_path.to_string_lossy().to_string();

        let out_file = fs::File::create(&archive_path).unwrap();
        let gz = GzEncoder::new(out_file, Compression::default());
        let mut builder = tar::Builder::new(gz);
        let skill_md = src_skills.path().join("gather").join("SKILL.md");
        builder
            .append_path_with_name(&skill_md, "gather/SKILL.md")
            .unwrap();
        builder.finish().unwrap();
        let gz_encoder = builder.into_inner().unwrap();
        gz_encoder.finish().unwrap();

        // Validate paths in archive
        assert!(validate_archive_entry_path("gather/SKILL.md").is_ok());

        // Import (simulate)
        let in_file = fs::File::open(&archive_path).unwrap();
        let gz = GzDecoder::new(BufReader::new(in_file));
        let mut archive = tar::Archive::new(gz);
        for entry in archive.entries().unwrap() {
            let mut entry = entry.unwrap();
            let entry_path = entry.path().unwrap().to_string_lossy().to_string();
            if entry.header().entry_type().is_dir() {
                continue;
            }
            validate_archive_entry_path(&entry_path).unwrap();
            let skill_name = entry_path.trim_end_matches("/SKILL.md");
            let dst_dir = dst_skills.path().join(skill_name);
            fs::create_dir_all(&dst_dir).unwrap();
            let mut content_bytes = Vec::new();
            std::io::copy(&mut entry, &mut content_bytes).unwrap();
            fs::write(dst_dir.join("SKILL.md"), &content_bytes).unwrap();
        }

        // Verify content preserved
        let imported_content =
            fs::read_to_string(dst_skills.path().join("gather").join("SKILL.md")).unwrap();
        assert_eq!(imported_content, content);
        let _ = archive_path_str;
    }

    #[test]
    fn test_export_import_hierarchical_round_trip() {
        let src_skills = tempfile::tempdir().unwrap();
        let dst_skills = tempfile::tempdir().unwrap();
        let archive_dir = tempfile::tempdir().unwrap();

        let content =
            "---\nname: issue/gather\ndescription: Issue gather skill\n---\n\nInstructions.\n";
        make_skill_dir(src_skills.path(), "issue/gather", content);

        let archive_path = archive_dir.path().join("skills.tar.gz");

        let out_file = fs::File::create(&archive_path).unwrap();
        let gz = GzEncoder::new(out_file, Compression::default());
        let mut builder = tar::Builder::new(gz);
        let skill_md = src_skills
            .path()
            .join("issue")
            .join("gather")
            .join("SKILL.md");
        builder
            .append_path_with_name(&skill_md, "issue/gather/SKILL.md")
            .unwrap();
        builder.finish().unwrap();
        let gz_encoder = builder.into_inner().unwrap();
        gz_encoder.finish().unwrap();

        // Validate hierarchical path
        assert!(validate_archive_entry_path("issue/gather/SKILL.md").is_ok());

        // Import
        let in_file = fs::File::open(&archive_path).unwrap();
        let gz = GzDecoder::new(BufReader::new(in_file));
        let mut archive = tar::Archive::new(gz);
        for entry in archive.entries().unwrap() {
            let mut entry = entry.unwrap();
            let entry_path = entry.path().unwrap().to_string_lossy().to_string();
            if entry.header().entry_type().is_dir() {
                continue;
            }
            validate_archive_entry_path(&entry_path).unwrap();
            let skill_name = entry_path.trim_end_matches("/SKILL.md");
            let dst_dir = dst_skills.path().join(skill_name);
            fs::create_dir_all(&dst_dir).unwrap();
            let mut content_bytes = Vec::new();
            std::io::copy(&mut entry, &mut content_bytes).unwrap();
            fs::write(dst_dir.join("SKILL.md"), &content_bytes).unwrap();
        }

        let imported = fs::read_to_string(
            dst_skills
                .path()
                .join("issue")
                .join("gather")
                .join("SKILL.md"),
        )
        .unwrap();
        assert_eq!(imported, content);
    }

    // ── import archive --yes overwrites ──────────────────────────────────────

    #[test]
    fn test_import_yes_overwrites_existing_skill() {
        let src_skills = tempfile::tempdir().unwrap();
        let dst_skills = tempfile::tempdir().unwrap();
        let archive_dir = tempfile::tempdir().unwrap();

        let original = "---\nname: gather\ndescription: Original\n---\n\nOriginal.\n";
        let updated = "---\nname: gather\ndescription: Updated\n---\n\nUpdated.\n";

        // Write original to destination
        make_skill_dir(dst_skills.path(), "gather", original);

        // Create archive with updated content
        make_skill_dir(src_skills.path(), "gather", updated);
        let archive_path = archive_dir.path().join("skills.tar.gz");
        let out_file = fs::File::create(&archive_path).unwrap();
        let gz = GzEncoder::new(out_file, Compression::default());
        let mut builder = tar::Builder::new(gz);
        let skill_md = src_skills.path().join("gather").join("SKILL.md");
        builder
            .append_path_with_name(&skill_md, "gather/SKILL.md")
            .unwrap();
        builder.finish().unwrap();
        let gz_encoder = builder.into_inner().unwrap();
        gz_encoder.finish().unwrap();

        // Import with yes=true
        let entries = read_archive_entries(archive_path.to_str().unwrap()).unwrap();
        let (imported, overwritten, skipped) =
            write_archive_entries(entries, dst_skills.path(), true).unwrap();

        assert_eq!(imported, 0);
        assert_eq!(overwritten, 1);
        assert_eq!(skipped, 0);

        let result = fs::read_to_string(dst_skills.path().join("gather").join("SKILL.md")).unwrap();
        assert_eq!(result, updated);
    }
}
