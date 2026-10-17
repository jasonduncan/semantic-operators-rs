//! Deliberately synthetic answers for offline examples; this is not inference.
use semantic_operators::{
    AnswerValue, Answers, Call, Error, ErrorKind, IndexMap, Provider, ProviderFuture, Question,
    Region, Request, complement, make_answer,
};
use serde_json::Value;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

/// A deterministic fixture interpreter, never a fallback for a real provider.
#[derive(Default)]
pub struct SyntheticProvider {
    calls: AtomicUsize,
}
impl SyntheticProvider {
    /// Count explicit synthetic invocations.
    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}
impl Provider for SyntheticProvider {
    fn ask<'a>(&'a self, request: &'a Request) -> ProviderFuture<'a> {
        Box::pin(async move {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let state = request.state().as_value();
            let fixture = state
                .get("document")
                .or_else(|| state.get("item"))
                .unwrap_or(state);
            if fixture.get("demo_provider_error") == Some(&Value::Bool(true)) {
                return Err(Error::new(
                    ErrorKind::Provider,
                    "intentional offline fixture failure",
                ));
            }
            let call = Arc::new(Call {
                provider: "offline-fixture".into(),
                model: Some("synthetic-fixture-v1".into()),
                input_tokens: None,
                output_tokens: None,
            });
            let mut answers = Answers::new();
            for (name, question) in request.questions() {
                let (value, probabilities, confidence) = match question {
                    Question::Boolean(_) => {
                        let p = fixture
                            .get("demo_yes_probability")
                            .and_then(Value::as_f64)
                            .unwrap_or(0.9);
                        (
                            AnswerValue::Boolean(p > 0.5),
                            IndexMap::from([("true".into(), p), ("false".into(), complement(p)?)]),
                            None,
                        )
                    }
                    Question::Choice(q) => {
                        let chosen = q
                            .options()
                            .get_index(0)
                            .ok_or_else(|| {
                                Error::new(ErrorKind::InvalidQuestion, "fixture requires choices")
                            })?
                            .0;
                        let probabilities = q
                            .options()
                            .keys()
                            .map(|key| (key.clone(), if key == chosen { 1.0 } else { 0.0 }))
                            .collect();
                        (AnswerValue::Choice(chosen.clone()), probabilities, None)
                    }
                    Question::Score(q) => {
                        let probabilities: IndexMap<String, f64> = if let Some(p) = fixture
                            .get("demo_score_probabilities")
                            .and_then(Value::as_array)
                        {
                            if p.len() != q.levels().len() {
                                return Err(Error::new(
                                    ErrorKind::InvalidResponse,
                                    "fixture probability count differs from rubric",
                                ));
                            }
                            q.levels()
                                .iter()
                                .cloned()
                                .zip(p.iter().map(|p| p.as_f64().unwrap_or(f64::NAN)))
                                .collect()
                        } else {
                            q.levels()
                                .iter()
                                .enumerate()
                                .map(|(i, level)| {
                                    (
                                        level.clone(),
                                        if i + 1 == q.levels().len() { 1.0 } else { 0.0 },
                                    )
                                })
                                .collect()
                        };
                        let expected = probabilities
                            .values()
                            .enumerate()
                            .map(|(i, p)| i as f64 * p)
                            .sum();
                        (AnswerValue::Score(expected), probabilities, None)
                    }
                    Question::Box(_) => (
                        AnswerValue::Box(Region::new(0.2, 0.7, 0.1, 0.6)?),
                        IndexMap::new(),
                        Some(0.9),
                    ),
                };
                answers.insert(
                    name.clone(),
                    make_answer(question, value, probabilities, confidence)?
                        .with_call(call.clone()),
                );
            }
            Ok(answers)
        })
    }
}
