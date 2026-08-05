/// Test that genesis::envelope can be used to wrap wai's --json output.
use genesis::envelope::{Envelope, EnvelopeKind, HintEntry, Warning};
use serde::Serialize;
use tempfile::TempDir;

#[derive(Serialize)]
struct TestPayload {
    value: String,
}

#[test]
fn test_envelope_wraps_payload() {
    let payload = TestPayload {
        value: "hello".into(),
    };

    let env = Envelope::success(
        env!("CARGO_PKG_VERSION"),
        EnvelopeKind::Ok,
        payload,
        vec![],
        vec![HintEntry {
            command: "wai status".into(),
            description: "check status".into(),
        }],
    );

    let json = serde_json::to_value(&env).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["envelope_version"], "0.1");
    assert_eq!(json["envelope_kind"], "ok");
    assert_eq!(json["data"]["value"], "hello");
    assert!(json.get("hints").is_some());
}

#[test]
fn test_envelope_round_trips() {
    let payload = TestPayload {
        value: "world".into(),
    };

    let env = Envelope::success(
        env!("CARGO_PKG_VERSION"),
        EnvelopeKind::Ok,
        payload,
        vec![],
        vec![],
    );
    let json = serde_json::to_value(&env).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["envelope_kind"], "ok");
    assert_eq!(json["data"]["value"], "world");
}

#[test]
fn test_envelope_status_fields() {
    let payload = serde_json::json!({
        "phase": "research",
        "projects": []
    });

    let env = Envelope::success(
        env!("CARGO_PKG_VERSION"),
        EnvelopeKind::Ok,
        payload,
        vec![Warning {
            rule_name: "test-rule".into(),
            entity_id: None,
            message: "test warning".into(),
            suggested_remediation: Some("run fix".into()),
        }],
        vec![],
    );

    let json = serde_json::to_value(&env).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["envelope_kind"], "ok");
    assert_eq!(json["data"]["phase"], "research");
    assert_eq!(json["warnings"][0]["message"], "test warning");
    assert!(json.get("hints").is_some());
}

#[test]
fn test_envelope_error() {
    use genesis::envelope::{ErrorResult, RemediationEntry};

    let err = ErrorResult::new(
        "E001",
        "something went wrong",
        None,
        None,
        None,
        vec![],
        vec![RemediationEntry {
            command: "just fix".into(),
            description: "run the fix".into(),
        }],
    )
    .unwrap();

    let env = Envelope::error(env!("CARGO_PKG_VERSION"), err, vec![]);
    let json = serde_json::to_value(&env).unwrap();
    assert_eq!(json["ok"], false);
    assert_eq!(json["envelope_kind"], "error");
    assert_eq!(json["data"]["code"], "E001");
    assert_eq!(json["data"]["message"], "something went wrong");
}

/// Integration: wai status --json and wai prime --json produce the correct envelope shape.
#[test]
fn test_wai_status_json_has_envelope_shape() {
    use assert_cmd::Command;

    // Need a workspace to run status
    let dir = TempDir::new().unwrap();
    let mut init_cmd = Command::cargo_bin("wai").unwrap();
    init_cmd.current_dir(dir.path());
    init_cmd.env("NO_COLOR", "1");
    init_cmd.args(["init", "--name", "shape-test"]);
    init_cmd.assert().success();

    // Check wai status --json
    let mut cmd = Command::cargo_bin("wai").unwrap();
    cmd.current_dir(dir.path());
    cmd.env("NO_COLOR", "1");
    cmd.arg("status").arg("--json");

    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("status --json should be valid JSON");

    // Verify envelope shape
    assert!(
        parsed.get("ok").is_some(),
        "status --json should have 'ok' field"
    );
    assert!(
        parsed.get("envelope_version").is_some(),
        "status --json should have 'envelope_version' field"
    );
    assert!(
        parsed.get("cli_version").is_some(),
        "status --json should have 'cli_version' field"
    );
    assert!(
        parsed.get("envelope_kind").is_some(),
        "status --json should have 'envelope_kind' field"
    );
    assert!(
        parsed.get("data").is_some(),
        "status --json should have 'data' field"
    );
    assert!(
        parsed.get("warnings").is_some(),
        "status --json should have 'warnings' field"
    );
    assert!(
        parsed.get("meta").is_some(),
        "status --json should have 'meta' field"
    );
}

#[test]
fn test_wai_prime_json_has_envelope_shape() {
    use assert_cmd::Command;

    let dir = TempDir::new().unwrap();
    let mut init_cmd = Command::cargo_bin("wai").unwrap();
    init_cmd.current_dir(dir.path());
    init_cmd.env("NO_COLOR", "1");
    init_cmd.args(["init", "--name", "shape-test"]);
    init_cmd.assert().success();

    let mut cmd = Command::cargo_bin("wai").unwrap();
    cmd.current_dir(dir.path());
    cmd.env("NO_COLOR", "1");
    cmd.arg("prime").arg("--json");

    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("prime --json should be valid JSON");

    // Verify envelope shape
    assert!(
        parsed.get("ok").is_some(),
        "prime --json should have 'ok' field"
    );
    assert!(
        parsed.get("envelope_version").is_some(),
        "prime --json should have 'envelope_version' field"
    );
    assert!(
        parsed.get("cli_version").is_some(),
        "prime --json should have 'cli_version' field"
    );
    assert!(
        parsed.get("envelope_kind").is_some(),
        "prime --json should have 'envelope_kind' field"
    );
    assert!(
        parsed.get("data").is_some(),
        "prime --json should have 'data' field"
    );
    assert!(
        parsed.get("warnings").is_some(),
        "prime --json should have 'warnings' field"
    );
    assert!(
        parsed.get("meta").is_some(),
        "prime --json should have 'meta' field"
    );
}

#[test]
fn test_wai_version_json_envelope_shape() {
    use assert_cmd::Command;

    let mut cmd = Command::cargo_bin("wai").unwrap();
    cmd.env("NO_COLOR", "1");
    cmd.arg("--version").arg("--json");

    let output = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("--version --json should be valid JSON");

    assert_eq!(parsed["ok"], true, "version envelope should have ok=true");
    assert_eq!(
        parsed["envelope_kind"], "version",
        "version envelope should have kind=version"
    );
    assert!(parsed.get("data").is_some(), "version should have data");
    assert_eq!(parsed["data"]["name"], "wai");
    assert!(parsed.get("meta").is_some());
}

#[test]
fn test_output_has_envelope_shape() {
    let payload = serde_json::json!({"msg": "hello"});
    let env = Envelope::success(
        env!("CARGO_PKG_VERSION"),
        EnvelopeKind::Ok,
        payload,
        vec![],
        vec![],
    );
    let json = serde_json::to_value(&env).unwrap();

    // Verify the top-level envelope shape
    let keys: Vec<&str> = json
        .as_object()
        .unwrap()
        .keys()
        .map(|k| k.as_str())
        .collect();
    assert!(keys.contains(&"ok"));
    assert!(keys.contains(&"envelope_version"));
    assert!(keys.contains(&"cli_version"));
    assert!(keys.contains(&"envelope_kind"));
    assert!(keys.contains(&"data"));
    assert!(keys.contains(&"warnings"));
    assert!(keys.contains(&"meta"));
}
