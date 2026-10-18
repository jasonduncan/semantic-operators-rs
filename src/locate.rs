//! Presence-gated localization in one provider invocation.
use crate::{
    Answer, AnswerValue, Boolean, BoxQuestion, Error, ErrorKind, Provider, Region, Request, Result,
    ask,
};
use indexmap::IndexMap;

/// Instructions and independent thresholds for presence and localization.
#[derive(Clone, Debug)]
pub struct LocateOptions {
    /// Description of the image placement, such as "`photo`".
    pub where_: String,
    /// Minimum confidence for the Boolean presence answer.
    pub min_presence: f64,
    /// Minimum confidence for the Box answer.
    pub min_box_confidence: f64,
}
impl Default for LocateOptions {
    fn default() -> Self {
        Self {
            where_: "the image".into(),
            min_presence: 0.5,
            min_box_confidence: 0.0,
        }
    }
}
/// Localization result retaining both answers, even when the target is absent.
#[derive(Clone, Debug)]
pub struct Located {
    /// True or false when presence is decided, otherwise absent.
    pub found: Option<bool>,
    /// Decided region only when presence is confidently true.
    pub region: Option<Region>,
    /// Full presence answer and provenance.
    pub presence: Answer<AnswerValue>,
    /// Full Box answer and provenance, whether or not the target is present.
    pub box_answer: Answer<AnswerValue>,
}

/// Ask presence and Box together, retaining the request's state and image assets.
///
/// The original questions are replaced by `present` and `box`. A Box answer alone
/// is never interpreted as evidence that the target exists. Provider capabilities
/// and validation are checked through the ordinary request boundary.
///
/// # Errors
/// Returns `InvalidArgument` for a blank target, `InvalidQuestion` for an
/// invalid confidence threshold, and propagates request/image, provider, or
/// response-validation errors. Uncertain presence or location remains a
/// successful result with the corresponding optional value absent.
pub async fn locate(
    provider: &dyn Provider,
    request: &Request,
    target: &str,
    options: LocateOptions,
) -> Result<Located> {
    if crate::question::is_blank(target) {
        return Err(Error::new(
            ErrorKind::InvalidArgument,
            "target must be a non-empty description",
        ));
    }
    let presence = Boolean::new(format!("Is there {target} in {}?", options.where_))?
        .with_min_confidence(options.min_presence)?;
    let region = BoxQuestion::new(format!("{target} in {}", options.where_))?
        .with_min_confidence(options.min_box_confidence)?;
    let questions = IndexMap::from([
        ("present".into(), presence.into()),
        ("box".into(), region.into()),
    ]);
    let request = request.with_questions(questions)?;
    let mut answers = ask(provider, &request).await?;
    let presence = answers
        .shift_remove("present")
        .ok_or_else(|| Error::new(ErrorKind::InvalidResponse, "missing presence answer"))?;
    let box_answer = answers
        .shift_remove("box")
        .ok_or_else(|| Error::new(ErrorKind::InvalidResponse, "missing box answer"))?;
    let found = match presence.value() {
        Some(AnswerValue::Boolean(value)) => Some(*value),
        _ => None,
    };
    let region = if found == Some(true) {
        match box_answer.value() {
            Some(AnswerValue::Box(region)) => Some(*region),
            _ => None,
        }
    } else {
        None
    };
    Ok(Located {
        found,
        region,
        presence,
        box_answer,
    })
}
