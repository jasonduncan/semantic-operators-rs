//! Validated provider-independent questions and regions.
use crate::{AnswerType, Error, ErrorKind};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
fn invalid(message: &str) -> Error {
    Error::new(ErrorKind::InvalidQuestion, message)
}
fn instructions(value: String) -> Result<String, Error> {
    if is_blank(&value) {
        Err(invalid("instructions must be a non-empty string"))
    } else {
        Ok(value)
    }
}
fn confidence(value: f64) -> Result<f64, Error> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(value)
    } else {
        Err(invalid(
            "min_confidence must be a finite number from 0 to 1",
        ))
    }
}
fn labels<'a>(values: impl IntoIterator<Item = &'a str>) -> Result<(), Error> {
    let values: Vec<_> = values.into_iter().collect();
    if values.len() < 2 {
        return Err(invalid("need at least 2 outcomes"));
    }
    let mut seen = std::collections::HashSet::new();
    if values.iter().any(|v| v.is_empty() || !seen.insert(*v)) {
        return Err(invalid("outcomes must be unique non-empty strings"));
    }
    Ok(())
}
/// A Boolean question with optional descriptions of each outcome.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "BooleanFields")]
pub struct Boolean {
    instructions: String,
    min_confidence: f64,
    true_description: Option<String>,
    false_description: Option<String>,
}
impl Boolean {
    /// Validate instructions and construct a question without a confidence threshold.
    ///
    /// # Errors
    /// Returns `InvalidQuestion` when instructions are empty or contain only
    /// reference-defined whitespace.
    pub fn new(text: impl Into<String>) -> Result<Self, Error> {
        Ok(Self {
            instructions: instructions(text.into())?,
            min_confidence: 0.0,
            true_description: None,
            false_description: None,
        })
    }
    /// Set descriptions for true and false, respectively.
    pub fn with_descriptions(mut self, yes: Option<String>, no: Option<String>) -> Self {
        self.true_description = yes;
        self.false_description = no;
        self
    }
    /// Description of the true outcome.
    pub fn true_description(&self) -> Option<&str> {
        self.true_description.as_deref()
    }
    /// Description of the false outcome.
    pub fn false_description(&self) -> Option<&str> {
        self.false_description.as_deref()
    }
}
/// A choice among ordered named outcomes.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "ChoiceFields")]
pub struct Choice {
    instructions: String,
    min_confidence: f64,
    options: IndexMap<String, Option<String>>,
}
impl Choice {
    /// Validate instructions and at least two distinct nonempty option names.
    ///
    /// # Errors
    /// Returns `InvalidQuestion` for blank instructions, fewer than two options,
    /// or an empty option name. The ordered map supplies unique names.
    pub fn new(
        text: impl Into<String>,
        options: IndexMap<String, Option<String>>,
    ) -> Result<Self, Error> {
        labels(options.keys().map(String::as_str))?;
        Ok(Self {
            instructions: instructions(text.into())?,
            min_confidence: 0.0,
            options,
        })
    }
    /// Ordered option names and descriptions.
    pub fn options(&self) -> &IndexMap<String, Option<String>> {
        &self.options
    }
}
/// An expected score on an ordered rubric.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "ScoreFields")]
pub struct Score {
    instructions: String,
    min_confidence: f64,
    levels: Vec<String>,
}
impl Score {
    /// Validate instructions and at least two unique nonempty levels.
    ///
    /// # Errors
    /// Returns `InvalidQuestion` for blank instructions, fewer than two levels,
    /// or empty or duplicate level descriptions.
    pub fn new(text: impl Into<String>, levels: Vec<String>) -> Result<Self, Error> {
        labels(levels.iter().map(String::as_str))?;
        Ok(Self {
            instructions: instructions(text.into())?,
            min_confidence: 0.0,
            levels,
        })
    }
    /// Rubric levels in score order, starting at zero.
    pub fn levels(&self) -> &[String] {
        &self.levels
    }
}
/// A request to locate a region in an image.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "BoxQuestionFields")]
pub struct BoxQuestion {
    instructions: String,
    min_confidence: f64,
}
impl BoxQuestion {
    /// Construct a localization question.
    ///
    /// # Errors
    /// Returns `InvalidQuestion` when instructions are empty or contain only
    /// reference-defined whitespace.
    pub fn new(text: impl Into<String>) -> Result<Self, Error> {
        Ok(Self {
            instructions: instructions(text.into())?,
            min_confidence: 0.0,
        })
    }
}
macro_rules! common { ($($ty:ty),*) => {$(
    impl $ty {
        /// Original instructions, preserved without trimming.
        pub fn instructions(&self)->&str { &self.instructions }
        /// Minimum confidence required for a decision.
        pub fn min_confidence(&self)->f64 { self.min_confidence }
        /// Set a finite confidence threshold in [0, 1].
        ///
        /// # Errors
        /// Returns `InvalidQuestion` if the threshold is nonfinite or outside `[0, 1]`.
        pub fn with_min_confidence(mut self, value:f64)->Result<Self,Error> { self.min_confidence=confidence(value)?; Ok(self) }
    }
)*}; }
common!(Boolean, Choice, Score, BoxQuestion);
/// A heterogeneous question.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Question {
    /// A yes/no question.
    Boolean(Boolean),
    /// An ordered choice.
    Choice(Choice),
    /// An expected rubric score.
    Score(Score),
    /// A localization question.
    Box(BoxQuestion),
}
impl Question {
    /// Original instructions.
    pub fn instructions(&self) -> &str {
        match self {
            Self::Boolean(q) => q.instructions(),
            Self::Choice(q) => q.instructions(),
            Self::Score(q) => q.instructions(),
            Self::Box(q) => q.instructions(),
        }
    }
    /// Decision threshold.
    pub fn min_confidence(&self) -> f64 {
        match self {
            Self::Boolean(q) => q.min_confidence(),
            Self::Choice(q) => q.min_confidence(),
            Self::Score(q) => q.min_confidence(),
            Self::Box(q) => q.min_confidence(),
        }
    }
    /// Schema name of this question type.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Boolean(_) => "boolean",
            Self::Choice(_) => "choice",
            Self::Score(_) => "score",
            Self::Box(_) => "box",
        }
    }
}
/// A question whose output has a statically known type.
pub trait TypedQuestion: Clone + Into<Question> {
    /// Type of a decided answer.
    type Output: AnswerType;
}
macro_rules! typed {
    ($ty:ty,$variant:ident,$output:ty) => {
        impl From<$ty> for Question {
            fn from(q: $ty) -> Self {
                Self::$variant(q)
            }
        }
        impl TypedQuestion for $ty {
            type Output = $output;
        }
    };
}
typed!(Boolean, Boolean, bool);
typed!(Choice, Choice, String);
typed!(Score, Score, f64);
typed!(BoxQuestion, Box, Region);
/// Normalized image edges. Zero-area regions are valid.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RegionFields")]
pub struct Region {
    top: f64,
    bottom: f64,
    left: f64,
    right: f64,
}
impl Region {
    /// Validate finite edges in [0, 1], with top <= bottom and left <= right.
    ///
    /// # Errors
    /// Returns `InvalidArgument` if an edge is nonfinite or outside `[0, 1]`,
    /// if top exceeds bottom, or if left exceeds right. Zero-area regions are valid.
    pub fn new(top: f64, bottom: f64, left: f64, right: f64) -> Result<Self, Error> {
        if [top, bottom, left, right]
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
            || top > bottom
            || left > right
        {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                "region edges must be ordered finite numbers from 0 to 1",
            ));
        }
        Ok(Self {
            top,
            bottom,
            left,
            right,
        })
    }
    /// Top edge.
    pub fn top(&self) -> f64 {
        self.top
    }
    /// Bottom edge.
    pub fn bottom(&self) -> f64 {
        self.bottom
    }
    /// Left edge.
    pub fn left(&self) -> f64 {
        self.left
    }
    /// Right edge.
    pub fn right(&self) -> f64 {
        self.right
    }
    /// Normalized width.
    pub fn width(&self) -> f64 {
        self.right - self.left
    }
    /// Normalized height.
    pub fn height(&self) -> f64 {
        self.bottom - self.top
    }
    /// Center as (x, y).
    pub fn center(&self) -> (f64, f64) {
        (
            (self.left + self.right) / 2.0,
            (self.top + self.bottom) / 2.0,
        )
    }
    /// Grow each side by a fraction of the region size, clipping to the image.
    ///
    /// # Errors
    /// Returns `InvalidArgument` for nonfinite padding or resulting invalid
    /// edges, such as negative padding that shrinks past the opposite edge.
    pub fn padded(&self, fraction: f64) -> Result<Self, Error> {
        if !fraction.is_finite() {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                "padding must be finite",
            ));
        }
        let dx = self.width() * fraction;
        let dy = self.height() * fraction;
        Self::new(
            (self.top - dy).max(0.0),
            (self.bottom + dy).min(1.0),
            (self.left - dx).max(0.0),
            (self.right + dx).min(1.0),
        )
    }
    /// Pixel edges (left, top, right, bottom), using ties-to-even rounding.
    ///
    /// ```
    /// use semantic_operators::Region;
    /// # fn main() -> semantic_operators::Result<()> {
    /// let region = Region::new(0.2, 0.6, 0.1, 0.5)?;
    /// assert_eq!(region.pixels(1000, 500)?, (100, 100, 500, 300));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Returns `InvalidArgument` if either pixel dimension is zero.
    pub fn pixels(&self, width: u32, height: u32) -> Result<(u32, u32, u32, u32), Error> {
        if width == 0 || height == 0 {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                "pixel dimensions must be positive",
            ));
        }
        Ok((
            (self.left * f64::from(width)).round_ties_even() as u32,
            (self.top * f64::from(height)).round_ties_even() as u32,
            (self.right * f64::from(width)).round_ties_even() as u32,
            (self.bottom * f64::from(height)).round_ties_even() as u32,
        ))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BooleanFields {
    instructions: String,
    #[serde(default)]
    min_confidence: f64,
    #[serde(default)]
    true_description: Option<String>,
    #[serde(default)]
    false_description: Option<String>,
}
impl TryFrom<BooleanFields> for Boolean {
    type Error = Error;
    fn try_from(v: BooleanFields) -> Result<Self, Error> {
        Self::new(v.instructions)?
            .with_descriptions(v.true_description, v.false_description)
            .with_min_confidence(v.min_confidence)
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChoiceFields {
    instructions: String,
    #[serde(default)]
    min_confidence: f64,
    options: IndexMap<String, Option<String>>,
}
impl TryFrom<ChoiceFields> for Choice {
    type Error = Error;
    fn try_from(v: ChoiceFields) -> Result<Self, Error> {
        Self::new(v.instructions, v.options)?.with_min_confidence(v.min_confidence)
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScoreFields {
    instructions: String,
    #[serde(default)]
    min_confidence: f64,
    levels: Vec<String>,
}
impl TryFrom<ScoreFields> for Score {
    type Error = Error;
    fn try_from(v: ScoreFields) -> Result<Self, Error> {
        Self::new(v.instructions, v.levels)?.with_min_confidence(v.min_confidence)
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BoxQuestionFields {
    instructions: String,
    #[serde(default)]
    min_confidence: f64,
}
impl TryFrom<BoxQuestionFields> for BoxQuestion {
    type Error = Error;
    fn try_from(v: BoxQuestionFields) -> Result<Self, Error> {
        Self::new(v.instructions)?.with_min_confidence(v.min_confidence)
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RegionFields {
    top: f64,
    bottom: f64,
    left: f64,
    right: f64,
}
impl TryFrom<RegionFields> for Region {
    type Error = Error;
    fn try_from(v: RegionFields) -> Result<Self, Error> {
        Self::new(v.top, v.bottom, v.left, v.right)
    }
}

macro_rules! private_debug {
    ($($ty:ty),*) => {$(
        impl std::fmt::Debug for $ty {
            fn fmt(&self, f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
                f.debug_struct(stringify!($ty)).field("min_confidence",&self.min_confidence).finish_non_exhaustive()
            }
        }
    )*};
}
private_debug!(Boolean, Choice, Score, BoxQuestion);

// Python str.strip includes the four ASCII information separators in addition
// to Unicode White_Space; Rust str::trim does not.
pub(crate) fn is_blank(value: &str) -> bool {
    value
        .chars()
        .all(|c| c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c))
}
