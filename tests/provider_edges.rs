//! Checked provider and flow boundary regression tests.
use semantic_operators::flow::FlowContext;
use semantic_operators::*;
struct Fixed(Answers);
impl Provider for Fixed {
    fn ask<'a>(&'a self, _: &'a Request) -> ProviderFuture<'a> {
        Box::pin(async move { Ok(self.0.clone()) })
    }
}
fn boolean(threshold: f64) -> Boolean {
    Boolean::new("q")
        .unwrap()
        .with_min_confidence(threshold)
        .unwrap()
}
fn answer(q: &Question, p: f64) -> Answer {
    make_answer(
        q,
        AnswerValue::Boolean(p > 0.5),
        IndexMap::from([("true".into(), p), ("false".into(), complement(p).unwrap())]),
        None,
    )
    .unwrap()
}
#[test]
fn checked_boundary_reorders_rejects_extra_and_preserves_explicit_abstention() {
    futures_executor::block_on(async {
        let q: Question = boolean(0.0).into();
        let request = Request::new(
            State::text("private"),
            IndexMap::from([("a".into(), q.clone()), ("b".into(), q.clone())]),
        )
        .unwrap();
        let provider = Fixed(IndexMap::from([
            ("b".into(), answer(&q, 0.9).abstain()),
            ("a".into(), answer(&q, 0.9)),
        ]));
        let output = ask(&provider, &request).await.unwrap();
        assert_eq!(
            output.keys().map(String::as_str).collect::<Vec<_>>(),
            ["a", "b"]
        );
        assert!(!output["b"].decided());
        let provider = Fixed(IndexMap::from([
            ("a".into(), answer(&q, 0.9)),
            ("extra".into(), answer(&q, 0.9)),
        ]));
        assert_eq!(
            ask(&provider, &request).await.unwrap_err().kind(),
            ErrorKind::InvalidResponse
        );
    });
}
#[test]
fn strict_operator_flow_stop_contains_normalized_undecided_answer() {
    futures_executor::block_on(async {
        let lenient = Operator::new("q", boolean(0.0)).unwrap();
        let strict = Operator::new("q", boolean(0.95)).unwrap();
        let provider = Fixed(IndexMap::from([(
            "q".into(),
            answer(&boolean(0.0).into(), 0.8),
        )]));
        let mut flow = FlowContext::new(&provider);
        let answers = flow
            .ask(&State::text("s"), &[lenient.erased()])
            .await
            .unwrap();
        let outcome = flow.finish(answers.require(&strict)).unwrap();
        assert!(!outcome.decided());
        assert!(!outcome.stopped_answer.unwrap().decided());
    });
}
#[cfg(feature = "tokio")]
#[test]
fn timeout_reports_runtime_error_when_timer_is_disabled() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let q: Question = boolean(0.0).into();
    let request =
        Request::new(State::text("s"), IndexMap::from([("q".into(), q.clone())])).unwrap();
    let provider = TimeLimited::new(
        Fixed(IndexMap::from([("q".into(), answer(&q, 0.9))])),
        std::time::Duration::from_secs(1),
    )
    .unwrap();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        runtime.block_on(ask(&provider, &request))
    }));
    assert!(
        outcome.is_ok(),
        "timeout wrapper panicked when timers were unavailable"
    );
    assert_eq!(outcome.unwrap().unwrap_err().kind(), ErrorKind::Runtime);
}
#[test]
fn function_provider_accepts_borrowing_closure_and_trait_object() {
    use semantic_operators::provider::FunctionProvider;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = calls.clone();
    let provider: Arc<dyn Provider> = Arc::new(FunctionProvider::new(move |request| {
        let calls = observed.clone();
        Box::pin(async move {
            calls.fetch_add(1, Ordering::SeqCst);
            assert_eq!(request.state().as_value(), "borrowed state");
            Ok(request
                .questions()
                .iter()
                .map(|(name, q)| (name.clone(), answer(q, 0.9)))
                .collect())
        })
    }));
    let request = Request::new(
        State::text("borrowed state"),
        IndexMap::from([("q".into(), boolean(0.0).into())]),
    )
    .unwrap();
    let future = ask(&provider, &request);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let result = futures_executor::block_on(future).unwrap();
    assert!(result["q"].decided());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
