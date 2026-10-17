//! Independent expectations executed by the pinned Python 0.12 reference.
use semantic_operators::each::{BatchOptions, Item, ItemResult};
use semantic_operators::filter::{FilterOutcome, Filtered, filter_items};
use semantic_operators::rerank::{Candidate, Ranking, rerank};
use semantic_operators::{
    Answer, AnswerValue, Answers, Boolean, Call, Error, ErrorKind, IndexMap, Provider,
    ProviderFuture, Question, Region, Request, Score, State, complement, make_answer,
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/python-recipes.json")).unwrap()
}
#[derive(Default)]
struct Synthetic {
    seen: Mutex<Vec<Value>>,
}
impl Provider for Synthetic {
    fn ask<'a>(&'a self, request: &'a Request) -> ProviderFuture<'a> {
        Box::pin(async move {
            let state = request.state().as_value();
            self.seen.lock().unwrap().push(state.clone());
            let fixture = state
                .get("document")
                .or_else(|| state.get("item"))
                .unwrap_or(state);
            if let Some(error) = fixture.get("error").and_then(Value::as_str) {
                return Err(Error::new(
                    if error == "timeout" {
                        ErrorKind::Timeout
                    } else {
                        ErrorKind::Provider
                    },
                    "synthetic offline failure",
                ));
            }
            let mut question = request.questions()["q"].clone();
            if let Some(threshold) = fixture.get("provider_threshold").and_then(Value::as_f64) {
                question = match question {
                    Question::Boolean(q) => q.with_min_confidence(threshold)?.into(),
                    Question::Score(q) => q.with_min_confidence(threshold)?.into(),
                    _ => unreachable!(),
                };
            }
            let (value, probabilities) = match &question {
                Question::Boolean(_) => {
                    let p = fixture.get("p").and_then(Value::as_f64).unwrap_or(0.9);
                    (
                        AnswerValue::Boolean(p > 0.5),
                        IndexMap::from([("true".into(), p), ("false".into(), complement(p)?)]),
                    )
                }
                Question::Score(q) => {
                    let values = fixture
                        .get("p")
                        .and_then(Value::as_array)
                        .map(|values| {
                            values
                                .iter()
                                .map(|v| v.as_f64().unwrap())
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or(vec![0.0, 0.0, 1.0]);
                    let value = values.iter().enumerate().map(|(i, p)| i as f64 * p).sum();
                    (
                        AnswerValue::Score(value),
                        q.levels().iter().cloned().zip(values).collect(),
                    )
                }
                _ => unreachable!(),
            };
            let model = match fixture.get("model") {
                Some(value) => value.as_str().map(str::to_owned),
                None => Some("model-a".into()),
            };
            let answer =
                make_answer(&question, value, probabilities, None)?.with_call(Arc::new(Call {
                    provider: "synthetic".into(),
                    model,
                    input_tokens: None,
                    output_tokens: None,
                }));
            Ok(Answers::from([("q".into(), answer)]))
        })
    }
}
fn answer_json(answer: &Answer) -> Value {
    let value = match answer.value() {
        Some(AnswerValue::Boolean(v)) => json!(v),
        Some(AnswerValue::Score(v)) => json!(v),
        None => Value::Null,
        _ => unreachable!(),
    };
    json!({"value":value,"confidence":answer.confidence(),"probabilities":answer.probabilities(),"model":answer.call().and_then(|c|c.model.as_deref())})
}
fn error_json(error: &Error) -> Value {
    json!({"error":if error.kind()==ErrorKind::Timeout {"timeout"}else{"provider"}})
}
fn result_json(result: &ItemResult) -> Value {
    match result {
        ItemResult::Answer(a) => answer_json(a),
        ItemResult::Error(e) => error_json(e),
    }
}
fn filter_json(result: &Filtered) -> Value {
    json!({"models":result.models(),"verdicts":result.verdicts().iter().map(|v|json!({
        "id":v.id(),"outcome":match v.outcome() {FilterOutcome::Match=>"match",FilterOutcome::No=>"no",FilterOutcome::Unsure=>"unsure",FilterOutcome::Failed=>"failed"},
        "result":if let Some(answer)=v.answer() {answer_json(answer)}else{error_json(v.error().unwrap())}
    })).collect::<Vec<_>>()})
}
fn ranking_json(result: &Ranking) -> Value {
    let top=[(0,false),(1,false),(10,false),(10,true)].into_iter().map(|(k,partial)| {
        match result.top(k,partial) {
            Ok(items)=>json!({"k":k,"allow_partial":partial,"ids":items.iter().map(|r|r.id()).collect::<Vec<_>>()}),
            Err(_)=>json!({"k":k,"allow_partial":partial,"error":true}),
        }
    }).collect::<Vec<_>>();
    json!({"complete":result.complete(),"models":result.models(),
        "ranked":result.ranked().iter().map(|r|json!({"id":r.id(),"position":r.position(),"score":r.score(),"answer":answer_json(r.answer())})).collect::<Vec<_>>(),
        "unscored":result.unscored().iter().map(|(id,result)|json!({"id":id,"result":result_json(result)})).collect::<Vec<_>>(),"top":top})
}
// Decisions, orders, IDs, fields, and errors are exact. Numeric evidence uses a
// small absolute tolerance across Python/Rust arithmetic and JSON parsing.
fn compare(actual: &Value, expected: &Value, label: &str) {
    match (actual, expected) {
        (Value::Number(a), Value::Number(b)) => assert!(
            (a.as_f64().unwrap() - b.as_f64().unwrap()).abs() <= 1e-14,
            "{label}: {actual} != {expected}"
        ),
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len(), "{label}");
            for (a, b) in a.iter().zip(b) {
                compare(a, b, label);
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(a.len(), b.len(), "{label}: {actual} != {expected}");
            for (key, b) in b {
                compare(
                    a.get(key)
                        .unwrap_or_else(|| panic!("{label}: missing {key}")),
                    b,
                    label,
                );
            }
        }
        _ => assert_eq!(actual, expected, "{label}"),
    }
}
#[test]
fn provenance_matches_recorded_reference_files_and_case_count() {
    let fixture = fixture();
    let baseline: Value =
        serde_json::from_str(include_str!("../docs/source-baseline.json")).unwrap();
    assert_eq!(
        fixture["reference_revision"],
        baseline["sources"][0]["revision"]
    );
    for file in fixture["reference_files"].as_array().unwrap() {
        assert!(
            baseline["sources"][0]["files"]
                .as_array()
                .unwrap()
                .contains(file)
        );
    }
    assert_eq!(fixture["reference_files"].as_array().unwrap().len(), 6);
    assert_eq!(
        [
            "filter_cases",
            "ranking_cases",
            "frame_cases",
            "region_cases"
        ]
        .iter()
        .map(|k| fixture[k].as_array().unwrap().len())
        .sum::<usize>(),
        25
    );
    assert_eq!(fixture["generator_sha256"].as_str().unwrap().len(), 64);
}
#[test]
fn all_six_python_filter_cases_preserve_outcomes_and_provider_inputs() {
    futures_executor::block_on(async {
        for case in fixture()["filter_cases"].as_array().unwrap() {
            let provider = Synthetic::default();
            let id = case["id"].as_str().unwrap();
            let question = Boolean::new("match?")
                .unwrap()
                .with_min_confidence(case["threshold"].as_f64().unwrap())
                .unwrap();
            let items = case["items"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| {
                    Item::new(
                        v[0].as_str().unwrap(),
                        State::try_from(v[1].clone()).unwrap(),
                    )
                    .unwrap()
                })
                .collect::<Vec<_>>();
            let options = BatchOptions {
                context: Some(case["context"].clone()),
                ..Default::default()
            };
            let result = filter_items(&provider, &question, &items, &options)
                .await
                .unwrap();
            compare(&filter_json(&result), &case["expected"], id);
            compare(
                &json!(*provider.seen.lock().unwrap()),
                &case["expected_states"],
                id,
            );
        }
    });
}
#[test]
fn all_ten_python_ranking_cases_preserve_evidence_and_reweight_without_calls() {
    futures_executor::block_on(async {
        for case in fixture()["ranking_cases"].as_array().unwrap() {
            let provider = Synthetic::default();
            let id = case["id"].as_str().unwrap();
            let question = Score::new(
                "relevance?",
                vec!["none".into(), "some".into(), "full".into()],
            )
            .unwrap()
            .with_min_confidence(case["threshold"].as_f64().unwrap())
            .unwrap();
            let candidates = case["items"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| Candidate::new(v[0].as_str().unwrap(), v[1].clone()).unwrap())
                .collect::<Vec<_>>();
            let options = BatchOptions {
                context: Some(case["context"].clone()),
                ..Default::default()
            };
            let result = rerank(
                &provider,
                &question,
                "synthetic query",
                &candidates,
                &options,
            )
            .await
            .unwrap();
            compare(&ranking_json(&result), &case["expected"], id);
            compare(
                &json!(*provider.seen.lock().unwrap()),
                &case["expected_states"],
                id,
            );
            if let Some(weights) = case["weights"].as_array() {
                let weights = weights
                    .iter()
                    .map(|w| w.as_f64().unwrap())
                    .collect::<Vec<_>>();
                let reweighted = result.reweighted(&weights);
                if case.get("expected_reweight_error").is_some() {
                    assert!(reweighted.is_err(), "{id}");
                } else {
                    compare(
                        &ranking_json(&reweighted.unwrap()),
                        &case["expected_reweighted"],
                        id,
                    );
                }
                assert_eq!(
                    provider.seen.lock().unwrap().len(),
                    candidates.len(),
                    "{id}: reweight made calls"
                );
            }
        }
    });
}
#[test]
fn all_five_python_region_cases_match_pixel_rounding_and_padding() {
    for case in fixture()["region_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let e = case["edges"].as_array().unwrap();
        let region = Region::new(
            e[0].as_f64().unwrap(),
            e[1].as_f64().unwrap(),
            e[2].as_f64().unwrap(),
            e[3].as_f64().unwrap(),
        )
        .unwrap();
        compare(&json!(region.center()), &case["expected_center"], id);
        compare(
            &json!(
                region
                    .pixels(
                        case["size"][0].as_u64().unwrap() as u32,
                        case["size"][1].as_u64().unwrap() as u32
                    )
                    .unwrap()
            ),
            &case["expected_pixels"],
            id,
        );
        let padded = region.padded(case["padding"].as_f64().unwrap());
        if case.get("expected_padding_error").is_some() {
            assert!(padded.is_err(), "{id}");
        } else {
            let r = padded.unwrap();
            compare(
                &json!({"top":r.top(),"bottom":r.bottom(),"left":r.left(),"right":r.right()}),
                &case["expected_padded"],
                id,
            );
        }
    }
}
#[cfg(feature = "images")]
#[test]
fn all_four_python_frame_cases_preserve_order_and_timestamps() {
    use semantic_operators::image::{Assets, Image, frames};
    let fixture = fixture();
    for case in fixture["frame_cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let images = [
            Image::from_base64(fixture["image_base64"].as_str().unwrap()).unwrap(),
            Image::from_base64(fixture["image_base64"].as_str().unwrap()).unwrap(),
        ];
        let selected = case["images"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| images[i.as_u64().unwrap() as usize].clone());
        let times = case["times"]
            .as_array()
            .map(|t| t.iter().map(|v| v.as_f64().unwrap()).collect::<Vec<_>>());
        let mut assets = Assets::new();
        let result = frames(&mut assets, selected, times.as_deref());
        if case.get("expected_error").is_some() {
            assert!(result.is_err(), "{id}");
            assert!(assets.is_empty(), "{id}: invalid input mutated assets");
        } else {
            let mut actual = result.unwrap().into_value();
            for frame in actual.as_array_mut().unwrap() {
                let index = images
                    .iter()
                    .position(|image| frame["frame"] == image.placeholder())
                    .unwrap();
                frame["frame"] = json!(format!("frame-{index}"));
            }
            compare(&actual, &case["expected"], id);
        }
    }
}
