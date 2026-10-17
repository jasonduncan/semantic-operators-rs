//! Typed semantic questions, confidence-aware answers, and provider-neutral composition.
//!
//! Default features require no network or particular async runtime. Enable
//! `typesafe`, `images`, `tokio`, or `blocking` for optional integrations.

#![doc = include_str!("../CRATE.md")]

pub mod answer;
pub mod bench;
#[cfg(feature = "blocking")]
pub mod blocking;
pub mod cascade;
pub mod each;
pub mod error;
pub mod filter;
pub mod flow;
#[cfg(feature = "images")]
pub mod image;
#[cfg(feature = "images")]
pub mod locate;
pub mod operator;
pub mod provider;
#[cfg(feature = "typesafe")]
pub mod providers;
pub mod question;
pub mod rerank;
pub mod state;
pub mod wire;

pub use answer::{
    Answer, AnswerType, AnswerValue, Call, Probabilities, complement, make_answer, token_count,
};
pub use cascade::Cascade;
#[cfg(feature = "tokio")]
pub use cascade::TimeLimited;
pub use error::{Error, ErrorKind};
pub use indexmap::IndexMap;
pub use operator::{NamedQuestion, Operator, apply};
pub use provider::{Answers, FunctionProvider, Provider, ProviderFuture, Questions, Request, ask};
pub use question::{Boolean, BoxQuestion, Choice, Question, Region, Score, TypedQuestion};
pub use state::State;

/// A library result. Uncertainty is represented inside a successful answer.
pub type Result<T, E = Error> = std::result::Result<T, E>;
