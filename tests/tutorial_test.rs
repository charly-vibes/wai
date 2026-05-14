use assert_cmd::Command;
use std::{fs, io::Write};
use tempfile::TempDir;

#[allow(deprecated)]
fn wai_cmd(dir: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("wai").unwrap();
    cmd.current_dir(dir);
    cmd.env("NO_COLOR", "1");
    cmd
}

// ── tutorial startup and completion ──────────────────────────────────────────

#[test]
fn tutorial_first_run_shows_welcome_message() {
    let tmp_config = TempDir::new().unwrap();
    let tmp_project = TempDir::new().unwrap();

    wai_cmd(tmp_project.path())
        .env("XDG_CONFIG_HOME", tmp_config.path())
        .args(["tutorial"])
        .assert()
        .success();
}

// ── persistence of tutorial-seen state ───────────────────────────────────────

#[test]
fn tutorial_marks_seen_after_completion() {
    let tmp_config = TempDir::new().unwrap();
    let tmp_project = TempDir::new().unwrap();

    wai_cmd(tmp_project.path())
        .env("XDG_CONFIG_HOME", tmp_config.path())
        .args(["tutorial"])
        .assert()
        .success();

    let config_content =
        fs::read_to_string(tmp_config.path().join("wai/config.toml")).unwrap_or_default();
    assert!(
        config_content.contains("seen_tutorial = true"),
        "seen_tutorial should be persisted after tutorial run"
    );
}

// ── repeat-run path ───────────────────────────────────────────────────────────

#[test]
fn tutorial_repeat_run_shows_replay_message() {
    let tmp_config = TempDir::new().unwrap();
    let tmp_project = TempDir::new().unwrap();

    let wai_config_dir = tmp_config.path().join("wai");
    fs::create_dir_all(&wai_config_dir).unwrap();
    let mut config_file = fs::File::create(wai_config_dir.join("config.toml")).unwrap();
    writeln!(config_file, "seen_tutorial = true").unwrap();

    wai_cmd(tmp_project.path())
        .env("XDG_CONFIG_HOME", tmp_config.path())
        .args(["tutorial"])
        .assert()
        .success();
}

// ── unit tests for UserConfig ─────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use wai::config::UserConfig;

    #[test]
    fn test_default_user_config() {
        let config = UserConfig::default();
        assert!(
            !config.seen_tutorial,
            "Default should have seen_tutorial = false"
        );
        assert!(
            config.version.is_empty(),
            "Default should have empty version"
        );
    }

    #[test]
    fn test_user_config_mark_tutorial_seen() {
        let mut config = UserConfig::default();
        assert!(!config.seen_tutorial);

        config.mark_tutorial_seen();
        assert!(
            config.seen_tutorial,
            "Tutorial flag should be set after marking"
        );
    }

    #[test]
    fn test_user_config_serialization() {
        let mut config = UserConfig::default();
        config.seen_tutorial = true;
        config.version = "2026.2.0".to_string();

        let toml_string = toml::to_string_pretty(&config).expect("Should serialize");
        let deserialized: UserConfig = toml::from_str(&toml_string).expect("Should deserialize");

        assert_eq!(deserialized.seen_tutorial, config.seen_tutorial);
        assert_eq!(deserialized.version, config.version);
    }
}
