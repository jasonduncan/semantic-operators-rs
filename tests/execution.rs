//! Provider, operator, cascade, flow, cancellation, and blocking integration tests.
use futures_executor::block_on;
use semantic_operators::flow::{FlowContext, FlowExit};
use semantic_operators::*;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Clone)]
struct Scripted {
    calls: Arc<Mutex<Vec<Vec<String>>>>,
    probabilities: IndexMap<String, f64>,
    model: String,
    fail: Option<ErrorKind>,
}
impl Scripted {
    fn new(model: &str, p: &[(&str, f64)]) -> Self {
        Self {
            calls: Default::default(),
            probabilities: p.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            model: model.into(),
            fail: None,
        }
    }
}
impl Provider for Scripted {
    fn ask<'a>(&'a self, request: &'a Request) -> ProviderFuture<'a> {
        Box::pin(async move {
            self.calls
                .lock()
                .unwrap()
                .push(request.questions().keys().cloned().collect());
            if let Some(kind) = self.fail {
                return Err(Error::new(kind, "scripted failure"));
            }
            let call = Arc::new(Call {
                provider: "scripted".into(),
                model: Some(self.model.clone()),
                input_tokens: Some(7),
                output_tokens: Some(2),
            });
            request
                .questions()
                .iter()
                .map(|(name, q)| {
                    let p = *self.probabilities.get(name).unwrap_or(&0.9);
                    let (v, probs) = match q {
                        Question::Boolean(_) => (
                            AnswerValue::Boolean(p > 0.5),
                            IndexMap::from([("true".into(), p), ("false".into(), complement(p)?)]),
                        ),
                        Question::Score(s) => {
                            let mut probs: Probabilities =
                                s.levels().iter().map(|s| (s.clone(), 0.0)).collect();
                            *probs.get_index_mut(0).unwrap().1 = 1.0;
                            (AnswerValue::Score(0.0), probs)
                        }
                        _ => {
                            return Err(Error::new(
                                ErrorKind::Unsupported,
                                "unsupported synthetic question",
                            ));
                        }
                    };
                    Ok((
                        name.clone(),
                        make_answer(q, v, probs, None)?.with_call(call.clone()),
                    ))
                })
                .collect()
        })
    }
}
fn request(names: &[&str]) -> Request {
    Request::new(
        State::text("private synthetic input"),
        names
            .iter()
            .map(|n| {
                (
                    n.to_string(),
                    Boolean::new("Synthetic question").unwrap().into(),
                )
            })
            .collect(),
    )
    .unwrap()
}

#[test]
fn future_is_lazy_and_dyn_providers_batch_once() {
    let provider = Scripted::new("m", &[]);
    let req = request(&["a", "b"]);
    let dynamic: &dyn Provider = &provider;
    let future = ask(dynamic, &req);
    assert!(provider.calls.lock().unwrap().is_empty());
    let answers = block_on(future).unwrap();
    assert_eq!(
        provider.calls.lock().unwrap().as_slice(),
        &[vec!["a".to_owned(), "b".to_owned()]]
    );
    assert!(Arc::ptr_eq(
        answers["a"].call().unwrap(),
        answers["b"].call().unwrap()
    ));
    assert!(!format!("{req:?}").contains("private synthetic input"));
}

#[test]
fn typed_apply_keeps_false_and_zero_and_rejects_duplicates() {
    let provider = Scripted::new("m", &[("a", 0.1)]);
    let boolean = Operator::new("a", Boolean::new("a").unwrap()).unwrap();
    let score = Operator::new(
        "b",
        Score::new("b", vec!["low".into(), "high".into()]).unwrap(),
    )
    .unwrap();
    let req = request(&["unused"]);
    let answers = block_on(apply(&provider, &req, &[boolean.erased(), score.erased()])).unwrap();
    assert_eq!(boolean.answer(&answers).unwrap().value(), Some(&false));
    assert_eq!(score.answer(&answers).unwrap().value(), Some(&0.0));
    assert!(
        block_on(apply(
            &provider,
            &req,
            &[boolean.erased(), boolean.erased()]
        ))
        .is_err()
    );
    assert!(block_on(apply(&provider, &req, &[])).is_err());
    assert_eq!(provider.calls.lock().unwrap().len(), 1);
}

#[test]
fn response_coverage_and_request_thresholds_are_checked() {
    let req = request(&["q"]);
    let provider = Scripted::new("m", &[("q", 0.7)]);
    let mut answers = block_on(ask(&provider, &req)).unwrap();
    let strict = Request::new(
        req.state().clone(),
        IndexMap::from([(
            "q".into(),
            Boolean::new("q")
                .unwrap()
                .with_min_confidence(0.9)
                .unwrap()
                .into(),
        )]),
    )
    .unwrap();
    assert!(
        !semantic_operators::provider::validate_answers(&strict, answers.clone()).unwrap()["q"]
            .decided()
    );
    answers.insert("extra".into(), answers["q"].clone());
    assert_eq!(
        semantic_operators::provider::validate_answers(&req, answers)
            .unwrap_err()
            .kind(),
        ErrorKind::InvalidResponse
    );
    assert!(semantic_operators::provider::validate_answers(&req, Answers::new()).is_err());
}

#[test]
fn cascade_reasks_only_uncertain_subset_and_preserves_call_identity() {
    let fast = Scripted::new("fast", &[("sure", 0.99), ("tie", 0.5), ("weak", 0.7)]);
    let strong = Scripted::new("strong", &[("tie", 0.95), ("weak", 0.96)]);
    let cascade =
        Cascade::new(vec![Arc::new(fast.clone()), Arc::new(strong.clone())], 0.8).unwrap();
    let req = request(&["sure", "tie", "weak"]);
    let answers = block_on(ask(&cascade, &req)).unwrap();
    assert_eq!(
        strong.calls.lock().unwrap().as_slice(),
        &[vec!["tie".to_owned(), "weak".to_owned()]]
    );
    assert_eq!(
        answers.keys().map(String::as_str).collect::<Vec<_>>(),
        vec!["sure", "tie", "weak"]
    );
    assert_eq!(
        answers["sure"].call().unwrap().model.as_deref(),
        Some("fast")
    );
    assert_eq!(
        answers["tie"].call().unwrap().model.as_deref(),
        Some("strong")
    );
}

#[test]
fn cascade_never_hides_failure_and_last_hop_may_abstain() {
    let mut bad = Scripted::new("bad", &[]);
    bad.fail = Some(ErrorKind::Provider);
    let next = Scripted::new("next", &[]);
    let cascade = Cascade::new(vec![Arc::new(bad), Arc::new(next.clone())], 0.9).unwrap();
    assert_eq!(
        block_on(ask(&cascade, &request(&["q"])))
            .unwrap_err()
            .kind(),
        ErrorKind::Provider
    );
    assert!(next.calls.lock().unwrap().is_empty());
    let a = Arc::new(Scripted::new("tie", &[("q", 0.5)]));
    let cascade = Cascade::new(vec![a.clone(), a], 0.9).unwrap();
    assert!(!block_on(ask(&cascade, &request(&["q"]))).unwrap()["q"].decided());
    assert!(Cascade::new(vec![], 0.5).is_err());
    assert!(Cascade::new(vec![Arc::new(next.clone()), Arc::new(next)], f64::NAN).is_err());
}

#[test]
fn flow_unused_uncertainty_does_not_stop_and_read_uncertainty_does() {
    let provider = Scripted::new("m", &[("uncertain", 0.5), ("certain", 0.99)]);
    let uncertain = Operator::new("uncertain", Boolean::new("q").unwrap()).unwrap();
    let certain = Operator::new("certain", Boolean::new("q").unwrap()).unwrap();
    let state = State::text("synthetic");
    let mut ctx = FlowContext::new(&provider);
    let result: Result<bool, FlowExit> = block_on(async {
        let a = ctx
            .ask(&state, &[uncertain.erased(), certain.erased()])
            .await?;
        a.require(&certain)
    });
    let outcome = ctx.finish(result).unwrap();
    assert_eq!(outcome.value, Some(true));
    assert_eq!(outcome.calls(), 1);
    let mut ctx = FlowContext::new(&provider);
    let result: Result<bool, FlowExit> = block_on(async {
        ctx.ask(&state, &[uncertain.erased()])
            .await?
            .require(&uncertain)
    });
    let outcome = ctx.finish(result).unwrap();
    assert!(!outcome.decided());
    assert_eq!(outcome.stopped_at.as_deref(), Some("uncertain"));
    assert!(!outcome.stopped_answer.unwrap().decided());
}

#[test]
fn handled_uncertainty_and_dependent_flow_calls() {
    let provider = Scripted::new("m", &[("q", 0.5)]);
    let q = Operator::new("q", Boolean::new("q").unwrap()).unwrap();
    let state = State::text("synthetic");
    let mut ctx = FlowContext::new(&provider);
    let result: Result<String, FlowExit> = block_on(async {
        let answers = ctx.ask(&state, &[q.erased()]).await?;
        if answers.peek(&q)?.value().is_none() {
            let other =
                Operator::new("other", Boolean::new("different question").unwrap()).unwrap();
            let second = ctx
                .ask(&State::text("dependent state"), &[other.erased()])
                .await?;
            if second.require(&other)? {
                return Ok("human".into());
            }
        }
        Ok("ordinary".into())
    });
    let outcome = ctx.finish(result).unwrap();
    assert_eq!(outcome.value.as_deref(), Some("human"));
    assert_eq!(outcome.calls(), 2);
}

#[test]
fn flow_cannot_swallow_provider_failure_and_trace_is_retained() {
    let mut provider = Scripted::new("m", &[]);
    provider.fail = Some(ErrorKind::Provider);
    let mut ctx = FlowContext::new(&provider);
    let req = request(&["q"]);
    assert!(block_on(ctx.ask_request(&req)).is_err());
    assert!(block_on(ctx.ask_request(&req)).is_err());
    let fail = ctx.finish(Ok("pretend-success")).unwrap_err();
    assert_eq!(fail.error.kind(), ErrorKind::Provider);
    assert_eq!(fail.trace.len(), 1);
    assert_eq!(provider.calls.lock().unwrap().len(), 1);
}

struct Never {
    entered: Arc<AtomicUsize>,
    dropped: Arc<AtomicUsize>,
}
struct Guard(Arc<AtomicUsize>);
impl Drop for Guard {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
impl Provider for Never {
    fn ask<'a>(&'a self, _: &'a Request) -> ProviderFuture<'a> {
        Box::pin(async move {
            self.entered.fetch_add(1, Ordering::SeqCst);
            let _guard = Guard(self.dropped.clone());
            std::future::pending().await
        })
    }
}
#[test]
fn dropping_future_drops_inflight_work_without_background_task() {
    let entered = Arc::new(AtomicUsize::new(0));
    let dropped = Arc::new(AtomicUsize::new(0));
    let provider = Never {
        entered: entered.clone(),
        dropped: dropped.clone(),
    };
    let req = request(&["q"]);
    let mut future = Box::pin(ask(&provider, &req));
    block_on(async {
        assert!(futures_util::poll!(&mut future).is_pending());
    });
    assert_eq!(entered.load(Ordering::SeqCst), 1);
    assert_eq!(dropped.load(Ordering::SeqCst), 0);
    drop(future);
    assert_eq!(dropped.load(Ordering::SeqCst), 1);
}

#[cfg(feature = "tokio")]
#[tokio::test(start_paused = true)]
async fn deadline_cancels_pending_work_and_does_not_mask_provider_error() {
    let entered = Arc::new(AtomicUsize::new(0));
    let dropped = Arc::new(AtomicUsize::new(0));
    let provider = TimeLimited::new(
        Never {
            entered: entered.clone(),
            dropped: dropped.clone(),
        },
        std::time::Duration::from_millis(10),
    )
    .unwrap();
    assert_eq!(
        ask(&provider, &request(&["q"])).await.unwrap_err().kind(),
        ErrorKind::Timeout
    );
    assert_eq!(dropped.load(Ordering::SeqCst), 1);
    let mut broken = Scripted::new("m", &[]);
    broken.fail = Some(ErrorKind::Provider);
    let timed = TimeLimited::new(broken, std::time::Duration::from_secs(1)).unwrap();
    assert_eq!(
        ask(&timed, &request(&["q"])).await.unwrap_err().kind(),
        ErrorKind::Provider
    );
}

#[cfg(feature = "tokio")]
#[tokio::test(start_paused = true)]
async fn whole_cascade_deadline_differs_from_per_hop_deadline() {
    use std::time::Duration;
    struct Delayed(Scripted);
    impl Provider for Delayed {
        fn ask<'a>(&'a self, r: &'a Request) -> ProviderFuture<'a> {
            Box::pin(async move {
                tokio::time::sleep(Duration::from_millis(7)).await;
                ask(&self.0, r).await
            })
        }
    }
    let make = || Arc::new(Delayed(Scripted::new("m", &[("q", 0.5)]))) as Arc<dyn Provider>;
    let whole = TimeLimited::new(
        Cascade::new(vec![make(), make()], 0.8).unwrap(),
        Duration::from_millis(10),
    )
    .unwrap();
    assert_eq!(
        ask(&whole, &request(&["q"])).await.unwrap_err().kind(),
        ErrorKind::Timeout
    );
    let hop = || {
        Arc::new(TimeLimited::new(make(), Duration::from_millis(10)).unwrap()) as Arc<dyn Provider>
    };
    let chain = Cascade::new(vec![hop(), hop()], 0.8).unwrap();
    assert!(!ask(&chain, &request(&["q"])).await.unwrap()["q"].decided());
}

#[cfg(feature = "blocking")]
#[test]
fn blocking_facade_runs_sync_and_rejects_nested_tokio() {
    let provider = semantic_operators::blocking::Blocking::new(Scripted::new("m", &[])).unwrap();
    let req = request(&["q"]);
    assert!(provider.ask(&req).unwrap()["q"].decided());
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        assert_eq!(provider.ask(&req).unwrap_err().kind(), ErrorKind::Runtime);
        drop(provider);
    });
}
