//! Ordinary Rust control flow with explicit uncertainty and retained traces.
use crate::{
    Answer, Answers, Error, ErrorKind, NamedQuestion, Operator, Provider, Request, State,
    TypedQuestion, ask, operator,
};

/// One attempted request and its result, including failed attempts.
#[derive(Clone, Debug)]
pub struct Step {
    /// Request whose Debug redacts its input and text.
    pub request: Request,
    /// Answers, validation/provider failure, or interrupted local execution.
    pub result: crate::Result<Answers>,
}
/// A structured early exit; handling uncertainty is explicit.
#[derive(Clone, Debug)]
pub enum FlowExit {
    /// An operator was read while undecided.
    Undecided {
        /// Operator identity.
        name: String,
        /// Full retained answer.
        answer: Box<Answer>,
    },
    /// An input or provider failure, separate from uncertainty.
    Error(Error),
}
impl From<Error> for FlowExit {
    fn from(error: Error) -> Self {
        Self::Error(error)
    }
}
/// Results from one ask, read through typed operators.
#[derive(Clone, Debug)]
pub struct FlowAnswers {
    answers: Answers,
}
impl FlowAnswers {
    /// Inspect all answers without requiring decisions.
    pub fn full(&self) -> &Answers {
        &self.answers
    }
    /// Inspect a typed answer without stopping on uncertainty.
    ///
    /// # Errors
    /// Returns an error if the operator is absent, its question does not match
    /// the retained answer, or conversion to its output type fails.
    pub fn peek<Q: TypedQuestion>(
        &self,
        operator: &Operator<Q>,
    ) -> crate::Result<Answer<Q::Output>> {
        operator.answer(&self.answers)
    }
    /// Read a typed value or return an uncertainty stop.
    ///
    /// # Errors
    /// Returns [`FlowExit::Undecided`] when the answer has no decision. Missing
    /// operators, incompatible answers, and conversion failures become
    /// [`FlowExit::Error`].
    pub fn require<Q: TypedQuestion>(&self, operator: &Operator<Q>) -> Result<Q::Output, FlowExit> {
        let answer = self.peek(operator)?;
        match answer.value() {
            Some(value) => Ok(value.clone()),
            None => Err(FlowExit::Undecided {
                name: operator.name().into(),
                answer: Box::new(
                    self.answers
                        .get(operator.name())
                        .ok_or_else(|| {
                            Error::new(ErrorKind::InvalidArgument, "operator not found")
                        })?
                        .revalidate(&operator.question().clone().into())?,
                ),
            }),
        }
    }
}
/// Completed application result or uncertainty stop, with a trace.
#[derive(Clone, Debug)]
pub struct FlowOutcome<T> {
    /// Present for completion, even if T itself is an application Option::None.
    pub value: Option<T>,
    /// Operator that stopped the flow, if any.
    pub stopped_at: Option<String>,
    /// Full answer responsible for a stop.
    pub stopped_answer: Option<Answer>,
    /// Every flow request attempt, including failures before provider dispatch.
    pub trace: Vec<Step>,
}
impl<T> FlowOutcome<T> {
    /// Whether the application completed without an uncertainty stop.
    pub fn decided(&self) -> bool {
        self.stopped_at.is_none()
    }
    /// Number of flow request attempts, including failures before dispatch.
    pub fn calls(&self) -> usize {
        self.trace.len()
    }
}
/// Failed flow retaining its complete trace.
#[derive(Clone, Debug)]
pub struct FlowFailure {
    /// First latched failure.
    pub error: Error,
    /// Every flow request attempt through this context, including interruption.
    pub trace: Vec<Step>,
}
impl std::fmt::Display for FlowFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}
impl std::error::Error for FlowFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}
/// Records calls and latches failures while application code controls sequencing.
pub struct FlowContext<'a> {
    provider: &'a dyn Provider,
    trace: Vec<Step>,
    failure: Option<Error>,
}
impl<'a> FlowContext<'a> {
    /// Begin an empty trace using a borrowed provider.
    pub fn new(provider: &'a dyn Provider) -> Self {
        Self {
            provider,
            trace: Vec::new(),
            failure: None,
        }
    }
    /// Ask independent operators together about one state.
    ///
    /// # Errors
    /// Returns [`FlowExit::Error`] for an empty batch, duplicate operator names,
    /// a previously latched failure, or request/provider/response validation
    /// failures. Successful undecided answers remain available for inspection.
    pub async fn ask(
        &mut self,
        state: &State,
        operators: &[NamedQuestion],
    ) -> Result<FlowAnswers, FlowExit> {
        let request = Request::new(state.clone(), operator::questions(operators)?)?;
        self.ask_request(&request).await
    }
    /// Ask a request that may carry image assets.
    ///
    /// # Errors
    /// Returns [`FlowExit::Error`] for a previously latched failure or any error
    /// from the checked provider call. A new call failure is retained in the
    /// trace and latched so later calls cannot silently proceed.
    ///
    /// Dropping this future after it starts retains the attempt and latches
    /// [`ErrorKind::Runtime`] for interrupted local execution. This does not
    /// establish whether remote work stopped or incurred charges. An unpolled
    /// future records no attempt.
    pub async fn ask_request(&mut self, request: &Request) -> Result<FlowAnswers, FlowExit> {
        if let Some(error) = &self.failure {
            return Err(FlowExit::Error(error.clone()));
        }
        // The exclusive context borrow prevents observing this provisional
        // failure while the future is running. If it is dropped at the await,
        // the retained failure prevents a partial execution appearing complete.
        let interrupted = Error::new(
            ErrorKind::Runtime,
            "flow request ended before local completion",
        );
        let position = self.trace.len();
        self.trace.push(Step {
            request: request.clone(),
            result: Err(interrupted.clone()),
        });
        self.failure = Some(interrupted);
        let result = ask(self.provider, request).await;
        self.trace[position].result = result.clone();
        self.failure = result.as_ref().err().cloned();
        match result {
            Ok(answers) => Ok(FlowAnswers { answers }),
            Err(error) => Err(FlowExit::Error(error)),
        }
    }
    /// Finish. A caught provider error cannot become successful completion.
    ///
    /// # Errors
    /// Returns [`FlowFailure`] with the trace when an earlier call failed,
    /// a polled request future was dropped before completing, or
    /// `result` contains [`FlowExit::Error`]. An uncertainty exit instead becomes
    /// a successful [`FlowOutcome`] with `stopped_at` set.
    pub fn finish<T>(self, result: Result<T, FlowExit>) -> Result<FlowOutcome<T>, FlowFailure> {
        if let Some(error) = self.failure {
            return Err(FlowFailure {
                error,
                trace: self.trace,
            });
        }
        match result {
            Ok(value) => Ok(FlowOutcome {
                value: Some(value),
                stopped_at: None,
                stopped_answer: None,
                trace: self.trace,
            }),
            Err(FlowExit::Undecided { name, answer }) => Ok(FlowOutcome {
                value: None,
                stopped_at: Some(name),
                stopped_answer: Some(*answer),
                trace: self.trace,
            }),
            Err(FlowExit::Error(error)) => Err(FlowFailure {
                error,
                trace: self.trace,
            }),
        }
    }
}
