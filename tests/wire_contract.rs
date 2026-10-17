//! Offline Python conformance and edge tests.
use semantic_operators::{wire::*, *};
use serde_json::{Value, json};
fn base() -> Value {
    json!({"schema_version":"1","state":{"arbitrary":true},"questions":[{"name":"q","type":"boolean","instructions":"q"}]})
}
#[test]
fn rejects_unknown_fields_at_every_owned_level_in_sorted_order() {
    let mut r = base();
    r["z"] = json!(true);
    r["a"] = json!(true);
    assert_eq!(decode_request(r).unwrap_err().path(), Some("a"));
    let mut r = base();
    r["questions"][0]["unexpected"] = json!(true);
    assert_eq!(
        decode_request(r).unwrap_err().path(),
        Some("questions[0].unexpected")
    );
    let mut r = base();
    r["questions"][0] = json!({"name":"q","type":"choice","instructions":"q","options":[{"name":"a","secret":1},"b"]});
    assert_eq!(
        decode_request(r).unwrap_err().path(),
        Some("questions[0].options[0].secret")
    );
}
#[test]
fn preserves_order_descriptions_and_false_zero() {
    let r=decode_request(json!({"state":[1,false],"questions":[{"name":"z","type":"choice","instructions":"q","options":["second",{"name":"first","description":"description"}]},{"name":"a","type":"boolean","instructions":"q","true":null,"false":"no"}]})).unwrap();
    assert_eq!(
        r.questions.keys().map(String::as_str).collect::<Vec<_>>(),
        vec!["z", "a"]
    );
    let Question::Choice(q) = &r.questions["z"] else {
        panic!()
    };
    assert_eq!(
        q.options().keys().map(String::as_str).collect::<Vec<_>>(),
        vec!["second", "first"]
    );
    assert_eq!(q.options()["second"], None);
    assert_eq!(q.options()["first"], Some("description".into()));
    let Question::Boolean(q) = &r.questions["a"] else {
        panic!()
    };
    assert_eq!(q.true_description(), None);
    assert_eq!(q.false_description(), Some("no"));
}
#[test]
fn encoding_rounds_to_four_significant_digits_and_projects_first_call() {
    let r=decode_request(json!({"request_id":"id","state":"s","questions":[{"name":"q","type":"boolean","instructions":"q"},{"name":"other","type":"boolean","instructions":"q"}]})).unwrap();
    let make = |name: &str| {
        make_answer(
            &r.questions["q"],
            AnswerValue::Boolean(false),
            IndexMap::from([("true".into(), 0.012345), ("false".into(), 0.987655)]),
            None,
        )
        .unwrap()
        .with_call(std::sync::Arc::new(Call {
            provider: name.into(),
            model: None,
            input_tokens: None,
            output_tokens: None,
        }))
        .with_raw(json!({"secret":"excluded"}))
    };
    let answers = IndexMap::from([
        ("q".into(), make("first")),
        ("other".into(), make("second")),
    ]);
    let out = encode_response(&r, &answers).unwrap();
    assert_eq!(out["call"]["provider"], "first");
    assert_eq!(out["answers"]["q"]["value"], false);
    assert_eq!(out["answers"]["q"]["confidence"], 0.9877);
    assert_eq!(out["answers"]["q"]["probabilities"]["true"], 0.01235);
    assert_eq!(out["request_id"], "id");
    assert!(!out.to_string().contains("secret"));
}
#[test]
fn validation_has_paths_and_three_wire_error_codes() {
    for (field, value, path) in [
        ("schema_version", json!(1), "schema_version"),
        ("request_id", json!(7), "request_id"),
        ("state", Value::Null, "state"),
    ] {
        let mut r = base();
        r[field] = value;
        assert_eq!(decode_request(r).unwrap_err().path(), Some(path));
    }
    let err = Error::new(ErrorKind::InvalidRequest, "unknown field").with_path("typo");
    assert_eq!(
        encode_error(&err, Some("x")),
        json!({"request_id":"x","error":{"code":"invalid_request","message":"typo: unknown field"}})
    );
    assert_eq!(
        encode_error(&Error::new(ErrorKind::Timeout, "deadline"), None)["error"]["code"],
        "timeout"
    );
    assert_eq!(
        encode_error(&Error::new(ErrorKind::InvalidResponse, "bad output"), None)["error"]["code"],
        "provider_error"
    );
}
#[test]
fn duplicate_labels_and_invalid_side_types_fail_without_silent_coercion() {
    let mut r = base();
    r["questions"][0]["true"] = json!(true);
    assert_eq!(
        decode_request(r).unwrap_err().path(),
        Some("questions[0].true")
    );
    let mut r = base();
    r["questions"][0] = json!({"name":"q","type":"choice","instructions":"q","options":["a","a"]});
    assert_eq!(
        decode_request(r).unwrap_err().path(),
        Some("questions[0].options[1]")
    );
    let mut r = base();
    r["questions"][0] = json!({"name":"q","type":"score","instructions":"q","levels":["a","a"]});
    assert_eq!(
        decode_request(r).unwrap_err().to_string(),
        "questions[0]: levels must be unique"
    );
}
