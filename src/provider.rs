//! Async provider contracts and the checked request/response boundary.

use crate::{Answer, Error, ErrorKind, IndexMap, Question, Result, State};
use std::{future::Future, pin::Pin, sync::Arc};

/// Questions in caller-supplied order, keyed by unique names.
pub type Questions = IndexMap<String, Question>;
/// Dynamic answers, returned in request order by [`ask`].
pub type Answers = IndexMap<String, Answer>;
/// Object-safe borrowed future. Work begins only when it is polled.
pub type ProviderFuture<'a> = Pin<Box<dyn Future<Output = Result<Answers>> + Send + 'a>>;

/// One state and its ordered questions, with optional request-owned assets.
#[derive(Clone)]
pub struct Request {
    state: State,
    questions: Questions,
    #[cfg(feature = "images")]
    assets: crate::image::Assets,
}
impl std::fmt::Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Request")
            .field("question_count", &self.questions.len())
            .finish_non_exhaustive()
    }
}
impl Request {
    /// Construct a nonempty request, rejecting empty names before any provider call.
    ///
    /// # Errors
    /// Returns `InvalidArgument` if questions are empty or any question name is
    /// empty. Image references are checked later by [`Request::validate`], so
    /// assets may be attached after construction.
    pub fn new(state: State, questions: Questions) -> Result<Self> {
        if questions.is_empty() || questions.keys().any(String::is_empty) {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                "a request needs nonempty, uniquely named questions",
            ));
        }
        Ok(Self {
            state,
            questions,
            #[cfg(feature = "images")]
            assets: Default::default(),
        })
    }
    /// Input state. Debug formatting never prints this value.
    pub fn state(&self) -> &State {
        &self.state
    }
    /// Ordered questions.
    pub fn questions(&self) -> &Questions {
        &self.questions
    }
    /// Replace questions while preserving state and attached assets.
    ///
    /// # Errors
    /// Returns `InvalidArgument` if the replacement map is empty or contains
    /// an empty question name.
    pub fn with_questions(&self, questions: Questions) -> Result<Self> {
        let request = Self::new(self.state.clone(), questions)?;
        #[cfg(feature = "images")]
        let request = request.with_assets(self.assets.clone());
        Ok(request)
    }
    /// Attach an explicit asset registry, validated before dispatch.
    #[cfg(feature = "images")]
    #[cfg_attr(docsrs, doc(cfg(feature = "images")))]
    pub fn with_assets(mut self, assets: crate::image::Assets) -> Self {
        self.assets = assets;
        self
    }
    /// Borrow assets without opening files or resolving URLs.
    #[cfg(feature = "images")]
    #[cfg_attr(docsrs, doc(cfg(feature = "images")))]
    pub fn assets(&self) -> &crate::image::Assets {
        &self.assets
    }
    /// Validate assets before calling a provider.
    ///
    /// # Errors
    /// With `images` enabled, returns `InvalidImage` for unknown image references
    /// or excessive repeated image use. With that feature disabled, this check
    /// always succeeds.
    pub fn validate(&self) -> Result<()> {
        #[cfg(feature = "images")]
        self.assets.validate(&self.state, &self.questions)?;
        Ok(())
    }
}

/// A provider must be lazy, cancellation-safe, and free of implicit retries.
/// Consumers should use [`ask`] to check response coverage and normalization.
pub trait Provider: Send + Sync {
    /// Answer the state and all its questions in one semantic invocation.
    ///
    /// # Errors
    /// The returned future may fail with provider, timeout, unsupported-capability,
    /// or input/response-validation errors. Implementations must preserve errors
    /// as errors; uncertainty belongs in a successful answer.
    fn ask<'a>(&'a self, request: &'a Request) -> ProviderFuture<'a>;
}
impl<P: Provider + ?Sized> Provider for Arc<P> {
    fn ask<'a>(&'a self, request: &'a Request) -> ProviderFuture<'a> {
        (**self).ask(request)
    }
}

/// Call a provider and verify input assets, response names, and answer semantics.
///
/// # Errors
/// Propagates request validation and provider errors. Returns `InvalidResponse`
/// for missing/extra answers or candidates incompatible with their questions.
pub async fn ask<P: Provider + ?Sized>(provider: &P, request: &Request) -> Result<Answers> {
    request.validate()?;
    validate_answers(request, provider.ask(request).await?)
}
/// Check response coverage and requested thresholds; return answers in request order.
///
/// # Errors
/// Returns `InvalidResponse` if answer names do not exactly match the request
/// or an answer fails normalization against its question.
pub fn validate_answers(request: &Request, answers: Answers) -> Result<Answers> {
    if answers.len() != request.questions.len()
        || answers
            .keys()
            .any(|name| !request.questions.contains_key(name))
    {
        return Err(Error::new(
            ErrorKind::InvalidResponse,
            "provider must answer exactly the requested question names",
        ));
    }
    request
        .questions
        .iter()
        .map(|(name, question)| {
            let answer = answers.get(name).ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidResponse,
                    "provider omitted a requested answer",
                )
            })?;
            Ok((name.clone(), answer.revalidate(question)?))
        })
        .collect()
}

/// An explicitly supplied async function implementing the provider contract.
pub struct FunctionProvider<F> {
    function: F,
}
impl<F> FunctionProvider<F>
where
    F: for<'a> Fn(&'a Request) -> ProviderFuture<'a> + Send + Sync,
{
    /// Wrap a local inference adapter or scripted function.
    pub fn new(function: F) -> Self {
        Self { function }
    }
}
impl<F> Provider for FunctionProvider<F>
where
    F: for<'a> Fn(&'a Request) -> ProviderFuture<'a> + Send + Sync,
{
    fn ask<'a>(&'a self, request: &'a Request) -> ProviderFuture<'a> {
        Box::pin(async move { (self.function)(request).await })
    }
}

// Tokio has no public timer-enabled query. Its timer constructor panics when
// a runtime exists without a timer driver. Catch only construction, before the
// provider future is polled; caller configuration errors remain typed errors.
#[cfg(feature = "tokio")]
pub(crate) async fn deadline<F: Future>(
    duration: std::time::Duration,
    future: F,
) -> Result<F::Output> {
    if tokio::runtime::Handle::try_current().is_err() {
        return Err(Error::new(
            ErrorKind::Runtime,
            "deadline requires a time-enabled Tokio runtime",
        ));
    }
    let timer = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokio::time::timeout(duration, future)
    }))
    .map_err(|_| Error::new(ErrorKind::Runtime, "Tokio timer driver is not enabled"))?;
    timer
        .await
        .map_err(|_| Error::new(ErrorKind::Timeout, "provider call exceeded its deadline"))
}
