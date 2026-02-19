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
