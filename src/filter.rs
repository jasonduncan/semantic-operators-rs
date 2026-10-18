//! Boolean filtering with one recorded outcome per input.
use crate::each::{BatchOptions, Item, ItemResult, ask_each, validate_ids};
use crate::{Answer, AnswerValue, Boolean, Error, Result, State};
use serde_json::{Map, Value};
use std::collections::BTreeSet;

/// Exhaustive filtering outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterOutcome {
    /// Decided true.
    Match,
    /// Decided false.
    No,
    /// Undecided because of a tie or confidence threshold.
    Unsure,
    /// Provider, timeout, or response failure.
    Failed,
}
/// One filtering result, retaining its input position and evidence.
#[derive(Clone, Debug)]
pub struct Verdict {
    id: String,
    position: usize,
    outcome: FilterOutcome,
    result: ItemResult,
}
impl Verdict {
    /// Local input identifier.
    pub fn id(&self) -> &str {
        &self.id
    }
    /// Zero-based position in the input sequence.
    pub fn position(&self) -> usize {
        self.position
    }
    /// Match, no, unsure, or failed.
    pub fn outcome(&self) -> FilterOutcome {
        self.outcome
    }
    /// Validated answer, including evidence for an unsure result.
    pub fn answer(&self) -> Option<&Answer<AnswerValue>> {
        self.result.answer()
    }
    /// Provider failure, if any.
    pub fn error(&self) -> Option<&Error> {
        self.result.error()
    }
}
/// Complete filtering report, in input order.
#[derive(Clone, Debug)]
pub struct Filtered {
    question: Boolean,
    verdicts: Vec<Verdict>,
}
impl Filtered {
    /// Question used for every item.
    pub fn question(&self) -> &Boolean {
        &self.question
    }
    /// Exactly one outcome per input, in input order.
    pub fn verdicts(&self) -> &[Verdict] {
        &self.verdicts
    }
    /// Decided true results, in input order.
    pub fn matched(&self) -> Vec<&Verdict> {
        self.with_outcome(FilterOutcome::Match)
    }
    /// Decided false results, in input order.
    pub fn rejected(&self) -> Vec<&Verdict> {
        self.with_outcome(FilterOutcome::No)
    }
    /// Undecided results, in input order.
    pub fn unsure(&self) -> Vec<&Verdict> {
        self.with_outcome(FilterOutcome::Unsure)
    }
    /// Failed calls, in input order.
    pub fn failed(&self) -> Vec<&Verdict> {
        self.with_outcome(FilterOutcome::Failed)
    }
    /// Nonempty model identities actually reported, including unsure results.
    pub fn models(&self) -> BTreeSet<&str> {
        self.verdicts
            .iter()
            .filter_map(|v| v.answer())
            .filter_map(|a| a.call())
            .filter_map(|c| c.model.as_deref())
            .filter(|m| !m.is_empty())
            .collect()
    }
    fn with_outcome(&self, outcome: FilterOutcome) -> Vec<&Verdict> {
        self.verdicts
            .iter()
            .filter(|v| v.outcome == outcome)
            .collect()
    }
}
/// Independently classify every item; IDs are never added to provider state.
///
/// With a non-null context, each state becomes `{"context": ..., "item": ...}`.
/// Duplicate IDs fail before I/O. Provider failures affect only their own items;
/// invalid configuration and unsupported capabilities abort the batch.
///
/// # Errors
/// Returns `InvalidArgument` for duplicate item identifiers and propagates
/// local validation or unsupported-capability errors from [`ask_each`].
/// Provider, timeout, and malformed-response failures become `Failed`
/// verdicts for their individual items.
pub async fn filter_items(
    provider: &dyn crate::Provider,
    question: &Boolean,
    items: &[Item],
    options: &BatchOptions,
) -> Result<Filtered> {
    validate_ids(items.iter().map(Item::id))?;
    let mut states = Vec::with_capacity(items.len());
    for item in items {
        let state = if let Some(context) = options.context.as_ref().filter(|v| !v.is_null()) {
            let mut value = Map::new();
            value.insert("context".into(), context.clone());
            value.insert("item".into(), item.state().as_value().clone());
            State::try_from(Value::Object(value))?
        } else {
            item.state().clone()
        };
        states.push(state);
    }
    let results = ask_each(provider, &question.clone().into(), &states, options).await?;
    let verdicts = items
        .iter()
        .zip(results)
        .enumerate()
        .map(|(position, (item, result))| {
            let outcome = match &result {
                ItemResult::Error(_) => FilterOutcome::Failed,
                ItemResult::Answer(answer) => match answer.value() {
                    Some(AnswerValue::Boolean(true)) => FilterOutcome::Match,
                    Some(AnswerValue::Boolean(false)) => FilterOutcome::No,
                    _ => FilterOutcome::Unsure,
                },
            };
            Verdict {
                id: item.id().to_owned(),
                position,
                outcome,
                result,
            }
        })
        .collect();
    Ok(Filtered {
        question: question.clone(),
        verdicts,
    })
}
