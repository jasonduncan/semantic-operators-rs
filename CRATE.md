# Semantic Operators

Native Rust questions and confidence-aware answers for interchangeable semantic
model providers. This is an unpublished implementation candidate, validated with
frozen Python outputs and offline scripted/localhost tests. Hosted inference and
model quality have not been qualified.

Default features provide Boolean, Choice, Score and Box questions, typed operators,
async provider contracts, uncertainty-only cascades, traced flows, filtering,
reranking, evaluation and Python-compatible text wire translation. No network
client or Tokio runtime is required by default.

Optional features: `typesafe` (explicit-config HTTP provider), `images` (assets,
frames and localization), `tokio` (deadline wrapper), `blocking` (sync facade).
For hosted images select both `typesafe` and `images`. Edition 2024, Rust 1.85, MIT.

```rust
use semantic_operators::{Boolean, Operator};
let complaint = Operator::new(
    "complaint",
    Boolean::new("Is the customer reporting a problem?")?.with_min_confidence(0.8)?,
)?;
assert_eq!(complaint.name(), "complaint");
# Ok::<(), semantic_operators::Error>(())
```

Call `complaint.ask(&provider, &state).await` from your application's executor.
The result is `Answer<bool>`: Some(true), Some(false), or None when undecided.
Probability evidence and optional reported call metadata remain available.
Provider failures are Result errors, never ordinary undecided answers.

Runnable examples included in this package use an explicit synthetic provider
and make no hosted calls: `cargo run --example hello`, `triage`, `filter`, `rerank`,
and `cargo run --features images --example images`. They demonstrate APIs and
control flow, not inference accuracy.

See the [repository](https://github.com/jasonduncan/semantic-operators-rs),
[API guide](https://github.com/jasonduncan/semantic-operators-rs/blob/main/docs/API.md),
and [compatibility contract](https://github.com/jasonduncan/semantic-operators-rs/blob/main/docs/COMPATIBILITY.md).
Provider confidence is not a guarantee of correctness; evaluate your own task.
