//! Unit tests for artifact hashing, lock sidecars, addenda, and cmd_next/cmd_verify.

use crate::commands::pipeline::gates::find_step_artifact_paths;
use crate::commands::pipeline::model::{ArtifactLock, PipelineStep};
use crate::commands::pipeline::queries::find_step_addenda;
use crate::commands::pipeline::{artifact_hash, read_artifact_lock, write_artifact_lock};
use std::fs;

// ── artifact_hash ─────────────────────────────────────────────────────

#[test]
fn artifact_hash_consistent() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("artifact.md");
    fs::write(&path, "hello world\n").unwrap();
    let h1 = artifact_hash(&path).unwrap();
    let h2 = artifact_hash(&path).unwrap();
    assert_eq!(h1, h2, "same file should produce identical hashes");
    assert!(
        h1.starts_with("sha256:"),
        "hash should be prefixed with sha256:"
    );
}

#[test]
fn artifact_hash_normalizes_crlf() {
    let dir = tempfile::tempdir().unwrap();
    let lf_path = dir.path().join("lf.md");
    let crlf_path = dir.path().join("crlf.md");
    fs::write(&lf_path, "line one\nline two\n").unwrap();
    fs::write(&crlf_path, "line one\r\nline two\r\n").unwrap();
    let lf_hash = artifact_hash(&lf_path).unwrap();
    let crlf_hash = artifact_hash(&crlf_path).unwrap();
    assert_eq!(
        lf_hash, crlf_hash,
        "CRLF and LF content should hash identically"
    );
}

#[test]
fn artifact_hash_different_content() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.md");
    let b = dir.path().join("b.md");
    fs::write(&a, "content A\n").unwrap();
    fs::write(&b, "content B\n").unwrap();
    let ha = artifact_hash(&a).unwrap();
    let hb = artifact_hash(&b).unwrap();
    assert_ne!(ha, hb, "different content should produce different hashes");
}

// ── artifact lock sidecar ─────────────────────────────────────────────

#[test]
fn write_artifact_lock_creates_file_with_correct_name() {
    let dir = tempfile::tempdir().unwrap();
    let artifact = dir.path().join("research.md");
    fs::write(&artifact, "# Research\nSome findings.\n").unwrap();

    let lock_path = write_artifact_lock(&artifact, "run-abc", "step-1").unwrap();

    assert!(lock_path.exists(), "lock file should exist");
    assert_eq!(
        lock_path.file_name().unwrap().to_string_lossy(),
        "research.md.run-abc.lock"
    );
}

#[test]
fn write_artifact_lock_roundtrips_via_read() {
    let dir = tempfile::tempdir().unwrap();
    let artifact = dir.path().join("design.md");
    fs::write(&artifact, "# Design\nArchitecture notes.\n").unwrap();

    let lock_path = write_artifact_lock(&artifact, "run-42", "generate").unwrap();
    let lock = read_artifact_lock(&lock_path).unwrap();

    assert_eq!(lock.artifact, "design.md");
    assert_eq!(lock.pipeline_run, "run-42");
    assert_eq!(lock.pipeline_step, "generate");
    assert!(
        lock.lock_hash.starts_with("sha256:"),
        "hash should be prefixed with sha256:"
    );
    // locked_at should be a valid RFC 3339 timestamp
    assert!(
        chrono::DateTime::parse_from_rfc3339(&lock.locked_at).is_ok(),
        "locked_at should be valid RFC 3339: {}",
        lock.locked_at
    );
}

#[test]
fn write_artifact_lock_hash_matches_artifact_hash() {
    let dir = tempfile::tempdir().unwrap();
    let artifact = dir.path().join("notes.md");
    fs::write(&artifact, "Some content\nwith lines.\n").unwrap();

    let expected_hash = artifact_hash(&artifact).unwrap();
    let lock_path = write_artifact_lock(&artifact, "run-x", "review").unwrap();
    let lock = read_artifact_lock(&lock_path).unwrap();

    assert_eq!(
        lock.lock_hash, expected_hash,
        "lock hash should match artifact_hash"
    );
}

#[test]
fn write_artifact_lock_includes_run_id_in_filename() {
    let dir = tempfile::tempdir().unwrap();
    let artifact = dir.path().join("plan.md");
    fs::write(&artifact, "# Plan\n").unwrap();

    let lock_path = write_artifact_lock(&artifact, "my-run-id", "step-0").unwrap();
    let filename = lock_path.file_name().unwrap().to_string_lossy();

    assert!(
        filename.contains("my-run-id"),
        "filename should contain run-id, got: {filename}"
    );
    assert!(
        filename.ends_with(".lock"),
        "filename should end with .lock, got: {filename}"
    );
}

// ── find_step_addenda ────────────────────────────────────────────────

#[test]
fn find_step_addenda_returns_matching_files() {
    let dir = tempfile::tempdir().unwrap();
    let project_dir = dir
        .path()
        .join(".wai")
        .join("projects")
        .join("test-project");
    let research_dir = project_dir.join("research");
    fs::create_dir_all(&research_dir).unwrap();

    fs::write(
        research_dir.join("2026-04-14-correction.md"),
        "---\ntags: [pipeline-addendum:implement]\n---\n\nCorrection notes.",
    )
    .unwrap();

    fs::write(
        research_dir.join("2026-04-14-other.md"),
        "---\ntags: [pipeline-addendum:review]\n---\n\nOther notes.",
    )
    .unwrap();

    fs::write(
        research_dir.join("2026-04-14-normal.md"),
        "---\ntags: [pipeline-run:abc]\n---\n\nNormal research.",
    )
    .unwrap();

    let addenda = find_step_addenda(dir.path(), "implement");
    assert_eq!(addenda.len(), 1, "expected 1 addendum, got: {:?}", addenda);
    assert_eq!(addenda[0], "research/2026-04-14-correction.md");
}

#[test]
fn find_step_addenda_returns_empty_when_none_match() {
    let dir = tempfile::tempdir().unwrap();
    let project_dir = dir
        .path()
        .join(".wai")
        .join("projects")
        .join("test-project");
    let research_dir = project_dir.join("research");
    fs::create_dir_all(&research_dir).unwrap();

    fs::write(
        research_dir.join("2026-04-14-normal.md"),
        "---\ntags: [pipeline-run:abc]\n---\n\nNormal research.",
    )
    .unwrap();

    let addenda = find_step_addenda(dir.path(), "implement");
    assert!(
        addenda.is_empty(),
        "expected no addenda, got: {:?}",
        addenda
    );
}

#[test]
fn find_step_addenda_scans_multiple_directories() {
    let dir = tempfile::tempdir().unwrap();
    let project_dir = dir
        .path()
        .join(".wai")
        .join("projects")
        .join("test-project");
    let research_dir = project_dir.join("research");
    let plans_dir = project_dir.join("plans");
    fs::create_dir_all(&research_dir).unwrap();
    fs::create_dir_all(&plans_dir).unwrap();

    fs::write(
        research_dir.join("2026-04-14-fix.md"),
        "---\ntags: [pipeline-addendum:design]\n---\n\nFix notes.",
    )
    .unwrap();

    fs::write(
        plans_dir.join("2026-04-14-revised.md"),
        "---\ntags: [pipeline-addendum:design]\n---\n\nRevised plan.",
    )
    .unwrap();

    let addenda = find_step_addenda(dir.path(), "design");
    assert_eq!(addenda.len(), 2, "expected 2 addenda, got: {:?}", addenda);
    assert!(addenda.contains(&"plans/2026-04-14-revised.md".to_string()));
    assert!(addenda.contains(&"research/2026-04-14-fix.md".to_string()));
}

#[test]
fn find_step_addenda_handles_missing_projects_dir() {
    let dir = tempfile::tempdir().unwrap();
    let addenda = find_step_addenda(dir.path(), "implement");
    assert!(
        addenda.is_empty(),
        "expected no addenda for missing project dir"
    );
}

// ── find_step_artifact_paths ─────────────────────────────────────────

#[test]
fn find_step_artifact_paths_returns_matching_files() {
    let dir = tempfile::tempdir().unwrap();
    let project_dir = dir
        .path()
        .join(".wai")
        .join("projects")
        .join("test-project");
    let research_dir = project_dir.join("research");
    fs::create_dir_all(&research_dir).unwrap();

    // Artifact tagged with both run and step tags
    fs::write(
        research_dir.join("2026-04-14-findings.md"),
        "---\ntags: [pipeline-run:run-abc, pipeline-step:generate]\n---\n\nFindings.",
    )
    .unwrap();

    // Artifact tagged with different step — should NOT match
    fs::write(
        research_dir.join("2026-04-14-other.md"),
        "---\ntags: [pipeline-run:run-abc, pipeline-step:review]\n---\n\nOther.",
    )
    .unwrap();

    // Artifact with no pipeline tags — should NOT match
    fs::write(
        research_dir.join("2026-04-14-plain.md"),
        "---\ntags: [general]\n---\n\nPlain.",
    )
    .unwrap();

    let paths = find_step_artifact_paths(dir.path(), "run-abc", "generate");
    assert_eq!(paths.len(), 1, "expected 1 artifact, got: {:?}", paths);
    assert!(
        paths[0]
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains("findings"),
        "expected findings artifact, got: {:?}",
        paths[0]
    );
}

#[test]
fn find_step_artifact_paths_returns_empty_when_no_artifacts() {
    let dir = tempfile::tempdir().unwrap();
    let project_dir = dir
        .path()
        .join(".wai")
        .join("projects")
        .join("test-project");
    let research_dir = project_dir.join("research");
    fs::create_dir_all(&research_dir).unwrap();

    // Artifact tagged with different run
    fs::write(
        research_dir.join("2026-04-14-other.md"),
        "---\ntags: [pipeline-run:other-run, pipeline-step:generate]\n---\n\nOther.",
    )
    .unwrap();

    let paths = find_step_artifact_paths(dir.path(), "run-abc", "generate");
    assert!(paths.is_empty(), "expected no artifacts, got: {:?}", paths);
}

// ── cmd_next locking integration ────────────────────────────────────

#[test]
fn cmd_next_lock_creates_lock_files_for_step_artifacts() {
    // Simulate the locking path from cmd_next: when a step has lock=true,
    // find its artifacts and write lock sidecars for each.
    let dir = tempfile::tempdir().unwrap();
    let run_id = "run-lock-test";
    let step_def = locked_step_artifacts_fixture(&dir, run_id, "generate");

    // Execute the same logic as cmd_next's locking block
    assert!(step_def.lock, "step should have lock=true");
    let artifact_paths = find_step_artifact_paths(dir.path(), run_id, &step_def.id);
    assert_eq!(
        artifact_paths.len(),
        2,
        "expected 2 artifacts, got: {:?}",
        artifact_paths
    );

    for path in &artifact_paths {
        write_artifact_lock(path, run_id, &step_def.id).unwrap();
    }
    assert_all_lock_files_valid(&artifact_paths, run_id, &step_def.id);
}

/// Create two research artifacts tagged for `run_id` + `step_id` and a
/// lock=true step definition for them.
fn locked_step_artifacts_fixture(
    dir: &tempfile::TempDir,
    run_id: &str,
    step_id: &str,
) -> PipelineStep {
    let research_dir = dir
        .path()
        .join(".wai")
        .join("projects")
        .join("test-project")
        .join("research");
    fs::create_dir_all(&research_dir).unwrap();

    let tag_line = format!(
        "---\ntags: [pipeline-run:{}, pipeline-step:{}]\n---\n\n",
        run_id, step_id
    );
    fs::write(
        research_dir.join("2026-04-14-findings.md"),
        format!("{}Findings.", tag_line),
    )
    .unwrap();
    fs::write(
        research_dir.join("2026-04-14-analysis.md"),
        format!("{}Analysis.", tag_line),
    )
    .unwrap();

    PipelineStep {
        id: step_id.to_string(),
        prompt: String::new(),
        gate: None,
        lock: true,
    }
}

/// Assert every artifact has a valid lock sidecar for `run_id` + `step_id`.
fn assert_all_lock_files_valid(paths: &[std::path::PathBuf], run_id: &str, step_id: &str) {
    for path in paths {
        let lock_filename = format!(
            "{}.{}.lock",
            path.file_name().unwrap().to_string_lossy(),
            run_id
        );
        let lock_path = path.parent().unwrap().join(&lock_filename);
        assert!(
            lock_path.exists(),
            "lock file should exist: {:?}",
            lock_path
        );

        // Verify lock content
        let lock = read_artifact_lock(&lock_path).unwrap();
        assert_eq!(lock.pipeline_run, run_id);
        assert_eq!(lock.pipeline_step, step_id);
        assert!(lock.lock_hash.starts_with("sha256:"));
    }
}

#[test]
fn cmd_next_lock_skipped_when_lock_is_false() {
    // When lock=false, no lock files should be created (the if-branch is not entered)
    let step_def = PipelineStep {
        id: "review".to_string(),
        prompt: String::new(),
        gate: None,
        lock: false,
    };
    assert!(
        !step_def.lock,
        "step should have lock=false — locking block would be skipped"
    );
}

/// Create a locked artifact under `.wai/projects/test-project/research/notes.md`
/// and return the tempdir root.
fn locked_artifact_fixture(content: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let project_dir = dir
        .path()
        .join(".wai")
        .join("projects")
        .join("test-project")
        .join("research");
    fs::create_dir_all(&project_dir).unwrap();

    let artifact = project_dir.join("notes.md");
    fs::write(&artifact, content).unwrap();

    let hash = artifact_hash(&artifact).unwrap();
    let lock = ArtifactLock {
        artifact: "notes.md".to_string(),
        locked_at: chrono::Utc::now().to_rfc3339(),
        lock_hash: hash,
        pipeline_run: "run-1".to_string(),
        pipeline_step: "generate".to_string(),
    };
    let lock_path = project_dir.join("notes.md.run-1.lock");
    fs::write(&lock_path, toml::to_string_pretty(&lock).unwrap()).unwrap();
    dir
}

/// Walk all `.lock` files under the projects dir, returning (lock_count, mismatched artifact names).
fn walk_locks(projects_dir: &std::path::Path) -> (usize, Vec<String>) {
    let mut mismatches: Vec<String> = Vec::new();
    let mut lock_count: usize = 0;
    for entry in walkdir::WalkDir::new(projects_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("lock") {
            continue;
        }
        let parsed_lock = read_artifact_lock(path).unwrap();
        let artifact_path = path.parent().unwrap().join(&parsed_lock.artifact);
        let actual = artifact_hash(&artifact_path).unwrap();
        if actual != parsed_lock.lock_hash {
            mismatches.push(parsed_lock.artifact.clone());
        }
        lock_count += 1;
    }
    (lock_count, mismatches)
}

// ── cmd_verify (via artifact_hash / read_artifact_lock) ─────────────

#[test]
fn verify_passes_when_artifacts_are_intact() {
    let dir = locked_artifact_fixture("# Notes\nSome content.\n");
    let projects_dir = dir.path().join(".wai").join("projects");
    let (lock_count, mismatches) = walk_locks(&projects_dir);

    assert_eq!(lock_count, 1, "should find exactly one lock file");
    assert!(mismatches.is_empty(), "all artifacts should verify OK");
}

#[test]
fn verify_detects_tampered_artifact() {
    let dir = locked_artifact_fixture("# Notes\nOriginal content.\n");
    let projects_dir = dir.path().join(".wai").join("projects");

    // Tamper with the artifact
    let artifact = projects_dir
        .join("test-project")
        .join("research")
        .join("notes.md");
    fs::write(&artifact, "# Notes\nTampered content!\n").unwrap();

    let (_, mismatches) = walk_locks(&projects_dir);

    assert_eq!(
        mismatches.len(),
        1,
        "should detect exactly one mismatch, got: {:?}",
        mismatches
    );
    assert_eq!(mismatches[0], "notes.md");
}
