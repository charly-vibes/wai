/// Test that genesis::suggestions is accessible and works from wai.
///
/// This test validates that the genesis suggestions API can be used
/// as a replacement for wai's local `src/suggestions.rs`. After migration,
/// wai will re-export `genesis::suggestions` rather than defining its own.
#[test]
fn test_genesis_suggestions_typo_detection() {
    use genesis::suggestions::{CommandRegistry, Suggestion, SuggestionEngine};

    let engine = SuggestionEngine::new();
    let mut reg = CommandRegistry::new();
    reg.register("wai", vec!["status".into(), "init".into(), "new".into()]);

    let suggestion = engine.suggest_typo("staus", &reg);
    assert!(suggestion.is_some());

    if let Some(Suggestion::DidYouMean {
        original,
        suggestion,
    }) = suggestion
    {
        assert_eq!(original, "staus");
        assert_eq!(suggestion, "status");
    } else {
        panic!("Expected DidYouMean suggestion, got: {:?}", suggestion);
    }
}

#[test]
fn test_genesis_suggestions_wrong_order() {
    use genesis::suggestions::{Suggestion, SuggestionEngine};

    let engine = SuggestionEngine::new();
    let patterns = &[("new", "project"), ("add", "research")];

    let suggestion = engine.suggest_order("project", "new", patterns);
    assert!(suggestion.is_some());

    if let Some(Suggestion::WrongOrder { original, correct }) = suggestion {
        assert_eq!(original, "project new");
        assert_eq!(correct, "new project");
    } else {
        panic!("Expected WrongOrder suggestion, got: {:?}", suggestion);
    }
}

#[test]
fn test_genesis_suggestions_no_typo_for_dissimilar() {
    use genesis::suggestions::{CommandRegistry, SuggestionEngine};

    let engine = SuggestionEngine::new();
    let mut reg = CommandRegistry::new();
    reg.register("wai", vec!["status".into(), "init".into()]);

    let suggestion = engine.suggest_typo("xyz", &reg);
    assert!(suggestion.is_none());
}

#[test]
fn test_genesis_suggestions_message_formatting() {
    use genesis::suggestions::Suggestion;

    let typo = Suggestion::DidYouMean {
        original: "staus".to_string(),
        suggestion: "status".to_string(),
    };
    let msg = typo.message();
    assert!(msg.contains("Did you mean"));
    assert!(msg.contains("staus"));
    assert!(msg.contains("status"));
}

#[test]
fn test_genesis_suggestions_command_registry() {
    use genesis::suggestions::CommandRegistry;

    let mut reg = CommandRegistry::new();
    assert!(reg.all().is_empty());

    reg.register("wai", vec!["status".into(), "init".into()]);
    assert_eq!(reg.all().len(), 2);
    assert_eq!(reg.for_tool("wai").len(), 2);
    assert!(reg.for_tool("unknown").is_empty());

    // Re-registering same tool should replace
    reg.register("wai", vec!["new".into()]);
    assert_eq!(reg.all().len(), 1);
    assert_eq!(reg.all(), vec!["new"]);
}
