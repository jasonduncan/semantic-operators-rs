//! Structured errors without automatic inclusion of request data.
use std::fmt;
/// Classification of a library failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// Invalid question definition.
    InvalidQuestion,
    /// Invalid argument.
    InvalidArgument,
    /// Invalid state shape.
    InvalidState,
    /// Invalid image.
    InvalidImage,
    /// Invalid wire request.
    InvalidRequest,
    /// Malformed provider output.
    InvalidResponse,
    /// Provider or transport failure.
    Provider,
    /// Deadline expired.
    Timeout,
    /// Unsupported capability.
    Unsupported,
    /// Runtime misuse or interrupted local execution.
    Runtime,
}
/// A classified error with an optional field path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    kind: ErrorKind,
    message: String,
    path: Option<String>,
}
impl Error {
    /// Construct an error. Messages should not contain secrets or private state.
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            path: None,
        }
    }
    /// Attach a schema field path.
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
    /// Error classification.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
    /// Safe diagnostic message, excluding the field path.
    pub fn message(&self) -> &str {
        &self.message
    }
    /// Optional schema field path.
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(p) = self.path.as_ref().filter(|p| !p.is_empty()) {
            write!(f, "{p}: ")?;
        }
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
