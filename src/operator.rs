//! Named typed questions and heterogeneous one-call application.
use crate::{
    Answer, Answers, Error, ErrorKind, Provider, Question, Questions, Request, Result, State,
    TypedQuestion, ask,
};

/// A named question erased to its dynamic variant.
#[derive(Clone, Debug)]
pub struct NamedQuestion {
    name: String,
    question: Question,
}
impl NamedQuestion {
    /// Bind a nonempty name to a question.
    ///
    /// # Errors
    /// Returns `InvalidArgument` if the operator name is empty.
    pub fn new(name: impl Into<String>, question: impl Into<Question>) -> Result<Self> {
        let name = name.into();
        if name.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                "operator name must not be empty",
            ));
        }
        Ok(Self {
            name,
            question: question.into(),
        })
    }
    /// Operator identity.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Underlying semantic definition.
    pub fn question(&self) -> &Question {
        &self.question
    }
}
/// A reusable typed question, independent of any inference provider.
///
/// This explicitly scripted provider demonstrates typed output without network
/// access or semantic inference.
///
/// ```
/// use semantic_operators::{
///     AnswerValue, Boolean, FunctionProvider, IndexMap, Operator,
///     ProviderFuture, Request, State, make_answer,
/// };
///
/// fn scripted(request: &Request) -> ProviderFuture<'_> {
///     Box::pin(async move {
///         request.questions().iter().map(|(name, question)| {
///             let probabilities = IndexMap::from([
///                 ("true".into(), 0.1), ("false".into(), 0.9),
///             ]);
///             Ok((name.clone(), make_answer(
///                 question, AnswerValue::Boolean(false), probabilities, None,
///             )?))
///         }).collect()
///     })
/// }
///
/// # fn main() -> semantic_operators::Result<()> {
/// let operator = Operator::new("urgent", Boolean::new("Is this urgent?")?)?;
/// let provider = FunctionProvider::new(scripted);
/// let answer = futures_executor::block_on(operator.ask(&provider, &State::text("Fixture")))?;
/// assert_eq!(answer.value(), Some(&false)); // False is a decided answer.
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct Operator<Q: TypedQuestion> {
    name: String,
    question: Q,
}
impl<Q: TypedQuestion> Operator<Q> {
    /// Create an operator with a nonempty name.
    ///
    /// # Errors
    /// Returns `InvalidArgument` if the operator name is empty.
    pub fn new(name: impl Into<String>, question: Q) -> Result<Self> {
        let name = name.into();
        if name.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                "operator name must not be empty",
            ));
        }
        Ok(Self { name, question })
    }
    /// Stable name used in batches and traces.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Typed question definition.
    pub fn question(&self) -> &Q {
        &self.question
    }
    /// Erase the type for a heterogeneous batch.
    pub fn erased(&self) -> NamedQuestion {
        NamedQuestion {
            name: self.name.clone(),
            question: self.question.clone().into(),
        }
    }
    /// Ask about a state and retain a typed answer, including uncertainty.
    ///
    /// # Errors
    /// Propagates request, provider, response-validation, and typed-conversion
    /// errors. An undecided answer is successful and retains its evidence.
    pub async fn ask<P: Provider + ?Sized>(
        &self,
        provider: &P,
        state: &State,
    ) -> Result<Answer<Q::Output>> {
        let request = Request::new(state.clone(), questions(&[self.erased()])?)?;
        self.ask_request(provider, &request).await
    }
    /// Ask about a request's state and assets, replacing its questions.
    ///
    /// # Errors
    /// Propagates request/image validation, provider, response-validation, and
    /// typed-conversion errors. The supplied request's questions are replaced.
    pub async fn ask_request<P: Provider + ?Sized>(
        &self,
        provider: &P,
        request: &Request,
    ) -> Result<Answer<Q::Output>> {
        let request = request.with_questions(questions(&[self.erased()])?)?;
        let answers = ask(provider, &request).await?;
        self.answer(&answers)
    }
    /// Read this operator's checked typed result from a heterogeneous batch.
    ///
    /// # Errors
    /// Returns `InvalidArgument` if this operator is absent, or a normalization
    /// or conversion error if the retained answer does not match its question/type.
    pub fn answer(&self, answers: &Answers) -> Result<Answer<Q::Output>> {
        answers
            .get(&self.name)
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidArgument,
                    "operator is absent from the answer batch",
                )
            })?
            .revalidate(&self.question.clone().into())?
            .typed()
    }
}
/// Build ordered questions, rejecting empty batches and duplicate names.
///
/// # Errors
/// Returns `InvalidArgument` for an empty operator list or duplicate names.
pub fn questions(operators: &[NamedQuestion]) -> Result<Questions> {
    if operators.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidArgument,
            "operator batch must not be empty",
        ));
    }
    let mut result = Questions::new();
    for operator in operators {
        if result
            .insert(operator.name.clone(), operator.question.clone())
            .is_some()
        {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                "operator names must be unique",
            ));
        }
    }
    Ok(result)
}
/// Ask all operators in one request, preserving its state and assets.
///
/// # Errors
/// Returns `InvalidArgument` for an empty batch or duplicate operator names.
/// Otherwise propagates checked request/image, provider, and response errors.
pub async fn apply<P: Provider + ?Sized>(
    provider: &P,
    request: &Request,
    operators: &[NamedQuestion],
) -> Result<Answers> {
    ask(provider, &request.with_questions(questions(operators)?)?).await
}
