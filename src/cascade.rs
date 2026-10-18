//! Uncertainty-only escalation and optional per-call deadlines.
use crate::{Answers, Error, ErrorKind, Provider, ProviderFuture, Request, Result, ask};
use std::sync::Arc;

/// Ordered providers; only uncertain successful answers advance to the next hop.
pub struct Cascade {
    providers: Vec<Arc<dyn Provider>>,
    escalate_below: f64,
}
impl Cascade {
    /// Create a cascade of at least two providers with a finite `[0,1]` threshold.
    ///
    /// # Errors
    /// Returns `InvalidArgument` if fewer than two providers are supplied or
    /// `escalate_below` is nonfinite or outside `[0, 1]`.
    pub fn new(providers: Vec<Arc<dyn Provider>>, escalate_below: f64) -> Result<Self> {
        if providers.len() < 2
            || !escalate_below.is_finite()
            || !(0.0..=1.0).contains(&escalate_below)
        {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                "cascade needs two providers and a threshold from zero to one",
            ));
        }
        Ok(Self {
            providers,
            escalate_below,
        })
    }
}
impl Provider for Cascade {
    fn ask<'a>(&'a self, request: &'a Request) -> ProviderFuture<'a> {
        Box::pin(async move {
            let mut pending = request.questions().clone();
            let mut results = Answers::new();
            for provider in &self.providers {
                let hop = request.with_questions(pending.clone())?;
                let answers = ask(provider, &hop).await?;
                pending.retain(|name, _| {
                    answers.get(name).is_some_and(|answer| {
                        !answer.decided() || answer.confidence() < self.escalate_below
                    })
                });
                results.extend(answers);
                if pending.is_empty() {
                    break;
                }
            }
            crate::provider::validate_answers(request, results)
        })
    }
}
/// Bound a call with Tokio's timer. Requires a time-enabled Tokio runtime.
/// Cancellation is local; the server may still finish and bill the request.
#[cfg(feature = "tokio")]
pub struct TimeLimited<P> {
    provider: P,
    duration: std::time::Duration,
}
#[cfg(feature = "tokio")]
impl<P: Provider> TimeLimited<P> {
    /// Reject zero durations; timer starts when the future is polled.
    ///
    /// # Errors
    /// Returns `InvalidArgument` for a zero duration. Runtime/timer availability
    /// is checked when a call is polled, not during construction.
    pub fn new(provider: P, duration: std::time::Duration) -> Result<Self> {
        if duration.is_zero() {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                "timeout must be positive",
            ));
        }
        Ok(Self { provider, duration })
    }
}
#[cfg(feature = "tokio")]
impl<P: Provider> Provider for TimeLimited<P> {
    fn ask<'a>(&'a self, request: &'a Request) -> ProviderFuture<'a> {
        Box::pin(async move {
            crate::provider::deadline(self.duration, ask(&self.provider, request)).await?
        })
    }
}
