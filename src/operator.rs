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
#[derive(Clone, Debug)]
pub struct Operator<Q: TypedQuestion> {
    name: String,
    question: Q,
}
impl<Q: TypedQuestion> Operator<Q> {
    /// Create an operator with a nonempty name.
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
    pub async fn ask<P: Provider + ?Sized>(
        &self,
        provider: &P,
        state: &State,
    ) -> Result<Answer<Q::Output>> {
        let request = Request::new(state.clone(), questions(&[self.erased()])?)?;
        self.ask_request(provider, &request).await
    }
    /// Ask about a request's state and assets, replacing its questions.
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
pub async fn apply<P: Provider + ?Sized>(
    provider: &P,
    request: &Request,
    operators: &[NamedQuestion],
) -> Result<Answers> {
    ask(provider, &request.with_questions(questions(operators)?)?).await
}
