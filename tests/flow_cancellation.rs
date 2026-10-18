//! Local flow cancellation retains attempts and cannot become successful completion.
use futures_executor::block_on;
use futures_util::FutureExt;
use semantic_operators::flow::{FlowContext, FlowExit};
use semantic_operators::*;
use std::future::Future;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::task::Context;

struct FirstReadyThenPending {
    entered: AtomicUsize,
    dropped: Arc<AtomicUsize>,
}
struct PendingGuard(Arc<AtomicUsize>);
impl Drop for PendingGuard {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
impl Provider for FirstReadyThenPending {
    fn ask<'a>(&'a self, request: &'a Request) -> ProviderFuture<'a> {
        Box::pin(async move {
            if self.entered.fetch_add(1, Ordering::SeqCst) > 0 {
                let _guard = PendingGuard(self.dropped.clone());
                return std::future::pending().await;
            }
            request
                .questions()
                .iter()
                .map(|(name, question)| {
                    Ok((
                        name.clone(),
                        make_answer(
                            question,
                            AnswerValue::Boolean(true),
                            IndexMap::from([("true".into(), 0.9), ("false".into(), 0.1)]),
                            None,
                        )?,
                    ))
                })
                .collect()
        })
    }
}
fn provider() -> FirstReadyThenPending {
    FirstReadyThenPending {
        entered: AtomicUsize::new(0),
        dropped: Arc::new(AtomicUsize::new(0)),
    }
}
fn request() -> Request {
    Request::new(
        State::text("synthetic"),
        IndexMap::from([(
            "q".into(),
            Boolean::new("Synthetic question").unwrap().into(),
        )]),
    )
    .unwrap()
}

#[test]
fn unpolled_flow_future_does_not_record_or_latch_failure() {
    let provider = provider();
    let request = request();
    let mut flow = FlowContext::new(&provider);
    drop(flow.ask_request(&request));
    assert_eq!(provider.entered.load(Ordering::SeqCst), 0);
    block_on(flow.ask_request(&request)).unwrap();
    let outcome = flow.finish(Ok(())).unwrap();
    assert_eq!(outcome.calls(), 1);
    assert!(outcome.trace[0].result.is_ok());
}

#[test]
fn cancelled_polled_flow_preserves_prior_trace_and_latches_local_failure() {
    let provider = provider();
    let request = request();
    let operator = Operator::new("q", Boolean::new("Synthetic question").unwrap()).unwrap();
    let operators = [operator.erased()];
    let mut flow = FlowContext::new(&provider);
    block_on(flow.ask_request(&request)).unwrap();
    let mut pending = Box::pin(flow.ask(request.state(), &operators));
    let mut context = Context::from_waker(futures_util::task::noop_waker_ref());
    assert!(pending.as_mut().poll(&mut context).is_pending());
    assert_eq!(provider.entered.load(Ordering::SeqCst), 2);
    drop(pending);
    assert_eq!(provider.dropped.load(Ordering::SeqCst), 1);

    let error = flow
        .ask_request(&request)
        .now_or_never()
        .expect("latched cancellation must return immediately")
        .unwrap_err();
    assert!(matches!(error, FlowExit::Error(ref e) if e.kind() == ErrorKind::Runtime));
    assert_eq!(provider.entered.load(Ordering::SeqCst), 2);
    let failure = flow.finish(Ok("caught cancellation")).unwrap_err();
    assert_eq!(failure.error.kind(), ErrorKind::Runtime);
    assert_eq!(failure.trace.len(), 2);
    assert!(failure.trace[0].result.is_ok());
    assert_eq!(failure.trace[1].request.state(), request.state());
    assert_eq!(
        failure.trace[1].result.as_ref().unwrap_err(),
        &failure.error
    );
    assert_eq!(
        failure.error.message(),
        "flow request ended before local completion"
    );
}
