//! Offline Python conformance and edge tests.
use semantic_operators::{
    answer::{complement, make_answer},
    wire, *,
};
use serde_json::{Value, json};
fn fixtures() -> Value {
    serde_json::from_str(include_str!("fixtures/python-parity.json")).unwrap()
}
fn question(v: &Value) -> Question {
    let instructions = v["instructions"].as_str().unwrap();
    let threshold = v["min_confidence"].as_f64().unwrap_or(0.0);
    match v["type"].as_str().unwrap() {
        "boolean" => Boolean::new(instructions)
            .unwrap()
            .with_min_confidence(threshold)
            .unwrap()
            .into(),
        "choice" => Choice::new(
            instructions,
            v["options"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| {
                    (
                        v[0].as_str().unwrap().to_owned(),
                        v[1].as_str().map(str::to_owned),
                    )
                })
                .collect(),
        )
        .unwrap()
        .with_min_confidence(threshold)
        .unwrap()
        .into(),
        "score" => Score::new(
            instructions,
            v["levels"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_owned())
                .collect(),
        )
        .unwrap()
        .with_min_confidence(threshold)
        .unwrap()
        .into(),
        "box" => BoxQuestion::new(instructions)
            .unwrap()
            .with_min_confidence(threshold)
            .unwrap()
            .into(),
        _ => unreachable!(),
    }
}
fn value(v: &Value) -> AnswerValue {
    match v {
        Value::Bool(v) => AnswerValue::Boolean(*v),
        Value::String(v) => AnswerValue::Choice(v.clone()),
        Value::Number(v) => AnswerValue::Score(v.as_f64().unwrap()),
        Value::Object(_) => AnswerValue::Box(
            Region::new(
                v["top"].as_f64().unwrap(),
                v["bottom"].as_f64().unwrap(),
                v["left"].as_f64().unwrap(),
                v["right"].as_f64().unwrap(),
            )
            .unwrap(),
        ),
        _ => unreachable!(),
    }
}
fn to_json(value: Option<&AnswerValue>) -> Value {
    match value {
        None => Value::Null,
        Some(AnswerValue::Boolean(v)) => json!(v),
        Some(AnswerValue::Choice(v)) => json!(v),
        Some(AnswerValue::Score(v)) => json!(v),
        Some(AnswerValue::Box(v)) => {
            json!({"top":v.top(),"bottom":v.bottom(),"left":v.left(),"right":v.right()})
        }
    }
}
#[test]
fn all_38_frozen_python_cases() {
    let fixture = fixtures();
    let mut count = 0;
    for case in fixture["answer_cases"].as_array().unwrap() {
        let answer = make_answer(
            &question(&case["question"]),
            value(&case["value"]),
            serde_json::from_value(case["probabilities"].clone()).unwrap(),
            case["confidence"].as_f64(),
        );
        if case.get("expected_error").is_some() {
            assert_eq!(
                answer.unwrap_err().kind(),
                ErrorKind::InvalidResponse,
                "{}",
                case["id"]
            );
        } else {
            let a = answer.unwrap();
            assert_eq!(
                json!({"value":to_json(a.value()),"decided":a.decided(),"confidence":a.confidence(),"probabilities":a.probabilities()}),
                case["expected"],
                "{}",
                case["id"]
            );
        }
        count += 1;
    }
    for case in fixture["complement_cases"].as_array().unwrap() {
        assert_eq!(
            complement(case["input"].as_f64().unwrap()).unwrap(),
            case["expected"].as_f64().unwrap()
        );
        count += 1;
    }
    for case in fixture["wire_decode_cases"].as_array().unwrap() {
        let decoded = wire::decode_request(case["request"].clone());
        if let Some(expected) = case.get("expected_error") {
            let error = decoded.unwrap_err();
            assert_eq!(
                error.path().unwrap_or(""),
                expected["path"].as_str().unwrap()
            );
            assert_eq!(error.to_string(), expected["message"].as_str().unwrap());
        } else {
            let r = decoded.unwrap();
            assert_eq!(
                json!({"state":r.state.as_value(),"question_names":r.questions.keys().collect::<Vec<_>>(),"request_id":r.request_id}),
                case["expected"]
            );
        }
        count += 1;
    }
    assert_eq!(count, 38);
}
#[test]
fn invalid_questions_and_serde_cannot_bypass_validation() {
    assert!(Boolean::new(" \n ").is_err());
    assert!(Score::new("q", vec!["a".into(), "a".into()]).is_err());
    assert!(Choice::new("q", IndexMap::from([("".into(), None), ("b".into(), None)])).is_err());
    for v in [f64::NAN, f64::INFINITY, -0.01, 1.01] {
        assert!(Boolean::new("q").unwrap().with_min_confidence(v).is_err());
    }
    assert!(Score::new("q", vec![" ".into(), "b".into()]).is_ok());
    assert!(serde_json::from_value::<Boolean>(json!({"instructions":""})).is_err());
    assert!(
        serde_json::from_value::<Question>(
            json!({"type":"score","instructions":"q","levels":["a"]})
        )
        .is_err()
    );
    assert!(
        serde_json::from_value::<Region>(json!({"top":0.9,"bottom":0.1,"left":0,"right":1}))
            .is_err()
    );
    assert!(serde_json::from_value::<State>(json!(false)).is_err());
    let q: Question = Boolean::new(" q ").unwrap().into();
    assert_eq!(
        serde_json::from_value::<Question>(serde_json::to_value(&q).unwrap()).unwrap(),
        q
    );
}
#[test]
fn thresholds_revalidation_and_type_checks_retain_evidence() {
    let q = Boolean::new("q").unwrap().with_min_confidence(0.9).unwrap();
    let a = make_answer(
        &q.clone().into(),
        AnswerValue::Boolean(true),
        IndexMap::from([("true".into(), 0.8), ("false".into(), 0.2)]),
        None,
    )
    .unwrap()
    .with_raw(json!({"private":"secret"}))
    .with_call(std::sync::Arc::new(Call {
        provider: "script".into(),
        ..Default::default()
    }));
    assert!(!a.decided());
    assert!(a.typed::<String>().is_err());
    assert!(a.typed::<bool>().unwrap().value().is_none());
    assert!(!a.clone().with_min_confidence(0.1).unwrap().decided());
    let low: Question = q.with_min_confidence(0.8).unwrap().into();
    let restored = a.revalidate(&low).unwrap();
    assert_eq!(restored.typed::<bool>().unwrap().value(), None);
    assert_eq!(restored.raw(), a.raw());
    assert_eq!(restored.call(), a.call());
    assert!(!format!("{a:?}").contains("secret"));
}
#[test]
fn numeric_boundaries_reject_nonfinite_and_do_not_merge_near_ties() {
    let q: Question = Boolean::new("q").unwrap().into();
    for p in [f64::NAN, f64::INFINITY, -0.01, 1.01] {
        assert!(
            make_answer(
                &q,
                AnswerValue::Boolean(true),
                IndexMap::from([("true".into(), p), ("false".into(), 0.0)]),
                None
            )
            .is_err()
        );
        assert!(complement(p).is_err());
    }
    let a = make_answer(
        &q,
        AnswerValue::Boolean(true),
        IndexMap::from([
            ("true".into(), 0.50000000001),
            ("false".into(), 0.49999999999),
        ]),
        None,
    )
    .unwrap();
    assert!(a.decided());
    assert!(
        make_answer(
            &q,
            AnswerValue::Boolean(true),
            IndexMap::from([("true".into(), 0.6), ("false".into(), 0.4100000009)]),
            None
        )
        .is_ok()
    );
    assert!(
        make_answer(
            &q,
            AnswerValue::Boolean(true),
            IndexMap::from([("true".into(), 0.6), ("false".into(), 0.4100000011)]),
            None
        )
        .is_err()
    );
    // Python round() binary/decimal boundaries: multiply-and-round gives a different result.
    assert_eq!(complement(0.99999999995).unwrap(), 0.0000000001);
    assert_eq!(complement(0.12345678905).unwrap(), 0.8765432109);
}
#[test]
fn region_geometry_uses_python_pixel_rounding() {
    let r = Region::new(0.25, 0.75, 0.25, 0.75).unwrap();
    assert_eq!(r.pixels(2, 2).unwrap(), (0, 0, 2, 2));
    assert_eq!(r.center(), (0.5, 0.5));
    assert_eq!(
        r.padded(1.0).unwrap(),
        Region::new(0.0, 1.0, 0.0, 1.0).unwrap()
    );
    assert!(r.padded(f64::NAN).is_err());
    assert!(r.pixels(0, 2).is_err());
    assert!(Region::new(0.2, 0.2, 0.4, 0.4).is_ok());
}
#[test]
fn additional_python_numeric_and_wire_edge_fixtures() {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/python-core-edges.json")).unwrap();
    for case in fixture["complement_cases"].as_array().unwrap() {
        assert_eq!(
            complement(case["input"].as_f64().unwrap()).unwrap(),
            case["expected"].as_f64().unwrap(),
            "{case}"
        );
    }
    for case in fixture["wire_errors"].as_array().unwrap() {
        let error = wire::decode_request(case["request"].clone()).unwrap_err();
        assert_eq!(error.path().unwrap_or(""), case["path"].as_str().unwrap());
        assert_eq!(error.to_string(), case["message"].as_str().unwrap());
    }
    let request = wire::decode_request(
        json!({"state":"synthetic","questions":[{"name":"q","type":"boolean","instructions":"q"}]}),
    )
    .unwrap();
    for case in fixture["wire_encodings"].as_array().unwrap() {
        let answer = make_answer(
            &request.questions["q"],
            AnswerValue::Boolean(case["value"].as_bool().unwrap()),
            serde_json::from_value(case["probabilities"].clone()).unwrap(),
            None,
        )
        .unwrap();
        assert_eq!(
            wire::encode_response(&request, &IndexMap::from([("q".into(), answer)])).unwrap(),
            case["expected"],
            "{case}"
        );
    }
}
#[test]
fn python_distribution_and_expected_score_boundaries() {
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/python-core-edges.json")).unwrap();
    for case in fixture["normalization_cases"].as_array().unwrap() {
        let result = make_answer(
            &question(&case["question"]),
            value(&case["value"]),
            serde_json::from_value(case["probabilities"].clone()).unwrap(),
            None,
        );
        if case.get("expected_error").is_some() {
            assert!(result.is_err(), "{case}");
        } else {
            let a = result.unwrap();
            assert_eq!(
                json!({"value":to_json(a.value()),"decided":a.decided(),"confidence":a.confidence(),"probabilities":a.probabilities()}),
                case["expected"],
                "{case}"
            );
        }
    }
}
