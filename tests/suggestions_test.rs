use std::fs;
use tempfile::TempDir;
use wai::suggestions::{Suggestion, SuggestionEngine};

// ─── Typo Detection ───────────────────────────────────────────────────────────

const WAI_COMMANDS: &[&str] = &[
    "new", "add", "show", "move", "init", "status", "phase", "sync", "config", "handoff", "search",
    "timeline", "plugin", "doctor", "way", "import", "resource", "tutorial",
];

#[test]
fn typo_status_statu() {
    let engine = SuggestionEngine::new();
    let suggestion = engine.suggest_typo("statu", WAI_COMMANDS);
    assert!(suggestion.is_some());
    match suggestion.unwrap() {
        Suggestion::DidYouMean { suggestion, .. } => assert_eq!(suggestion, "status"),
        other => panic!("expected DidYouMean, got {:?}", other),
    }
}

#[test]
fn typo_doctor_doctr() {
    let engine = SuggestionEngine::new();
    let suggestion = engine.suggest_typo("doctr", WAI_COMMANDS);
    assert!(suggestion.is_some());
    match suggestion.unwrap() {
        Suggestion::DidYouMean { suggestion, .. } => assert_eq!(suggestion, "doctor"),
        other => panic!("expected DidYouMean, got {:?}", other),
    }
}

#[test]
fn typo_init_iint() {
    let engine = SuggestionEngine::new();
    let suggestion = engine.suggest_typo("iint", WAI_COMMANDS);
    assert!(suggestion.is_some());
    match suggestion.unwrap() {
        Suggestion::DidYouMean { suggestion, .. } => assert_eq!(suggestion, "init"),
        other => panic!("expected DidYouMean, got {:?}", other),
    }
}

#[test]
fn typo_search_searh() {
    let engine = SuggestionEngine::new();
    let suggestion = engine.suggest_typo("searh", WAI_COMMANDS);
    assert!(suggestion.is_some());
    match suggestion.unwrap() {
        Suggestion::DidYouMean { suggestion, .. } => assert_eq!(suggestion, "search"),
        other => panic!("expected DidYouMean, got {:?}", other),
    }
}

#[test]
fn typo_no_match_for_gibberish() {
    let engine = SuggestionEngine::new();
    assert!(engine.suggest_typo("xqz", WAI_COMMANDS).is_none());
    assert!(engine.suggest_typo("aaaaa", WAI_COMMANDS).is_none());
}

#[test]
fn typo_ranking_picks_closest_match() {
    let engine = SuggestionEngine::new();
    // "statuz" is closest to "status" not "sync" or "show"
    let suggestion = engine.suggest_typo("statuz", WAI_COMMANDS);
    assert!(suggestion.is_some());
    match suggestion.unwrap() {
        Suggestion::DidYouMean { suggestion, .. } => assert_eq!(suggestion, "status"),
        other => panic!("expected DidYouMean, got {:?}", other),
    }
}

#[test]
fn typo_message_contains_original_and_suggestion() {
    let engine = SuggestionEngine::new();
    let suggestion = engine.suggest_typo("statu", WAI_COMMANDS).unwrap();
    let msg = suggestion.message();
    assert!(
        msg.contains("statu"),
        "message should contain original: {}",
        msg
    );
    assert!(
        msg.contains("status"),
        "message should contain suggestion: {}",
        msg
    );
    assert!(
        msg.contains("Did you mean"),
        "message should contain hint phrase: {}",
        msg
    );
}

// ─── Wrong Order Detection ────────────────────────────────────────────────────

const WAI_PATTERNS: &[(&str, &str)] = &[
    ("new", "project"),
    ("new", "area"),
    ("new", "resource"),
    ("add", "research"),
    ("add", "plan"),
    ("add", "design"),
    ("phase", "next"),
    ("phase", "set"),
    ("handoff", "create"),
    ("plugin", "list"),
];

#[test]
fn wrong_order_project_new() {
    let engine = SuggestionEngine::new();
    let suggestion = engine.suggest_order("project", "new", WAI_PATTERNS);
    assert!(suggestion.is_some());
    match suggestion.unwrap() {
        Suggestion::WrongOrder { original, correct } => {
            assert_eq!(original, "project new");
            assert_eq!(correct, "new project");
        }
        other => panic!("expected WrongOrder, got {:?}", other),
    }
}

#[test]
fn wrong_order_research_add() {
    let engine = SuggestionEngine::new();
    let suggestion = engine.suggest_order("research", "add", WAI_PATTERNS);
    assert!(suggestion.is_some());
    match suggestion.unwrap() {
        Suggestion::WrongOrder { correct, .. } => assert_eq!(correct, "add research"),
        other => panic!("expected WrongOrder, got {:?}", other),
    }
}

#[test]
fn wrong_order_plan_add() {
    let engine = SuggestionEngine::new();
    let suggestion = engine.suggest_order("plan", "add", WAI_PATTERNS);
    assert!(suggestion.is_some());
    match suggestion.unwrap() {
        Suggestion::WrongOrder { correct, .. } => assert_eq!(correct, "add plan"),
        other => panic!("expected WrongOrder, got {:?}", other),
    }
}

#[test]
fn wrong_order_no_match_for_valid_pattern() {
    let engine = SuggestionEngine::new();
    // "new project" is already correct, no suggestion
    assert!(
        engine
            .suggest_order("new", "project", WAI_PATTERNS)
            .is_none()
    );
    assert!(
        engine
            .suggest_order("add", "research", WAI_PATTERNS)
            .is_none()
    );
}

#[test]
fn wrong_order_no_match_for_unknown_pair() {
    let engine = SuggestionEngine::new();
    assert!(engine.suggest_order("foo", "bar", WAI_PATTERNS).is_none());
    assert!(engine.suggest_order("new", "foo", WAI_PATTERNS).is_none());
}

#[test]
fn wrong_order_message_format() {
    let engine = SuggestionEngine::new();
    let suggestion = engine
        .suggest_order("project", "new", WAI_PATTERNS)
        .unwrap();
    let msg = suggestion.message();
    assert!(
        msg.contains("project new"),
        "message should contain original: {}",
        msg
    );
    assert!(
        msg.contains("new project"),
        "message should contain correction: {}",
        msg
    );
    assert!(
        msg.contains("Did you mean"),
        "message should contain hint phrase: {}",
        msg
    );
}

// ─── Context Inference (subdirectory detection) ───────────────────────────────

#[test]
fn context_no_hint_when_wai_in_current_dir() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir(tmp.path().join(".wai")).unwrap();

    let engine = SuggestionEngine::new();
    // .wai is in current dir → no hint needed
    let hint = engine.suggest_context(tmp.path(), ".wai");
    assert!(
        hint.is_none(),
        "no hint expected when .wai is in current dir"
    );
}

#[test]
fn context_hint_when_wai_in_parent() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir(tmp.path().join(".wai")).unwrap();
    let subdir = tmp.path().join("src");
    fs::create_dir(&subdir).unwrap();

    let engine = SuggestionEngine::new();
    // .wai is in parent (tmp), user is in subdir
    let hint = engine.suggest_context(&subdir, ".wai");
    assert!(
        hint.is_some(),
        "expected hint when .wai is in parent directory"
    );
    match hint.unwrap() {
        Suggestion::ContextHint { message, path } => {
            assert!(
                message.contains("parent"),
                "message should mention parent: {}",
                message
            );
            assert!(
                path.as_deref()
                    .unwrap_or("")
                    .contains(tmp.path().to_str().unwrap()),
                "path should point to workspace root"
            );
        }
        other => panic!("expected ContextHint, got {:?}", other),
    }
}

#[test]
fn context_hint_when_wai_in_grandparent() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir(tmp.path().join(".wai")).unwrap();
    let subdir = tmp.path().join("src").join("lib");
    fs::create_dir_all(&subdir).unwrap();

    let engine = SuggestionEngine::new();
    let hint = engine.suggest_context(&subdir, ".wai");
    assert!(
        hint.is_some(),
        "expected hint when .wai is in grandparent directory"
    );
}

#[test]
fn context_no_hint_when_wai_not_found() {
    let tmp = TempDir::new().unwrap();
    // No .wai anywhere
    let engine = SuggestionEngine::new();
    let hint = engine.suggest_context(tmp.path(), ".wai");
    assert!(hint.is_none(), "no hint expected when .wai is not found");
}
