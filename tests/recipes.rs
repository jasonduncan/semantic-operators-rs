//! Offline per-item filtering, concurrency, ranking, and reweighting contracts.
use indexmap::IndexMap;
use semantic_operators::each::{BatchOptions, Item};
use semantic_operators::filter::{FilterOutcome, filter_items};
use semantic_operators::rerank::{Candidate, rerank};
use semantic_operators::{
    AnswerValue, Answers, Boolean, Call, Error, ErrorKind, Provider, ProviderFuture, Question,
    Request, Score, State, make_answer,
};
use serde_json::{Value, json};
use std::{
    num::NonZeroUsize,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

#[derive(Clone)]
struct Case {
    probabilities: Vec<f64>,
    model: Option<String>,
    error: Option<ErrorKind>,
    delay_ms: u64,
}
impl Case {
    fn boolean(p: f64) -> Self {
        Self {
            probabilities: vec![p, 1.0 - p],
            model: Some("reported-model".into()),
            error: None,
            delay_ms: 0,
        }
    }
    fn score(p: &[f64]) -> Self {
        Self {
            probabilities: p.to_vec(),
            ..Self::boolean(1.0)
        }
    }
    fn error(kind: ErrorKind) -> Self {
        Self {
            error: Some(kind),
            ..Self::boolean(1.0)
        }
    }
    fn delayed(mut self, ms: u64) -> Self {
        self.delay_ms = ms;
        self
    }
    fn model(mut self, name: &str) -> Self {
        self.model = Some(name.into());
        self
    }
}
struct RecipeProvider {
    cases: IndexMap<String, Case>,
    calls: AtomicUsize,
    active: AtomicUsize,
    peak: AtomicUsize,
    states: Mutex<Vec<Value>>,
    completed: Mutex<Vec<String>>,
}
impl RecipeProvider {
    fn new(cases: impl IntoIterator<Item = (&'static str, Case)>) -> Self {
        Self {
            cases: cases.into_iter().map(|(k, v)| (k.into(), v)).collect(),
            calls: AtomicUsize::new(0),
            active: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
            states: Mutex::new(Vec::new()),
            completed: Mutex::new(Vec::new()),
        }
    }
}
struct Active<'a>(&'a AtomicUsize);
impl Drop for Active<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}
impl Provider for RecipeProvider {
    fn ask<'a>(&'a self, request: &'a Request) -> ProviderFuture<'a> {
        Box::pin(async move {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
            let _guard = Active(&self.active);
            self.peak.fetch_max(active, Ordering::SeqCst);
            assert_eq!(request.questions().len(), 1);
            assert!(request.questions().contains_key("q"));
            let state = request.state().as_value();
            self.states.lock().unwrap().push(state.clone());
            let item = state
                .get("document")
                .or_else(|| state.get("item"))
                .unwrap_or(state);
            let key = item.as_str().unwrap();
            let case = &self.cases[key];
            if case.delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(case.delay_ms)).await;
            }
            self.completed.lock().unwrap().push(key.into());
            if let Some(kind) = case.error {
                return Err(Error::new(kind, "scripted item failure"));
            }
            let q = &request.questions()["q"];
            let (value, probabilities) = match q {
                Question::Boolean(_) => (
                    AnswerValue::Boolean(case.probabilities[0] >= case.probabilities[1]),
                    IndexMap::from([
                        ("true".into(), case.probabilities[0]),
                        ("false".into(), case.probabilities[1]),
                    ]),
                ),
                Question::Score(q) => (
                    AnswerValue::Score(
                        case.probabilities
                            .iter()
                            .enumerate()
                            .map(|(i, p)| i as f64 * p)
                            .sum(),
                    ),
                    q.levels()
                        .iter()
                        .cloned()
                        .zip(case.probabilities.iter().copied())
                        .collect(),
                ),
                _ => unreachable!(),
            };
            let answer = make_answer(q, value, probabilities, None)?.with_call(Arc::new(Call {
                provider: "script".into(),
                model: case.model.clone(),
                input_tokens: None,
                output_tokens: None,
            }));
            Ok(Answers::from([("q".into(), answer)]))
        })
    }
}
fn item(id: &str, text: &str) -> Item {
    Item::new(id, State::text(text)).unwrap()
}
fn candidate(id: &str, text: &str) -> Candidate {
    Candidate::new(id, json!(text)).unwrap()
}
fn score() -> Score {
    Score::new(
        "How useful?",
        vec!["none".into(), "some".into(), "full".into()],
    )
    .unwrap()
}

#[tokio::test]
async fn filter_keeps_every_outcome_and_preserves_context_without_ids() {
    let provider = RecipeProvider::new([
        ("yes", Case::boolean(0.9)),
        ("no", Case::boolean(0.1)),
        ("tie", Case::boolean(0.5)),
        ("low", Case::boolean(0.7)),
        ("fail", Case::error(ErrorKind::Provider)),
        ("timeout", Case::error(ErrorKind::Timeout)),
        ("bad", Case::error(ErrorKind::InvalidResponse)),
    ]);
    let items = [
        item("secret-id-1", "yes"),
        item("secret-id-2", "no"),
        item("secret-id-3", "tie"),
        item("secret-id-4", "low"),
        item("secret-id-5", "fail"),
        item("secret-id-6", "timeout"),
        item("secret-id-7", "bad"),
    ];
    let options = BatchOptions {
        context: Some(json!({"rubric":"shared"})),
        ..Default::default()
    };
    let question = Boolean::new("Match?")
        .unwrap()
        .with_min_confidence(0.8)
        .unwrap();
    let result = filter_items(&provider, &question, &items, &options)
        .await
        .unwrap();
    assert_eq!(
        result
            .verdicts()
            .iter()
            .map(|v| v.outcome())
            .collect::<Vec<_>>(),
        [
            FilterOutcome::Match,
            FilterOutcome::No,
            FilterOutcome::Unsure,
            FilterOutcome::Unsure,
            FilterOutcome::Failed,
            FilterOutcome::Failed,
            FilterOutcome::Failed
        ]
    );
    assert_eq!(result.matched().len(), 1);
    assert_eq!(result.rejected().len(), 1);
    assert_eq!(result.unsure().len(), 2);
    assert_eq!(result.failed().len(), 3);
    assert_eq!(
        result.models().into_iter().collect::<Vec<_>>(),
        ["reported-model"]
    );
    for (position, verdict) in result.verdicts().iter().enumerate() {
        assert_eq!(verdict.position(), position);
        assert_eq!(verdict.id(), items[position].id());
    }
    for state in provider.states.lock().unwrap().iter() {
        assert_eq!(state.as_object().unwrap().len(), 2);
        assert_eq!(state["context"], json!({"rubric":"shared"}));
        assert!(!state.to_string().contains("secret-id"));
    }
    assert_eq!(result.unsure()[0].answer().unwrap().confidence(), 0.5);
    assert_eq!(
        result.failed()[1].error().unwrap().kind(),
        ErrorKind::Timeout
    );
}
#[tokio::test]
async fn duplicate_ids_invalid_query_and_empty_inputs_do_no_work() {
    let provider = RecipeProvider::new([]);
    let options = BatchOptions::default();
    let question = Boolean::new("Match?").unwrap();
    assert!(Item::new("", State::text("data")).is_err());
    assert!(Candidate::new("", Value::Null).is_err());
    assert!(
        filter_items(
            &provider,
            &question,
            &[item("a", "one"), item("a", "two")],
            &options
        )
        .await
        .is_err()
    );
    assert!(
        rerank(
            &provider,
            &score(),
            "query",
            &[candidate("a", "one"), candidate("a", "two")],
            &options
        )
        .await
        .is_err()
    );
    assert!(
        rerank(&provider, &score(), " ", &[], &options)
            .await
            .is_err()
    );
    assert!(
        filter_items(&provider, &question, &[], &options)
            .await
            .unwrap()
            .verdicts()
            .is_empty()
    );
    assert!(
        rerank(&provider, &score(), "query", &[], &options)
            .await
            .unwrap()
            .complete()
    );
    assert_eq!(provider.calls.load(Ordering::SeqCst), 0);
    assert_eq!(options.concurrency.get(), 8);
}
#[tokio::test]
async fn bounded_concurrency_restores_input_order_after_out_of_order_completion() {
    let provider = RecipeProvider::new([
        ("slow", Case::boolean(0.9).delayed(40)),
        ("fast", Case::boolean(0.9).delayed(1)),
        ("middle", Case::boolean(0.1).delayed(2)),
    ]);
    let options = BatchOptions {
        concurrency: NonZeroUsize::new(2).unwrap(),
        ..Default::default()
    };
    let result = filter_items(
        &provider,
        &Boolean::new("Match?").unwrap(),
        &[
            item("first", "slow"),
            item("second", "fast"),
            item("third", "middle"),
        ],
        &options,
    )
    .await
    .unwrap();
    assert_eq!(provider.peak.load(Ordering::SeqCst), 2);
    assert_eq!(provider.active.load(Ordering::SeqCst), 0);
    assert_eq!(
        provider.completed.lock().unwrap().as_slice(),
        ["fast", "middle", "slow"]
    );
    assert_eq!(
        result.verdicts().iter().map(|v| v.id()).collect::<Vec<_>>(),
        ["first", "second", "third"]
    );
}
#[tokio::test]
async fn configuration_errors_abort_and_drop_pending_work_while_provider_errors_continue() {
    let provider = RecipeProvider::new([
        ("wait", Case::boolean(0.9).delayed(1000)),
        ("invalid", Case::error(ErrorKind::InvalidArgument)),
        ("never", Case::boolean(0.9)),
    ]);
    let options = BatchOptions {
        concurrency: NonZeroUsize::new(2).unwrap(),
        ..Default::default()
    };
    let error = filter_items(
        &provider,
        &Boolean::new("Match?").unwrap(),
        &[
            item("first", "wait"),
            item("second", "invalid"),
            item("third", "never"),
        ],
        &options,
    )
    .await
    .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidArgument);
    assert_eq!(provider.active.load(Ordering::SeqCst), 0);
    assert_eq!(provider.calls.load(Ordering::SeqCst), 2);
    let unsupported = RecipeProvider::new([("unsupported", Case::error(ErrorKind::Unsupported))]);
    assert_eq!(
        filter_items(
            &unsupported,
            &Boolean::new("Match?").unwrap(),
            &[item("id", "unsupported")],
            &BatchOptions::default()
        )
        .await
        .unwrap_err()
        .kind(),
        ErrorKind::Unsupported
    );
}
#[tokio::test]
async fn dropping_batch_future_cancels_local_polled_calls() {
    let provider = RecipeProvider::new([("wait", Case::boolean(0.9).delayed(1000))]);
    let items = [
        item("one", "wait"),
        item("two", "wait"),
        item("three", "wait"),
    ];
    let question = Boolean::new("Match?").unwrap();
    let options = BatchOptions {
        concurrency: NonZeroUsize::new(2).unwrap(),
        ..Default::default()
    };
    assert!(
        tokio::time::timeout(
            Duration::from_millis(5),
            filter_items(&provider, &question, &items, &options)
        )
        .await
        .is_err()
    );
    assert_eq!(provider.active.load(Ordering::SeqCst), 0);
    assert_eq!(provider.calls.load(Ordering::SeqCst), 2);
}
#[tokio::test]
async fn ranking_retains_partial_evidence_stable_ties_and_local_identifiers() {
    let provider = RecipeProvider::new([
        ("first", Case::score(&[0.1, 0.2, 0.7])),
        ("second", Case::score(&[0.1, 0.2, 0.7])),
        ("low", Case::score(&[0.6, 0.2, 0.2])),
        ("failure", Case::error(ErrorKind::Provider)),
    ]);
    let question = score().with_min_confidence(0.65).unwrap();
    let candidates = [
        candidate("a-secret", "first"),
        candidate("b-secret", "second"),
        candidate("c-secret", "low"),
        candidate("d-secret", "failure"),
    ];
    let options = BatchOptions {
        context: Some(json!(["background"])),
        ..Default::default()
    };
    let ranking = rerank(&provider, &question, "target query", &candidates, &options)
        .await
        .unwrap();
    assert!(!ranking.complete());
    assert!(ranking.top(10, false).is_err());
    assert_eq!(ranking.top(10, true).unwrap().len(), 2);
    assert_eq!(
        ranking.ranked().iter().map(|r| r.id()).collect::<Vec<_>>(),
        ["a-secret", "b-secret"]
    );
    assert_eq!(
        ranking
            .unscored()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["c-secret", "d-secret"]
    );
    assert!(!ranking.unscored()["c-secret"].answer().unwrap().decided());
    assert!(ranking.unscored()["d-secret"].error().is_some());
    assert_eq!(ranking.ranked()[0].position(), 0);
    assert_eq!(ranking.ranked()[1].position(), 1);
    for state in provider.states.lock().unwrap().iter() {
        assert_eq!(state["query"], "target query");
        assert_eq!(state["context"], json!(["background"]));
        assert!(!state.to_string().contains("secret"));
    }
}
#[tokio::test]
async fn mixed_models_including_undecided_always_prevent_top() {
    let provider = RecipeProvider::new([
        ("decided", Case::score(&[0.0, 0.0, 1.0]).model("model-a")),
        ("unsure", Case::score(&[0.4, 0.3, 0.3]).model("model-b")),
    ]);
    let ranking = rerank(
        &provider,
        &score().with_min_confidence(0.8).unwrap(),
        "query",
        &[candidate("a", "decided"), candidate("b", "unsure")],
        &BatchOptions::default(),
    )
    .await
    .unwrap();
    assert_eq!(ranking.models().len(), 2);
    assert!(ranking.top(1, true).is_err());
    assert!(ranking.top(0, true).is_err());
    let provider = RecipeProvider::new([
        ("a", Case::score(&[0.0, 0.0, 1.0]).model("")),
        ("b", Case::score(&[0.0, 0.0, 1.0]).model("model")),
    ]);
    let ranking = rerank(
        &provider,
        &score(),
        "query",
        &[candidate("a", "a"), candidate("b", "b")],
        &BatchOptions::default(),
    )
    .await
    .unwrap();
    assert!(ranking.top(9, false).is_ok());
    assert_eq!(ranking.models().len(), 1);
}
#[tokio::test]
async fn reweighting_changes_order_without_calls_and_normalizes_probability_mass() {
    let provider = RecipeProvider::new([
        ("a", Case::score(&[0.0, 0.8, 0.2])),
        ("b", Case::score(&[0.5, 0.0, 0.5])),
        ("mass", Case::score(&[0.2, 0.8, 0.002])),
    ]);
    let ranking = rerank(
        &provider,
        &score(),
        "query",
        &[
            candidate("a", "a"),
            candidate("b", "b"),
            candidate("mass", "mass"),
        ],
        &BatchOptions::default(),
    )
    .await
    .unwrap();
    assert_eq!(ranking.ranked()[0].id(), "a");
    let changed = ranking.reweighted(&[0.0, 1.0, 100.0]).unwrap();
    assert_eq!(changed.ranked()[0].id(), "b");
    let normalized = ranking.reweighted(&[0.0, 1.0, 2.0]).unwrap();
    let mass = normalized
        .ranked()
        .iter()
        .find(|r| r.id() == "mass")
        .unwrap();
    assert!((mass.score() - 0.804 / 1.002).abs() < 1e-15);
    assert_eq!(
        changed.ranked()[0].answer().value(),
        ranking.ranked()[1].answer().value()
    );
    assert_eq!(provider.calls.load(Ordering::SeqCst), 3);
    for weights in [
        vec![0.0, 1.0],
        vec![0.0, 1.0, 1.0],
        vec![2.0, 1.0, 0.0],
        vec![0.0, 1.0, f64::NAN],
        vec![0.0, 1.0, f64::INFINITY],
    ] {
        assert!(ranking.reweighted(&weights).is_err());
    }
    assert!(ranking.reweighted(&[-3.0, -1.0, 2.0]).is_ok());
}
#[tokio::test]
async fn null_context_is_omitted_and_arbitrary_json_documents_are_supported() {
    struct Inspect;
    impl Provider for Inspect {
        fn ask<'a>(&'a self, r: &'a Request) -> ProviderFuture<'a> {
            Box::pin(async move {
                assert!(
                    !r.state()
                        .as_value()
                        .as_object()
                        .unwrap()
                        .contains_key("context")
                );
                assert_eq!(r.state().as_value()["document"], Value::Null);
                Ok(Answers::from([(
                    "q".into(),
                    make_answer(
                        &r.questions()["q"],
                        AnswerValue::Score(2.0),
                        IndexMap::from([
                            ("none".into(), 0.0),
                            ("some".into(), 0.0),
                            ("full".into(), 1.0),
                        ]),
                        None,
                    )?,
                )]))
            })
        }
    }
    let options = BatchOptions {
        context: Some(Value::Null),
        ..Default::default()
    };
    let ranking = rerank(
        &Inspect,
        &score(),
        "query",
        &[Candidate::new("null", Value::Null).unwrap()],
        &options,
    )
    .await
    .unwrap();
    assert!(ranking.complete());
}

#[cfg(feature = "images")]
#[tokio::test]
async fn invalid_image_reference_in_later_item_prevents_all_dispatch() {
    let provider = RecipeProvider::new([("ordinary", Case::boolean(1.0))]);
    let items = [
        item("ok", "ordinary"),
        item("missing", "image-00000000-0000-0000-0000-000000000099"),
    ];
    let result = filter_items(
        &provider,
        &Boolean::new("Match?").unwrap(),
        &items,
        &BatchOptions::default(),
    )
    .await;
    assert_eq!(result.unwrap_err().kind(), ErrorKind::InvalidImage);
    assert_eq!(provider.calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn positive_and_negative_zero_are_exact_ties() {
    struct SignedZero;
    impl Provider for SignedZero {
        fn ask<'a>(&'a self, r: &'a Request) -> ProviderFuture<'a> {
            Box::pin(async move {
                let value = if r.state().as_value()["document"] == "minus" {
                    -0.0
                } else {
                    0.0
                };
                Ok(Answers::from([(
                    "q".into(),
                    make_answer(
                        &r.questions()["q"],
                        AnswerValue::Score(value),
                        IndexMap::from([
                            ("none".into(), 1.0),
                            ("some".into(), 0.0),
                            ("full".into(), 0.0),
                        ]),
                        None,
                    )?,
                )]))
            })
        }
    }
    let ranking = rerank(
        &SignedZero,
        &score(),
        "query",
        &[candidate("first", "minus"), candidate("second", "plus")],
        &BatchOptions::default(),
    )
    .await
    .unwrap();
    assert_eq!(
        ranking.ranked().iter().map(|r| r.id()).collect::<Vec<_>>(),
        ["first", "second"]
    );
}
