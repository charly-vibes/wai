use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

#[derive(Debug, Serialize, Deserialize)]
pub struct FreshnessSidecar {
    pub artifact: String,
    pub verified_at: String,
    pub tracked: Vec<TrackedEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackedEntry {
    pub path: String,
    pub mtime: u64,
    pub hash: String,
}

pub fn write_sidecar(artifact_path: &Path, repo_root: &Path, tracks: &[String]) {
    let mut entries = Vec::new();
    for track in tracks {
        let abs = repo_root.join(track);
        if abs.exists() {
            entries.push(TrackedEntry {
                path: track.clone(),
                mtime: mtime_secs(&abs),
                hash: hash_file(&abs),
            });
        } else {
            entries.push(TrackedEntry {
                path: track.clone(),
                mtime: 0,
                hash: "missing".to_string(),
            });
        }
    }

    let sidecar = FreshnessSidecar {
        artifact: artifact_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default(),
        verified_at: Utc::now().to_rfc3339(),
        tracked: entries,
    };

    let sidecar_path = sidecar_path_for(artifact_path);
    if let Ok(toml_str) = toml::to_string(&sidecar) {
        let _ = fs::write(sidecar_path, toml_str);
    }
}

pub fn read_sidecar(artifact_path: &Path) -> Option<FreshnessSidecar> {
    let path = sidecar_path_for(artifact_path);
    let content = fs::read_to_string(path).ok()?;
    toml::from_str(&content).ok()
}

pub fn sidecar_path_for(artifact_path: &Path) -> PathBuf {
    let stem = artifact_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    artifact_path
        .parent()
        .unwrap_or(Path::new("."))
        .join(format!("{}.fresh.lock", stem))
}

pub fn is_stale(repo_root: &Path, sidecar: &FreshnessSidecar) -> (bool, Vec<String>) {
    let mut changed = Vec::new();
    for entry in &sidecar.tracked {
        let abs = repo_root.join(&entry.path);
        if !abs.exists() {
            changed.push(entry.path.clone());
            continue;
        }
        if hash_file(&abs) != entry.hash {
            changed.push(entry.path.clone());
        }
    }
    (!changed.is_empty(), changed)
}

fn mtime_secs(path: &Path) -> u64 {
    fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn hash_file(path: &Path) -> String {
    let content = fs::read(path).unwrap_or_default();
    let normalized: Vec<u8> = content
        .split(|&b| b == b'\r')
        .flat_map(|chunk| chunk.iter().copied())
        .collect();
    let mut hasher = Sha256::new();
    hasher.update(&normalized);
    format!("sha256:{:x}", hasher.finalize())
}

// ── Frontmatter parsing ───────────────────────────────────────────────────────

pub fn parse_tracks_from_frontmatter(content: &str) -> Vec<String> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return vec![];
    }
    let rest = &content[3..];
    let end = match rest.find("\n---") {
        Some(i) => i,
        None => return vec![],
    };
    parse_tracks_yaml(&rest[..end])
}

fn parse_tracks_yaml(frontmatter: &str) -> Vec<String> {
    let mut in_tracks = false;
    let mut tracks = Vec::new();

    for line in frontmatter.lines() {
        if line.trim_start().starts_with("tracks:") {
            let after_colon = line.trim_start().trim_start_matches("tracks:").trim();
            if after_colon.starts_with('[') {
                let inner = after_colon.trim_matches(|c| c == '[' || c == ']');
                tracks = inner
                    .split(',')
                    .map(|s| s.trim().trim_matches('"').to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                return tracks;
            }
            in_tracks = true;
            continue;
        }
        if in_tracks {
            if line.starts_with(' ') || line.starts_with('\t') {
                let item = line.trim().trim_start_matches('-').trim();
                if !item.is_empty() {
                    tracks.push(item.trim_matches('"').to_string());
                }
            } else {
                in_tracks = false;
            }
        }
    }
    tracks
}

// ── Scanner ───────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct FreshnessReport {
    pub stale: Vec<StaleEntry>,
    pub untracked: Vec<String>,
    pub clean: usize,
    pub stale_count: usize,
}

#[derive(Debug, Serialize)]
pub struct StaleEntry {
    pub artifact: String,
    pub decision_point: Option<String>,
    pub changed_paths: Vec<String>,
}

pub fn scan_freshness(project_root: &Path) -> FreshnessReport {
    let projects_dir = project_root.join(".wai/projects");
    let artifact_subdirs = ["research", "design", "plans"];

    let mut stale = Vec::new();
    let mut untracked = Vec::new();
    let mut clean = 0usize;

    let Ok(project_entries) = fs::read_dir(&projects_dir) else {
        return FreshnessReport {
            stale,
            untracked,
            clean,
            stale_count: 0,
        };
    };

    for proj_entry in project_entries.filter_map(|e| e.ok()) {
        let proj_path = proj_entry.path();
        if !proj_path.is_dir() {
            continue;
        }
        for subdir in &artifact_subdirs {
            let art_dir = proj_path.join(subdir);
            let Ok(art_entries) = fs::read_dir(&art_dir) else {
                continue;
            };
            for art_entry in art_entries.filter_map(|e| e.ok()) {
                let path = art_entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("md") {
                    continue;
                }
                let Ok(content) = fs::read_to_string(&path) else {
                    continue;
                };
                let tracks = parse_tracks_from_frontmatter(&content);
                if tracks.is_empty() {
                    continue;
                }
                match read_sidecar(&path) {
                    None => {
                        untracked.push(path.to_string_lossy().to_string());
                    }
                    Some(sc) => {
                        let (stale_flag, changed) = is_stale(project_root, &sc);
                        if stale_flag {
                            stale.push(StaleEntry {
                                artifact: path.to_string_lossy().to_string(),
                                decision_point: parse_decision_point(&content),
                                changed_paths: changed,
                            });
                        } else {
                            clean += 1;
                        }
                    }
                }
            }
        }
    }

    let stale_count = stale.len();
    FreshnessReport {
        stale,
        untracked,
        clean,
        stale_count,
    }
}

fn parse_decision_point(content: &str) -> Option<String> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return None;
    }
    let rest = &content[3..];
    let end = rest.find("\n---")?;
    let frontmatter = &rest[..end];
    for line in frontmatter.lines() {
        if line.trim_start().starts_with("decision_point:") {
            let val = line
                .trim_start()
                .trim_start_matches("decision_point:")
                .trim()
                .trim_matches('"');
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}
