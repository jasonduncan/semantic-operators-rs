//! Validated text and JSON state.
use crate::{Error, ErrorKind};
use serde_json::Value;
/// State passed to a provider. Debug output intentionally omits its contents.
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "Value", into = "Value")]
pub struct State(Value);
impl State {
    /// Construct text state, including empty text.
    pub fn text(text: impl Into<String>) -> Self {
        Self(Value::String(text.into()))
    }
    /// Borrow the original JSON representation.
    pub fn as_value(&self) -> &Value {
        &self.0
    }
    /// Consume this state into its original JSON representation.
    pub fn into_value(self) -> Value {
        self.0
    }
}
impl TryFrom<Value> for State {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Error> {
        match value {
            Value::String(_) | Value::Object(_) | Value::Array(_) => Ok(Self(value)),
            _ => Err(Error::new(
                ErrorKind::InvalidState,
                "state must be text, a JSON object, or an array",
            )),
        }
    }
}
impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("State(<private>)")
    }
}

impl From<State> for Value {
    fn from(state: State) -> Value {
        state.0
    }
}
