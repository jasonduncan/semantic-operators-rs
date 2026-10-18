//! Independent relevance scoring and stable ranking without synthetic scores.
use crate::each::{BatchOptions, ItemResult, ask_each, validate_ids};
use crate::{Answer, AnswerValue, Error, ErrorKind, Provider, Result, Score, State};
use indexmap::IndexMap;
use serde_json::{Map, Value};
use std::collections::BTreeSet;

/// A caller-retrieved document and its local identifier.
#[derive(Clone)]
pub struct Candidate {
    id: String,
    document: Value,
}
impl Candidate {
    /// Validate a local identifier; documents may be any JSON value.
    ///
    /// # Errors
    /// Returns `InvalidArgument` if `id` is empty. [`rerank`] separately checks
    /// identifiers for uniqueness.
    pub fn new(id: impl Into<String>, document: Value) -> Result<Self> {
        let id = id.into();
        validate_ids([id.as_str()])?;
        Ok(Self { id, document })
    }
    /// Local identifier, never added to the provider's state.
    pub fn id(&self) -> &str {
        &self.id
    }
    /// Caller-provided document value.
    pub fn document(&self) -> &Value {
        &self.document
    }
}
impl std::fmt::Debug for Candidate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Candidate").finish_non_exhaustive()
    }
}
/// One decided candidate and its ranking key.
#[derive(Clone, Debug)]
pub struct Ranked {
    id: String,
    answer: Answer<AnswerValue>,
    score: f64,
    position: usize,
}
impl Ranked {
    /// Local candidate identifier.
    pub fn id(&self) -> &str {
        &self.id
    }
    /// Original normalized score evidence and call provenance.
    pub fn answer(&self) -> &Answer<AnswerValue> {
        &self.answer
    }
    /// Expected rubric level, or expected custom weight after reweighting.
    pub fn score(&self) -> f64 {
        self.score
    }
    /// Retrieval position used to preserve exact ties.
    pub fn position(&self) -> usize {
        self.position
    }
}
/// Sorted decided candidates and every unscored candidate's evidence.
#[derive(Clone, Debug)]
pub struct Ranking {
    question: Score,
    ranked: Vec<Ranked>,
    unscored: IndexMap<String, ItemResult>,
}
impl Ranking {
    /// Original scoring rubric.
    pub fn question(&self) -> &Score {
        &self.question
    }
    /// Candidates sorted by descending score, with exact ties in retrieval order.
    pub fn ranked(&self) -> &[Ranked] {
        &self.ranked
    }
    /// Undecided answers and provider failures, in retrieval order.
    pub fn unscored(&self) -> &IndexMap<String, ItemResult> {
        &self.unscored
    }
    /// Whether every candidate has a decided score.
    pub fn complete(&self) -> bool {
        self.unscored.is_empty()
    }
    /// Nonempty model identities actually reported, including undecided answers.
    pub fn models(&self) -> BTreeSet<&str> {
        self.ranked
            .iter()
            .map(|r| &r.answer)
            .chain(self.unscored.values().filter_map(ItemResult::answer))
            .filter_map(|a| a.call())
            .filter_map(|c| c.model.as_deref())
            .filter(|m| !m.is_empty())
            .collect()
    }
    /// Return at most `k` candidates, checking completeness and reported-model consistency.
    ///
    /// `allow_partial` permits unscored candidates; it never permits mixed models.
    /// Unknown model metadata stays unknown rather than claiming one common model.
    ///
    /// # Errors
    /// Returns `InvalidArgument` if candidates are unscored and `allow_partial`
    /// is false, or if answers report more than one nonempty model identity.
    /// The mixed-model check applies even when partial results are allowed.
    pub fn top(&self, k: usize, allow_partial: bool) -> Result<&[Ranked]> {
        if !self.complete() && !allow_partial {
            return Err(invalid(
                "ranking has unscored candidates; explicitly allow partial results",
            ));
        }
        if self.models().len() > 1 {
            return Err(invalid(
                "ranking contains answers from different reported models",
            ));
        }
        Ok(&self.ranked[..k.min(self.ranked.len())])
    }
    /// Re-sort decided answers using finite strictly increasing rubric weights.
    ///
    /// Uses normalized `sum(weight * probability) / sum(probability)` and retains
    /// the original answers and unscored entries. No provider call occurs.
    ///
    /// # Errors
    /// Returns `InvalidArgument` if weights do not match the rubric length, are
    /// nonfinite or not strictly increasing, or weighted arithmetic is nonfinite.
    pub fn reweighted(&self, weights: &[f64]) -> Result<Self> {
        if weights.len() != self.question.levels().len()
            || weights.iter().any(|w| !w.is_finite())
            || weights.windows(2).any(|w| w[0] >= w[1])
        {
            return Err(invalid(
                "need one finite strictly increasing weight per rubric level",
            ));
        }
        let mut ranked = Vec::with_capacity(self.ranked.len());
        for old in &self.ranked {
            let probabilities: Vec<_> = self
                .question
                .levels()
                .iter()
                .map(|label| old.answer.probabilities()[label])
                .collect();
            let total = accurate_sum(probabilities.iter().copied())?;
            let weighted = accurate_sum(weights.iter().zip(&probabilities).map(|(w, p)| w * p))?;
            let score = weighted / total;
            if !score.is_finite() {
                return Err(invalid("reweighted score is not finite"));
            }
            ranked.push(Ranked {
                id: old.id.clone(),
                answer: old.answer.clone(),
                score,
                position: old.position,
            });
        }
        sort(&mut ranked);
        Ok(Self {
            question: self.question.clone(),
            ranked,
            unscored: self.unscored.clone(),
        })
    }
}
/// Score each candidate independently against a nonblank query.
///
/// Each request contains `query`, `document`, and optional non-null `context`.
/// Local IDs and retrieval positions are never added. Failures and abstentions
/// remain unscored, while exact score ties preserve retrieval order.
///
/// # Errors
/// Returns `InvalidArgument` for a blank query or duplicate candidate IDs,
/// and propagates local validation or unsupported-capability errors. Provider,
/// timeout, and malformed-response failures are retained in `unscored` rather
/// than converted into scores.
pub async fn rerank(
    provider: &dyn Provider,
    question: &Score,
    query: &str,
    candidates: &[Candidate],
    options: &BatchOptions,
) -> Result<Ranking> {
    if crate::question::is_blank(query) {
        return Err(invalid("query must be a non-empty string"));
    }
    validate_ids(candidates.iter().map(Candidate::id))?;
    let mut states = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let mut value = Map::new();
        value.insert("query".into(), Value::String(query.into()));
        value.insert("document".into(), candidate.document.clone());
        if let Some(context) = options.context.as_ref().filter(|v| !v.is_null()) {
            value.insert("context".into(), context.clone());
        }
        states.push(State::try_from(Value::Object(value))?);
    }
    let results = ask_each(provider, &question.clone().into(), &states, options).await?;
    let mut ranked = Vec::new();
    let mut unscored = IndexMap::new();
    for (position, (candidate, result)) in candidates.iter().zip(results).enumerate() {
        match result {
            ItemResult::Answer(answer) if answer.decided() => {
                let Some(AnswerValue::Score(score)) = answer.value() else {
                    return Err(Error::new(
                        ErrorKind::InvalidResponse,
                        "rerank answer is not a score",
                    ));
                };
                ranked.push(Ranked {
                    id: candidate.id.clone(),
                    score: *score,
                    answer,
                    position,
                });
            }
            other => {
                unscored.insert(candidate.id.clone(), other);
            }
        }
    }
    sort(&mut ranked);
    Ok(Ranking {
        question: question.clone(),
        ranked,
        unscored,
    })
}
/// Re-rank existing answers with no additional calls.
///
/// # Errors
/// Propagates the weight-validation and arithmetic errors from
/// [`Ranking::reweighted`].
pub fn reweighted(ranking: &Ranking, weights: &[f64]) -> Result<Ranking> {
    ranking.reweighted(weights)
}
fn sort(items: &mut [Ranked]) {
    items.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.position.cmp(&b.position))
    });
}
fn invalid(message: &str) -> Error {
    Error::new(ErrorKind::InvalidArgument, message)
}
// Error-free partial summation plus half-even correction, matching math.fsum.
fn accurate_sum(values: impl IntoIterator<Item = f64>) -> Result<f64> {
    let mut partials: Vec<f64> = Vec::new();
    for mut x in values {
        if !x.is_finite() {
            return Err(invalid("reweighted sum overflow"));
        }
        let mut i = 0;
        for j in 0..partials.len() {
            let mut y = partials[j];
            if x.abs() < y.abs() {
                std::mem::swap(&mut x, &mut y);
            }
            let hi = x + y;
            if !hi.is_finite() {
                return Err(invalid("reweighted sum overflow"));
            }
            let lo = y - (hi - x);
            if lo != 0.0 {
                partials[i] = lo;
                i += 1;
            }
            x = hi;
        }
        partials.truncate(i);
        partials.push(x);
    }
    let mut hi = partials.pop().unwrap_or(0.0);
    let mut lo = 0.0;
    while let Some(y) = partials.pop() {
        let x = hi;
        hi = x + y;
        lo = y - (hi - x);
        if lo != 0.0 {
            break;
        }
    }
    if let Some(last) = partials.last() {
        if (lo < 0.0 && *last < 0.0) || (lo > 0.0 && *last > 0.0) {
            let y = lo * 2.0;
            let x = hi + y;
            if y == x - hi {
                hi = x;
            }
        }
    }
    Ok(hi)
}
