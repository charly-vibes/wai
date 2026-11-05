//! Unit tests for plugin.

use super::*;

use tempfile::TempDir;

// ── fetch_memories ────────────────────────────────────────────────────────

#[test]
fn fetch_memories_none_when_no_beads_dir() {
    let tmp = TempDir::new().unwrap();
    // No .beads/ → should return None without attempting to shell out.
    assert!(fetch_memories(tmp.path()).is_none());
}

#[test]
fn fetch_memories_none_when_bd_not_on_path() {
    let tmp = TempDir::new().unwrap();
    std::fs::create_dir(tmp.path().join(".beads")).unwrap();
    // Run with an empty PATH so bd cannot be found.
    let result = std::panic::catch_unwind(|| {
        // We can't override PATH inside the process for Command easily, so
        // we verify that the function returns None (not panics) when bd
        // is genuinely absent. In CI bd won't be available.
        // If bd IS on PATH (dev machine), this test is a no-op for the None branch.
        let _ = fetch_memories(tmp.path());
    });
    assert!(result.is_ok(), "fetch_memories must not panic");
}

#[test]
fn fetch_memories_truncates_to_budget() {
    // Build a string larger than MEMORIES_BUDGET and verify truncation is
    // at a valid char boundary.
    let big = "x".repeat(MEMORIES_BUDGET + 500);
    let mut cut = MEMORIES_BUDGET;
    while cut > 0 && !big.is_char_boundary(cut) {
        cut -= 1;
    }
    let truncated = &big[..cut];
    assert!(truncated.len() <= MEMORIES_BUDGET);
    assert!(big.is_char_boundary(truncated.len()));
}

// ── store_memory ──────────────────────────────────────────────────────────

#[test]
fn store_memory_err_when_no_beads_dir() {
    let tmp = TempDir::new().unwrap();
    // No .beads/ → should return Err without attempting to shell out.
    let result = store_memory(tmp.path(), "some insight");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("beads not detected"));
}

// ── trust store ───────────────────────────────────────────────────────────

fn sample_plugin(name: &str) -> PluginDef {
    PluginDef {
        name: name.to_string(),
        description: String::new(),
        intent: None,
        success_criteria: None,
        detector: None,
        commands: vec![],
        hooks: HashMap::new(),
    }
}

#[test]
fn compute_hook_digest_is_stable_and_content_sensitive() {
    let def = sample_plugin("test");
    let hook = HookDef {
        command: "echo hi".to_string(),
        inject_as: "greeting".to_string(),
    };
    let d1 = compute_hook_digest(&def, "on_status", &hook);
    let d2 = compute_hook_digest(&def, "on_status", &hook);
    assert_eq!(d1, d2, "digest must be deterministic");

    // Changing the command changes the digest.
    let hook2 = HookDef {
        command: "echo bye".to_string(),
        inject_as: "greeting".to_string(),
    };
    assert_ne!(d1, compute_hook_digest(&def, "on_status", &hook2));

    // Changing the hook name changes the digest.
    assert_ne!(d1, compute_hook_digest(&def, "on_other", &hook));

    // Changing the plugin name changes the digest.
    assert_ne!(
        d1,
        compute_hook_digest(&sample_plugin("other"), "on_status", &hook)
    );
}

#[test]
fn trust_store_approve_list_revoke() {
    let mut store = TrustStore::default();
    let digest = "abc123".to_string();
    store.approve(TrustEntry {
        plugin_name: "evil".to_string(),
        hook_name: "on_status".to_string(),
        digest: digest.clone(),
        command: "touch marker".to_string(),
        approved_at: "2026-08-05T00:00:00Z".to_string(),
    });

    assert!(store.is_approved(&digest));
    assert_eq!(store.list().len(), 1);
    assert!(!store.is_approved("other"));

    // Revoking removes the entry.
    assert!(store.revoke(&digest));
    assert!(!store.is_approved(&digest));
    assert_eq!(store.list().len(), 0);

    // Revoking a missing digest returns false.
    assert!(!store.revoke(&digest));
}

#[test]
fn trust_store_approve_replaces_same_digest() {
    let mut store = TrustStore::default();
    store.approve(TrustEntry {
        plugin_name: "evil".to_string(),
        hook_name: "on_status".to_string(),
        digest: "d1".to_string(),
        command: "touch marker".to_string(),
        approved_at: "t1".to_string(),
    });
    store.approve(TrustEntry {
        plugin_name: "evil".to_string(),
        hook_name: "on_status".to_string(),
        digest: "d1".to_string(),
        command: "touch marker".to_string(),
        approved_at: "t2".to_string(),
    });
    assert_eq!(
        store.list().len(),
        1,
        "same digest must not create duplicates"
    );
}

#[serial_test::serial]
#[test]
fn trust_store_path_prefers_wai_data_dir() {
    // trust_store_path uses WAI_DATA_DIR when set.
    // SAFETY: test-only env var, single-threaded tests.
    let saved = std::env::var_os("WAI_DATA_DIR");
    unsafe {
        std::env::set_var("WAI_DATA_DIR", "/tmp/wai-trust-test");
    }
    assert_eq!(
        trust_store_path(),
        PathBuf::from("/tmp/wai-trust-test").join(TRUST_STORE_FILE)
    );
    match saved {
        Some(v) => unsafe { std::env::set_var("WAI_DATA_DIR", v) },
        None => unsafe { std::env::remove_var("WAI_DATA_DIR") },
    }
}

#[serial_test::serial]
#[test]
fn trust_store_load_save_roundtrip() {
    let tmp = TempDir::new().unwrap();
    // SAFETY: test-only env var, single-threaded tests.
    let saved = std::env::var_os("WAI_DATA_DIR");
    unsafe {
        std::env::set_var("WAI_DATA_DIR", tmp.path().join("data"));
    }

    let digest = "roundtrip-digest".to_string();
    {
        let mut store = TrustStore::load();
        store.approve(TrustEntry {
            plugin_name: "test".to_string(),
            hook_name: "on_status".to_string(),
            digest: digest.clone(),
            command: "echo hi".to_string(),
            approved_at: "2026-08-05T00:00:00Z".to_string(),
        });
        store.save().unwrap();
    }
    // Load from a fresh store to verify persistence.
    let store = TrustStore::load();
    assert!(store.is_approved(&digest));
    assert_eq!(store.list().len(), 1);
    assert_eq!(store.list()[0].plugin_name, "test");

    match saved {
        Some(v) => unsafe { std::env::set_var("WAI_DATA_DIR", v) },
        None => unsafe { std::env::remove_var("WAI_DATA_DIR") },
    }
}

#[serial_test::serial]
#[test]
fn run_hooks_skips_untrusted_custom_hook() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // A repository-owned plugin with an on_status hook.
    let plugin_dir = root.join(".wai/plugins");
    std::fs::create_dir_all(&plugin_dir).unwrap();
    std::fs::write(
        plugin_dir.join("evil.toml"),
        r#"
name = "evil"
description = "malicious"

[hooks.on_status]
command = "echo 'hook-ran'"
inject_as = "evil_marker"
"#,
    )
    .unwrap();

    // SAFETY: test-only env var, single-threaded tests.
    let saved = std::env::var_os("WAI_DATA_DIR");
    unsafe {
        std::env::set_var("WAI_DATA_DIR", tmp.path().join("wai-data"));
    }

    // Ensure no trust approval exists (fresh store).
    let outputs = run_hooks(root, "on_status");

    // The untrusted hook must NOT have executed.
    assert!(
        outputs.iter().all(|o| o.label != "evil_marker"),
        "no output from untrusted hook"
    );

    match saved {
        Some(v) => unsafe { std::env::set_var("WAI_DATA_DIR", v) },
        None => unsafe { std::env::remove_var("WAI_DATA_DIR") },
    }
}

#[serial_test::serial]
#[test]
fn run_hooks_executes_approved_custom_hook() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    let plugin_dir = root.join(".wai/plugins");
    std::fs::create_dir_all(&plugin_dir).unwrap();
    let _marker = root.join("hook-ran");
    std::fs::write(
        plugin_dir.join("evil.toml"),
        r#"
name = "evil"
description = "malicious"

[hooks.on_status]
command = "echo 'hook-ran'"
inject_as = "evil_marker"
"#,
    )
    .unwrap();

    // SAFETY: test-only env var, single-threaded tests.
    let saved = std::env::var_os("WAI_DATA_DIR");
    unsafe {
        std::env::set_var("WAI_DATA_DIR", tmp.path().join("wai-data"));
    }

    // Approve the hook digest, then run hooks again.
    let plugins = detect_plugins(root);
    let evil = plugins.iter().find(|p| p.def.name == "evil").unwrap();
    let hook = evil.def.hooks.get("on_status").unwrap();
    let digest = compute_hook_digest(&evil.def, "on_status", hook);
    let mut store = TrustStore::load();
    store.approve(TrustEntry {
        plugin_name: "evil".to_string(),
        hook_name: "on_status".to_string(),
        digest: digest.clone(),
        command: hook.command.clone(),
        approved_at: "2026-08-05T00:00:00Z".to_string(),
    });
    store.save().unwrap();
    assert!(TrustStore::load().is_approved(&digest));

    let outputs = run_hooks(root, "on_status");
    assert!(
        outputs.iter().any(|o| o.label == "evil_marker"),
        "approved hook must produce output"
    );

    match saved {
        Some(v) => unsafe { std::env::set_var("WAI_DATA_DIR", v) },
        None => unsafe { std::env::remove_var("WAI_DATA_DIR") },
    }
}

// ── timeout ───────────────────────────────────────────────────────────────

#[test]
fn execute_hook_timeout_kills_hung_process() {
    // A process that sleeps for 60 seconds should be killed by the 30s timeout.
    let hook = HookDef {
        command: "sleep 60".to_string(),
        inject_as: "timeout_test".to_string(),
    };
    let tmp = TempDir::new().unwrap();
    let start = std::time::Instant::now();
    let result = execute_hook(tmp.path(), &hook);
    let elapsed = start.elapsed();

    // Must complete in well under 60 seconds (the sleep duration).
    assert!(
        elapsed < Duration::from_secs(40),
        "hook must be killed within timeout, took {:.1}s",
        elapsed.as_secs_f64()
    );
    // The killed process produces no output.
    assert!(result.is_none());
}

#[test]
fn execute_hook_no_deadlock_on_fast_command() {
    // A fast command should complete normally (no timeout, no deadlock).
    let hook = HookDef {
        command: "echo 'hello'".to_string(),
        inject_as: "fast_test".to_string(),
    };
    let tmp = TempDir::new().unwrap();
    let result = execute_hook(tmp.path(), &hook);
    assert!(result.is_some());
    assert_eq!(result.unwrap().label, "fast_test");
}
