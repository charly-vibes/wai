/// Test that genesis::envelope can be used to wrap wai's --json output.
use genesis::envelope::{Envelope, EnvelopeKind, HintEntry, Warning};
use serde::Serialize;

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

    let env = Envelope::success(EnvelopeKind::Ok, payload, vec![], vec![]);
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

    let env = Envelope::error(err, vec![]);
    let json = serde_json::to_value(&env).unwrap();
    assert_eq!(json["ok"], false);
    assert_eq!(json["envelope_kind"], "error");
    assert_eq!(json["data"]["code"], "E001");
    assert_eq!(json["data"]["message"], "something went wrong");
}

#[test]
fn test_output_has_envelope_shape() {
    let payload = serde_json::json!({"msg": "hello"});
    let env = Envelope::success(EnvelopeKind::Ok, payload, vec![], vec![]);
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
