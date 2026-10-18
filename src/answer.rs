//! Central answer normalization and typed projections.
use crate::{Error, ErrorKind, Question, Region};
use indexmap::IndexMap;
use serde_json::Value;
use std::sync::Arc;
/// Ordered outcome probabilities.
pub type Probabilities = IndexMap<String, f64>;
/// Metadata actually reported by one provider call.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Call {
    /// Provider identity.
    pub provider: String,
    /// Reported model identity, when known.
    pub model: Option<String>,
    /// Reported input token usage.
    pub input_tokens: Option<u64>,
    /// Reported output token usage.
    pub output_tokens: Option<u64>,
}
/// A heterogeneous decided value.
#[derive(Clone, Debug, PartialEq)]
pub enum AnswerValue {
    /// Boolean outcome.
    Boolean(bool),
    /// Chosen option name.
    Choice(String),
    /// Expected score.
    Score(f64),
    /// Image region.
    Box(Region),
}
/// Conversion supported by a typed answer.
pub trait AnswerType: Clone + Send + Sync + 'static {
    /// Check and copy a heterogeneous value.
    ///
    /// # Errors
    /// Returns an error when the dynamic value cannot be represented by this
    /// output type. Built-in conversions report `InvalidResponse` for a variant mismatch.
    fn from_answer_value(value: &AnswerValue) -> Result<Self, Error>;
}
macro_rules! typed {
    ($ty:ty,$variant:ident) => {
        impl AnswerType for $ty {
            fn from_answer_value(v: &AnswerValue) -> Result<Self, Error> {
                match v {
                    AnswerValue::$variant(x) => Ok(x.clone()),
                    _ => Err(invalid("answer type does not match the requested output")),
                }
            }
        }
    };
}
typed!(bool, Boolean);
typed!(String, Choice);
typed!(f64, Score);
typed!(Region, Box);
impl AnswerType for AnswerValue {
    fn from_answer_value(v: &AnswerValue) -> Result<Self, Error> {
        Ok(v.clone())
    }
}
/// A normalized answer. Undecided answers retain evidence and call metadata.
#[derive(Clone, PartialEq)]
pub struct Answer<T = AnswerValue> {
    value: Option<T>,
    candidate: AnswerValue,
    probabilities: Probabilities,
    confidence: f64,
    call: Option<Arc<Call>>,
    raw: Option<Value>,
}
impl<T> std::fmt::Debug for Answer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Answer")
            .field("decided", &self.value.is_some())
            .field("confidence", &self.confidence)
            .field("call", &self.call)
            .finish_non_exhaustive()
    }
}
impl<T> Answer<T> {
    /// Decided value, or None for abstention.
    pub fn value(&self) -> Option<&T> {
        self.value.as_ref()
    }
    /// Whether this answer is decided. False and zero are decided values.
    pub fn decided(&self) -> bool {
        self.value.is_some()
    }
    /// Reported decision confidence.
    pub fn confidence(&self) -> f64 {
        self.confidence
    }
    /// Ordered probabilities, including for abstentions.
    pub fn probabilities(&self) -> &Probabilities {
        &self.probabilities
    }
    /// Metadata of the actual call, if reported.
    pub fn call(&self) -> Option<&Arc<Call>> {
        self.call.as_ref()
    }
    /// Optional private provider response data, excluded from Debug and wire encoding.
    pub fn raw(&self) -> Option<&Value> {
        self.raw.as_ref()
    }
    /// Attach reported call metadata.
    pub fn with_call(mut self, call: Arc<Call>) -> Self {
        self.call = Some(call);
        self
    }
    /// Attach raw provider data for explicit inspection.
    pub fn with_raw(mut self, raw: Value) -> Self {
        self.raw = Some(raw);
        self
    }
    /// Apply a threshold without restoring any previously undecided value.
    ///
    /// # Errors
    /// Returns `InvalidArgument` if the threshold is nonfinite or outside `[0, 1]`.
    pub fn with_min_confidence(mut self, threshold: f64) -> Result<Self, Error> {
        if !threshold.is_finite() || !(0.0..=1.0).contains(&threshold) {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                "threshold must be finite and from 0 to 1",
            ));
        }
        if self.confidence < threshold {
            self.value = None;
        }
        Ok(self)
    }
    /// Explicitly remove a decision while retaining all evidence.
    pub fn abstain(mut self) -> Self {
        self.value = None;
        self
    }
}
impl Answer {
    /// Validate the retained provider candidate against the actual requested question.
    /// Existing abstentions remain undecided, including explicit provider abstentions.
    ///
    /// # Errors
    /// Returns the normalization errors from [`make_answer`] if the retained
    /// candidate or distribution is incompatible with `question`.
    pub fn revalidate(&self, question: &Question) -> Result<Self, Error> {
        let explicit = if matches!(question, Question::Box(_)) {
            Some(self.confidence)
        } else {
            None
        };
        let mut answer = make_answer(
            question,
            self.candidate.clone(),
            self.probabilities.clone(),
            explicit,
        )?;
        if self.value.is_none() {
            answer.value = None;
        }
        answer.call = self.call.clone();
        answer.raw = self.raw.clone();
        Ok(answer)
    }
    /// Project into a typed answer, checking type even when the answer is undecided.
    ///
    /// # Errors
    /// Propagates conversion errors from [`AnswerType::from_answer_value`],
    /// including a type mismatch even when this answer is undecided.
    pub fn typed<T: AnswerType>(&self) -> Result<Answer<T>, Error> {
        let candidate = T::from_answer_value(&self.candidate)?;
        Ok(Answer {
            value: self.value.as_ref().map(|_| candidate),
            candidate: self.candidate.clone(),
            probabilities: self.probabilities.clone(),
            confidence: self.confidence,
            call: self.call.clone(),
            raw: self.raw.clone(),
        })
    }
}
fn invalid(msg: &str) -> Error {
    Error::new(ErrorKind::InvalidResponse, msg)
}
/// Validate a provider candidate and centrally apply ties and confidence thresholds.
///
/// A valid response below the threshold is successful but undecided; its
/// probabilities remain available for inspection.
///
/// ```
/// use semantic_operators::{AnswerValue, Boolean, IndexMap, Question, make_answer};
/// # fn main() -> semantic_operators::Result<()> {
/// let question = Question::from(Boolean::new("Does this need review?")?
///     .with_min_confidence(0.8)?);
/// let probabilities = IndexMap::from([("true".into(), 0.7), ("false".into(), 0.3)]);
/// let answer = make_answer(&question, AnswerValue::Boolean(true), probabilities, None)?
///     .typed::<bool>()?;
/// assert_eq!(answer.value(), None);
/// assert_eq!(answer.confidence(), 0.7);
/// assert_eq!(answer.probabilities()["true"], 0.7);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
/// Returns `InvalidResponse` for a mismatched value type, invalid or incomplete
/// probabilities, disagreement between the value and distribution, or invalid
/// Box confidence. Ties and insufficient confidence produce a successful
/// undecided answer rather than an error.
pub fn make_answer(
    question: &Question,
    value: AnswerValue,
    probabilities: Probabilities,
    explicit_confidence: Option<f64>,
) -> Result<Answer, Error> {
    if !matches!(question, Question::Box(_)) && explicit_confidence.is_some() {
        return Err(invalid("explicit confidence is only valid for Box answers"));
    }
    let (confidence, tied) = match (question, &value) {
        (Question::Boolean(_), AnswerValue::Boolean(v)) => {
            distribution(&probabilities, &["true", "false"])?;
            let yes = probabilities["true"];
            let no = probabilities["false"];
            let tied = yes == no;
            if !tied && *v != (yes > no) {
                return Err(invalid("Boolean value disagrees with its probabilities"));
            }
            (yes.max(no), tied)
        }
        (Question::Choice(q), AnswerValue::Choice(v)) => {
            let labels: Vec<_> = q.options().keys().map(String::as_str).collect();
            distribution(&probabilities, &labels)?;
            let top = probabilities.values().copied().fold(0.0, f64::max);
            if probabilities.get(v) != Some(&top) {
                return Err(invalid("Choice value must name a most likely outcome"));
            }
            (
                top,
                probabilities.values().filter(|p| **p == top).count() > 1,
            )
        }
        (Question::Score(q), AnswerValue::Score(v)) => {
            let labels: Vec<_> = q.levels().iter().map(String::as_str).collect();
            distribution(&probabilities, &labels)?;
            let highest = q.levels().len() - 1;
            if !v.is_finite() || *v < 0.0 || *v > highest as f64 {
                return Err(invalid("Score value is outside the rubric"));
            }
            let expected = python_sum(
                q.levels()
                    .iter()
                    .enumerate()
                    .map(|(i, label)| i as f64 * probabilities[label]),
            );
            let margin = 0.005 * (1.0 + (highest as f64 * (highest as f64 + 1.0) / 2.0));
            if (*v - expected).abs() > margin {
                return Err(invalid("Score value disagrees with its probabilities"));
            }
            let nearest = ((*v + 0.5).floor() as usize).min(highest);
            (probabilities[&q.levels()[nearest]], false)
        }
        (Question::Box(_), AnswerValue::Box(_)) => {
            if !probabilities.is_empty() {
                return Err(invalid("Box answers have no probabilities"));
            }
            let c = explicit_confidence.unwrap_or(0.0);
            if !c.is_finite() || !(0.0..=1.0).contains(&c) {
                return Err(invalid("Box confidence must be finite and from 0 to 1"));
            }
            (c, false)
        }
        _ => return Err(invalid("answer value type does not match question")),
    };
    let decided = if tied || confidence < question.min_confidence() {
        None
    } else {
        Some(value.clone())
    };
    Ok(Answer {
        value: decided,
        candidate: value,
        probabilities,
        confidence,
        call: None,
        raw: None,
    })
}
fn distribution(p: &Probabilities, labels: &[&str]) -> Result<(), Error> {
    if p.len() != labels.len() || labels.iter().any(|label| !p.contains_key(*label)) {
        return Err(invalid(
            "probabilities must cover exactly the question outcomes",
        ));
    }
    if p.values()
        .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
    {
        return Err(invalid("probabilities must be finite numbers from 0 to 1"));
    }
    if (accurate_sum(p.values().copied()) - 1.0).abs() > 0.005 * labels.len() as f64 + 1e-9 {
        return Err(invalid("probabilities must sum to 1"));
    }
    Ok(())
}
// Nonoverlapping partial sums with the final half-even correction used by math.fsum.
// Adapted from CPython math.fsum; see NOTICE.md and LICENSE-PYTHON.
fn accurate_sum(values: impl IntoIterator<Item = f64>) -> f64 {
    let mut partials: Vec<f64> = Vec::new();
    for mut x in values {
        let mut i = 0;
        for j in 0..partials.len() {
            let mut y = partials[j];
            if x.abs() < y.abs() {
                std::mem::swap(&mut x, &mut y);
            }
            let hi = x + y;
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
    if partials
        .last()
        .is_some_and(|last| (lo < 0.0 && *last < 0.0) || (lo > 0.0 && *last > 0.0))
    {
        let y = lo * 2.0;
        let x = hi + y;
        if y == x - hi {
            hi = x;
        }
    }
    hi
}
/// Python-compatible `round(1 - p, 10)` for a finite probability in [0, 1].
/// Invalid inputs return an argument error rather than entering normalization.
///
/// # Errors
/// Returns `InvalidArgument` for nonfinite probabilities or values outside
/// `[0, 1]`, and `Runtime` if decimal conversion fails.
pub fn complement(p: f64) -> Result<f64, Error> {
    if !p.is_finite() || !(0.0..=1.0).contains(&p) {
        return Err(Error::new(
            ErrorKind::InvalidArgument,
            "probability must be finite and from 0 to 1",
        ));
    }
    // Formatting rounds the binary float directly to decimal, avoiding the extra
    // rounding introduced by multiplying by 10^10 before rounding to an integer.
    format!("{:.10}", 1.0 - p)
        .parse()
        .map_err(|_| Error::new(ErrorKind::Runtime, "decimal conversion failed"))
}
/// Return a nonnegative JSON integer token count, rejecting booleans and floats.
pub fn token_count(value: &Value) -> Option<u64> {
    value.as_u64()
}

// CPython 3.12+ sum of floats uses Neumaier compensated summation.
fn python_sum(values: impl IntoIterator<Item = f64>) -> f64 {
    let mut hi: f64 = 0.0;
    let mut lo = 0.0;
    for x in values {
        let t = hi + x;
        lo += if hi.abs() >= x.abs() {
            (hi - t) + x
        } else {
            (x - t) + hi
        };
        hi = t;
    }
    hi + lo
}
