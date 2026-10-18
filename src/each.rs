//! Bounded independent calls, preserving input order and per-item failures.
use crate::{
    Answer, AnswerValue, Error, ErrorKind, Provider, Question, Request, Result, State, ask,
};
use futures_util::{StreamExt, TryStreamExt, stream};
use indexmap::IndexMap;
use serde_json::Value;
use std::{collections::HashSet, num::NonZeroUsize};

/// Options shared by per-item filtering and reranking.
#[derive(Clone)]
pub struct BatchOptions {
    /// Maximum concurrently polled provider calls; defaults to eight.
    pub concurrency: NonZeroUsize,
    /// Shared background included in each item's independent request.
    pub context: Option<Value>,
    /// Explicit images available to each independent request.
    #[cfg(feature = "images")]
    pub assets: crate::image::Assets,
}
impl Default for BatchOptions {
    fn default() -> Self {
        Self {
            concurrency: NonZeroUsize::new(8).expect("eight is positive"),
            context: None,
            #[cfg(feature = "images")]
            assets: crate::image::Assets::default(),
        }
    }
}
impl std::fmt::Debug for BatchOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BatchOptions")
            .field("concurrency", &self.concurrency)
            .field("has_context", &self.context.is_some())
            .finish_non_exhaustive()
    }
}
/// One independently scored input. Identifiers are local metadata, never added to state.
#[derive(Clone)]
pub struct Item {
    id: String,
    state: State,
}
impl Item {
    /// Validate an identifier and retain a previously validated state.
    ///
    /// # Errors
    /// Returns `InvalidArgument` if `id` is empty. Batch functions separately
    /// check identifiers for uniqueness.
    pub fn new(id: impl Into<String>, state: State) -> Result<Self> {
        let id = id.into();
        validate_ids([id.as_str()])?;
        Ok(Self { id, state })
    }
    /// Local identifier.
    pub fn id(&self) -> &str {
        &self.id
    }
    /// State sent to the provider.
    pub fn state(&self) -> &State {
        &self.state
    }
}
impl std::fmt::Debug for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Item").finish_non_exhaustive()
    }
}
/// Evidence or a provider failure for one input; an error is never an abstention.
#[derive(Clone, Debug)]
pub enum ItemResult {
    /// A validated answer, possibly undecided.
    Answer(Answer<AnswerValue>),
    /// A provider, timeout, or malformed-response failure.
    Error(Error),
}
impl ItemResult {
    /// Answer evidence, if this item completed successfully.
    pub fn answer(&self) -> Option<&Answer<AnswerValue>> {
        match self {
            Self::Answer(a) => Some(a),
            Self::Error(_) => None,
        }
    }
    /// Failure evidence, if this item failed.
    pub fn error(&self) -> Option<&Error> {
        match self {
            Self::Answer(_) => None,
            Self::Error(e) => Some(e),
        }
    }
}

pub(crate) fn validate_ids<'a>(ids: impl IntoIterator<Item = &'a str>) -> Result<()> {
    let mut seen = HashSet::new();
    for id in ids {
        if id.is_empty() || !seen.insert(id) {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                "item identifiers must be nonempty and unique",
            ));
        }
    }
    Ok(())
}
/// Ask one question independently for every state with bounded concurrency.
///
/// Output order matches input order. Provider, timeout, and malformed-response
/// errors remain per-item results. Other errors abort and drop pending futures;
/// dropping a future does not establish remote cancellation. Empty inputs do no I/O.
/// This primitive sends its states unchanged; recipes apply `context` beforehand.
///
/// # Errors
/// Propagates local request/image validation errors and provider errors
/// other than `Provider`, `Timeout`, or `InvalidResponse`. Those three kinds
/// are retained as per-item [`ItemResult::Error`] values instead of failing
/// the batch. An outer error drops pending local futures.
pub async fn ask_each(
    provider: &dyn Provider,
    question: &Question,
    states: &[State],
    options: &BatchOptions,
) -> Result<Vec<ItemResult>> {
    let questions = IndexMap::from([("q".to_owned(), question.clone())]);
    let mut requests = Vec::with_capacity(states.len());
    // Validate all locally available configuration before polling any provider.
    for state in states {
        let request = Request::new(state.clone(), questions.clone())?;
        #[cfg(feature = "images")]
        let request = {
            options.assets.validate(state, &questions)?;
            request.with_assets(options.assets.clone())
        };
        requests.push(request);
    }
    let mut results: Vec<(usize, ItemResult)> = stream::iter(requests.into_iter().enumerate().map(
        |(position, request)| async move {
            let result = match ask(provider, &request).await {
                Ok(mut answers) => match answers.shift_remove("q") {
                    Some(answer) => ItemResult::Answer(answer),
                    None => ItemResult::Error(Error::new(
                        ErrorKind::InvalidResponse,
                        "provider omitted item answer",
                    )),
                },
                Err(error) => match error.kind() {
                    ErrorKind::Provider | ErrorKind::Timeout | ErrorKind::InvalidResponse => {
                        ItemResult::Error(error)
                    }
                    _ => return Err(error),
                },
            };
            Ok((position, result))
        },
    ))
    .buffer_unordered(options.concurrency.get())
    .try_collect()
    .await?;
    results.sort_unstable_by_key(|(position, _)| *position);
    Ok(results.into_iter().map(|(_, result)| result).collect())
}
