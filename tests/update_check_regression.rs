//! Regression tests for the crates.io update-check wiring (wai-khgi).
//!
//! Exercises genesis's `check_with` through the same code path the binary
//! uses (`maybe_notify_update` in main.rs), with a hermetic cache dir.

use genesis::update_check;
use std::path::PathBuf;

/// Fixture: fresh (non-expired) cache entry whose `latest` predates the
/// installed version — the exact scenario that made wai 2026.10.4 print
/// "wai-cli 2026.9.28 available — you have 2026.10.4".
fn seed_stale_latest_cache(cache_dir: &std::path::Path, latest: &str) {
    let entry = format!(
        r#"{{"checked_at":{},"latest":"{}","published_at":"2026-09-28T20:37:04.183514Z","ttl_secs":604800}}"#,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        latest,
    );
    std::fs::create_dir_all(cache_dir).expect("create cache dir");
    std::fs::write(cache_dir.join("wai-cli.json"), entry).expect("write cache fixture");
}

fn tmp_cache_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("wai-update-check-tests")
        .join(format!("{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// Installed version NEWER than cached latest → no notice (no downgrade
/// suggestion). Regression for genesis-u47; red against genesis-vibes 0.11.x.
#[test]
fn no_notice_when_installed_version_newer_than_cached_latest() {
    let cache_dir = tmp_cache_dir("newer-installed");
    seed_stale_latest_cache(&cache_dir, "2026.9.28");

    let info = update_check::check_with(
        "wai-cli",
        "2026.10.4",
        &cache_dir,
        update_check::CRATES_IO_API,
    );
    assert!(
        info.is_none(),
        "must not suggest a downgrade: cached latest 2026.9.28 vs installed 2026.10.4 → {info:?}"
    );
}

/// Installed version older than cached latest → notice IS produced.
#[test]
fn notice_when_cached_latest_is_newer() {
    let cache_dir = tmp_cache_dir("older-installed");
    seed_stale_latest_cache(&cache_dir, "2026.10.4");

    let info = update_check::check_with(
        "wai-cli",
        "2026.9.28",
        &cache_dir,
        update_check::CRATES_IO_API,
    );
    let info = info.expect("newer cached latest must produce UpdateInfo");
    assert_eq!(info.latest, "2026.10.4");
    assert_eq!(info.current, "2026.9.28");
}

/// Equal versions → no notice (fresh-cache short-circuit, honors TTL).
#[test]
fn no_notice_when_versions_equal() {
    let cache_dir = tmp_cache_dir("equal");
    seed_stale_latest_cache(&cache_dir, "2026.10.4");

    let info = update_check::check_with(
        "wai-cli",
        "2026.10.4",
        &cache_dir,
        update_check::CRATES_IO_API,
    );
    assert!(info.is_none(), "equal versions must not notify: {info:?}");
}
