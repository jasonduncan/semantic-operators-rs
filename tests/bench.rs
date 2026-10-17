//! Python-generated benchmark parity and bounded runner regressions.
use semantic_operators::{
    bench::*,
    flow::{FlowContext, FlowExit, FlowOutcome, Step},
    *,
};
use serde_json::{Value, json};
use std::{
    num::NonZeroUsize,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/python-bench.json")).unwrap()
}
fn questions() -> Questions {
    IndexMap::from([
        ("b".into(), Boolean::new("q").unwrap().into()),
        (
            "c".into(),
            Choice::new(
                "q",
                IndexMap::from([("a".into(), None), ("b".into(), None)]),
            )
            .unwrap()
            .into(),
        ),
        (
            "s".into(),
            Score::new("q", vec!["low".into(), "middle".into(), "high".into()])
                .unwrap()
                .into(),
        ),
    ])
}
fn decision(value: &Value) -> Decision {
    match value {
        Value::Bool(v) => Decision::Boolean(*v),
        Value::String(v) => Decision::Choice(v.clone()),
        Value::Number(v) => Decision::Score(v.as_u64().unwrap() as usize),
        _ => panic!(),
    }
}
fn label_json(value: &Decision) -> Value {
    match value {
        Decision::Boolean(v) => json!(v),
        Decision::Choice(v) => json!(v),
        Decision::Score(v) => json!(v),
    }
}
fn cases() -> Vec<Case> {
    fixture()["labels"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(i, row)| {
            Case::new(
                State::text(format!("synthetic-{i}")),
                row.as_object()
                    .unwrap()
                    .iter()
                    .map(|(name, value)| (name.clone(), decision(value)))
                    .collect(),
            )
        })
        .collect()
}
fn answers() -> Vec<Answers> {
    let q = questions();
    fixture()["answers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            row.as_object()
                .unwrap()
                .iter()
                .map(|(name, spec)| {
                    let value = match name.as_str() {
                        "b" => AnswerValue::Boolean(spec[0].as_bool().unwrap()),
                        "c" => AnswerValue::Choice(spec[0].as_str().unwrap().into()),
                        _ => AnswerValue::Score(spec[0].as_f64().unwrap()),
                    };
                    (
                        name.clone(),
                        make_answer(
                            &q[name],
                            value,
                            serde_json::from_value(spec[1].clone()).unwrap(),
                            None,
                        )
                        .unwrap(),
                    )
                })
                .collect()
        })
        .collect()
}
fn report() -> Report {
    score(&questions(), &cases(), answers(), vec![1., 2., 3., 4.], 9.).unwrap()
}
fn report_json(r: &Report) -> Value {
    let stats:serde_json::Map<String,Value>=r.questions().iter().map(|(name,s)|(name.clone(),json!({"correct":s.correct(),"undecided":s.undecided(),"total":s.total(),"answered":s.answered(),"accuracy":s.accuracy(),"accuracy_when_answered":s.accuracy_when_answered(),"mean_p_correct":s.mean_p_correct(),"p_correct":s.p_correct()}))).collect();
    let decisions: Vec<Value> = r
        .decisions()
        .iter()
        .map(|row| {
            Value::Object(
                row.iter()
                    .map(|(name, v)| {
                        (
                            name.clone(),
                            v.as_ref().map(label_json).unwrap_or(Value::Null),
                        )
                    })
                    .collect(),
            )
        })
        .collect();
    json!({"questions":stats,"decisions":decisions,"misses":r.misses().iter().map(|m|json!({"case":m.case,"question":m.question,"expected":label_json(&m.expected)})).collect::<Vec<_>>(),"latencies_ms":r.latencies_ms(),"total_ms":r.total_ms()})
}
#[test]
fn pure_reports_thresholds_stability_and_ndcg_match_python() {
    let f = fixture();
    let r = report();
    assert_eq!(report_json(&r), f["report"]);
    for case in f["thresholds"].as_array().unwrap() {
        let rescored = at_min_confidence(
            &r,
            &questions(),
            &cases(),
            case["threshold"].as_f64().unwrap(),
        )
        .unwrap();
        assert_eq!(report_json(&rescored), case["expected"]);
    }
    let strict = at_min_confidence(&r, &questions(), &cases(), 0.95).unwrap();
    assert_eq!(
        report_json(&at_min_confidence(&strict, &questions(), &cases(), 0.).unwrap()),
        f["no_restore"]
    );
    assert_eq!(
        serde_json::to_value(stability(&[r, strict]).unwrap()).unwrap(),
        f["stability"]
    );
    for case in f["ndcg"].as_array().unwrap() {
        let order = serde_json::from_value::<Vec<String>>(case["order"].clone()).unwrap();
        let grades = serde_json::from_value(case["grades"].clone()).unwrap();
        let got = ndcg(&order, &grades, case["k"].as_u64().unwrap() as usize).unwrap();
        if case["expected"].is_null() {
            assert_eq!(got, None);
        } else {
            assert!((got.unwrap() - case["expected"].as_f64().unwrap()).abs() < 1e-14);
        }
    }
    assert_eq!(
        ndcg_default(&["a".into()], &IndexMap::from([("a".into(), 1)])).unwrap(),
        Some(1.)
    );
}
#[test]
fn empty_subset_labels_misalignment_and_invalid_numeric_inputs() {
    let empty = score(&questions(), &[], vec![], vec![], 0.).unwrap();
    assert_eq!(empty.questions()["b"].accuracy(), 0.);
    assert_eq!(empty.questions()["b"].accuracy_when_answered(), 0.);
    assert_eq!(empty.questions()["b"].mean_p_correct(), 0.);
    assert!(stability(&[empty]).is_err());
    assert!(stability(&[]).is_err());
    let q = questions();
    let unlabeled = vec![Case::new(State::text("s"), IndexMap::new())];
    let r = score(&q, &unlabeled, vec![answers().remove(0)], vec![0.], 0.).unwrap();
    assert_eq!(r.questions()["b"].total(), 0);
    assert!(r.decisions()[0]["b"].is_some());
    assert!(score(&q, &cases(), vec![], vec![0.; 4], 0.).is_err());
    assert!(score(&q, &cases(), answers(), vec![], 0.).is_err());
    assert!(score(&q, &cases(), answers(), vec![0.; 4], f64::NAN).is_err());
    for bad in [Decision::Boolean(true), Decision::Score(99)] {
        let cases = vec![Case::new(
            State::text("s"),
            IndexMap::from([("s".into(), bad)]),
        )];
        assert!(score(&q, &cases, vec![answers().remove(0)], vec![0.], 0.).is_err());
    }
    assert!(ndcg(&["a".into()], &IndexMap::from([("a".into(), u32::MAX)]), 10).is_err());
    assert!(at_min_confidence(&report(), &q, &cases(), f64::NAN).is_err());
    let boxed = BoxQuestion::new("q").unwrap();
    let a = make_answer(
        &boxed.clone().into(),
        AnswerValue::Box(Region::new(0., 1., 0., 1.).unwrap()),
        IndexMap::new(),
        None,
    )
    .unwrap()
    .abstain();
    assert!(decide(&boxed.into(), &a).is_err());
}
struct Counting {
    calls: AtomicUsize,
    active: Arc<AtomicUsize>,
    peak: AtomicUsize,
    fail: Option<usize>,
}
struct Active(Arc<AtomicUsize>);
impl Drop for Active {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}
impl Provider for Counting {
    fn ask<'a>(&'a self, r: &'a Request) -> ProviderFuture<'a> {
        Box::pin(async move {
            let call = self.calls.fetch_add(1, Ordering::SeqCst);
            let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
            let _guard = Active(self.active.clone());
            self.peak.fetch_max(active, Ordering::SeqCst);
            if self.fail == Some(call) {
                return Err(Error::new(ErrorKind::Provider, "scripted"));
            }
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            Ok(r.questions()
                .iter()
                .map(|(name, q)| {
                    (
                        name.clone(),
                        make_answer(
                            q,
                            AnswerValue::Boolean(true),
                            IndexMap::from([("true".into(), 0.9), ("false".into(), 0.1)]),
                            None,
                        )
                        .unwrap(),
                    )
                })
                .collect())
        })
    }
}
fn counting(fail: Option<usize>) -> Counting {
    Counting {
        calls: AtomicUsize::new(0),
        active: Arc::new(AtomicUsize::new(0)),
        peak: AtomicUsize::new(0),
        fail,
    }
}
fn run_data() -> (Questions, Vec<Case>) {
    (
        IndexMap::from([("q".into(), Boolean::new("q").unwrap().into())]),
        (0..5)
            .map(|i| {
                Case::new(
                    State::text(i.to_string()),
                    IndexMap::from([("q".into(), Decision::Boolean(true))]),
                )
            })
            .collect(),
    )
}
#[tokio::test]
async fn runner_exposes_warmup_cost_and_bounded_measured_work() {
    let p = counting(None);
    let (q, cases) = run_data();
    let options = RunOptions {
        concurrency: NonZeroUsize::new(2).unwrap(),
        ..Default::default()
    };
    let r = run(&p, &q, &cases, &options).await.unwrap();
    assert_eq!(p.calls.load(Ordering::SeqCst), 6);
    assert_eq!(r.warmup_calls(), 1);
    assert_eq!(r.calls(), 6);
    assert_eq!(r.questions()["q"].correct(), 5);
    assert_eq!(p.peak.load(Ordering::SeqCst), 2);
    assert_eq!(p.active.load(Ordering::SeqCst), 0);
    assert_eq!(RunOptions::default().concurrency.get(), 4);
}
#[tokio::test]
async fn runner_validates_labels_before_warmup_and_aborts_provider_failure() {
    let p = counting(None);
    let (q, mut cases) = run_data();
    cases.push(Case::new(
        State::text("invalid"),
        IndexMap::from([("unknown".into(), Decision::Boolean(true))]),
    ));
    assert!(run(&p, &q, &cases, &RunOptions::default()).await.is_err());
    assert_eq!(p.calls.load(Ordering::SeqCst), 0);
    let p = counting(Some(1));
    let (q, cases) = run_data();
    assert_eq!(
        run(
            &p,
            &q,
            &cases,
            &RunOptions {
                warmup: 0,
                concurrency: NonZeroUsize::new(2).unwrap(),
                ..Default::default()
            }
        )
        .await
        .unwrap_err()
        .kind(),
        ErrorKind::Provider
    );
    assert_eq!(p.active.load(Ordering::SeqCst), 0);
    assert_eq!(p.calls.load(Ordering::SeqCst), 2);
}
fn flow_outcome(value: Option<&str>, stop: Option<&str>, calls: usize) -> FlowOutcome<String> {
    let request = Request::new(
        State::text("s"),
        IndexMap::from([("q".into(), Boolean::new("q").unwrap().into())]),
    )
    .unwrap();
    FlowOutcome {
        value: value.map(str::to_owned),
        stopped_at: stop.map(str::to_owned),
        stopped_answer: None,
        trace: (0..calls)
            .map(|_| Step {
                request: request.clone(),
                result: Ok(IndexMap::new()),
            })
            .collect(),
    }
}
#[test]
fn traced_flow_reports_match_python_and_reject_misalignment() {
    let report = score_flows(
        vec![
            flow_outcome(Some("yes"), None, 1),
            flow_outcome(None, Some("q"), 2),
            flow_outcome(Some("no"), None, 0),
        ],
        vec!["yes".into(); 3],
        vec![1., 2., 3.],
    )
    .unwrap();
    let observed = json!({"total":report.total(),"stopped":report.stopped(),"correct":report.correct(),"accuracy_when_answered":report.accuracy_when_answered(),"calls":report.calls(),"misses":report.misses().iter().map(|m|json!([m.case,m.expected,m.got])).collect::<Vec<_>>()});
    assert_eq!(observed, fixture()["flow"]);
    assert!(score_flows(vec![flow_outcome(Some("yes"), None, 1)], vec![], vec![0.]).is_err());
    let empty = score_flows::<String>(vec![], vec![], vec![]).unwrap();
    assert_eq!(empty.accuracy_when_answered(), 0.);
}
#[tokio::test]
async fn flow_runner_accepts_borrowing_async_closure_and_retains_traces() {
    let p = counting(None);
    let op = Operator::new("q", Boolean::new("q").unwrap()).unwrap();
    let cases = vec![(State::text("first"), true), (State::text("second"), true)];
    let report = run_flow(&cases, |state| {
        let op = &op;
        let p = &p;
        async move {
            let mut flow = FlowContext::new(p);
            let result: std::result::Result<bool, FlowExit> = async {
                let answers = flow.ask(state, &[op.erased()]).await?;
                answers.require(op)
            }
            .await;
            flow.finish(result)
        }
    })
    .await
    .unwrap();
    assert_eq!(report.correct(), 2);
    assert_eq!(report.calls(), 2);
    assert_eq!(report.results()[0].trace[0].request.state(), &cases[0].0);
}
#[test]
fn stricter_scoring_stores_abstention_and_stability_rejects_misalignment() {
    let q = questions();
    let mut stricter = q.clone();
    stricter.insert(
        "b".into(),
        Boolean::new("q")
            .unwrap()
            .with_min_confidence(0.95)
            .unwrap()
            .into(),
    );
    let r = score(&stricter, &cases(), answers(), vec![0.; 4], 0.).unwrap();
    assert!(!r.answers()[0]["b"].decided());
    let lower = at_min_confidence(&r, &q, &cases(), 0.).unwrap();
    assert!(!lower.answers()[0]["b"].decided());
    let short = score(&q, &cases()[..1], vec![answers().remove(0)], vec![0.], 0.).unwrap();
    assert!(stability(&[report(), short]).is_err());
    let alternate: Questions =
        IndexMap::from([("other".into(), Boolean::new("q").unwrap().into())]);
    let rows = (0..4)
        .map(|_| {
            IndexMap::from([(
                "other".into(),
                make_answer(
                    &alternate["other"],
                    AnswerValue::Boolean(true),
                    IndexMap::from([("true".into(), 1.), ("false".into(), 0.)]),
                    None,
                )
                .unwrap(),
            )])
        })
        .collect();
    let unlabeled = (0..4)
        .map(|_| Case::new(State::text("s"), IndexMap::new()))
        .collect::<Vec<_>>();
    let alternate = score(&alternate, &unlabeled, rows, vec![0.; 4], 0.).unwrap();
    assert!(stability(&[report(), alternate]).is_err());
}
#[cfg(feature = "images")]
#[tokio::test]
async fn benchmark_prevalidates_later_image_handles_before_warmup() {
    let p = counting(None);
    let (q, mut cases) = run_data();
    cases.push(Case::new(
        State::text("image-00000000-0000-0000-0000-000000000099"),
        IndexMap::new(),
    ));
    assert_eq!(
        run(&p, &q, &cases, &RunOptions::default())
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::InvalidImage
    );
    assert_eq!(p.calls.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn warmup_failure_and_flow_failure_preserve_error_identity_and_trace() {
    let p = counting(Some(0));
    let (q, cases) = run_data();
    assert_eq!(
        run(&p, &q, &cases, &RunOptions::default())
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::Provider
    );
    assert_eq!(p.calls.load(Ordering::SeqCst), 1);
    let p = counting(Some(0));
    let op = Operator::new("q", Boolean::new("q").unwrap()).unwrap();
    let cases = [(State::text("s"), true)];
    let error = run_flow(&cases, |state| {
        let p = &p;
        let op = &op;
        async move {
            let mut flow = FlowContext::new(p);
            let result: std::result::Result<bool, FlowExit> =
                async { flow.ask(state, &[op.erased()]).await?.require(op) }.await;
            flow.finish(result)
        }
    })
    .await
    .unwrap_err();
    assert_eq!(error.error.kind(), ErrorKind::Provider);
    assert_eq!(error.trace.len(), 1);
    assert!(error.trace[0].result.is_err());
}
#[test]
fn malformed_flow_outcome_aborts_before_next_case_and_retains_trace() {
    futures_executor::block_on(async {
        for failed_trace in [false, true] {
            let calls = AtomicUsize::new(0);
            let cases = [(0, "yes".to_owned()), (1, "yes".to_owned())];
            let failure = run_flow(&cases, |_| {
                calls.fetch_add(1, Ordering::SeqCst);
                let mut outcome =
                    flow_outcome(if failed_trace { Some("yes") } else { None }, None, 1);
                if failed_trace {
                    outcome.trace[0].result =
                        Err(Error::new(ErrorKind::Provider, "retained trace failure"));
                }
                std::future::ready(Ok(outcome))
            })
            .await
            .unwrap_err();
            assert_eq!(calls.load(Ordering::SeqCst), 1);
            assert_eq!(failure.error.kind(), ErrorKind::InvalidArgument);
            assert_eq!(failure.trace.len(), 1);
            assert_eq!(failure.trace[0].request.state().as_value(), "s");
            assert_eq!(failure.trace[0].result.is_err(), failed_trace);
        }
    });
}
