# Semantic Operators

Native Rust questions and confidence-aware answers for interchangeable semantic
model providers. The implementation is validated with
frozen Python outputs and offline scripted/localhost tests. Hosted inference and
model quality have not been qualified.

Version 0.1.0-alpha.1 is an initial alpha with an unstable API. It is a native
Rust counterpart to Python semantic-operators 0.12 and C++ libsemop. No Python
interpreter or C++ FFI is required. The implementation preserves the frozen Python
normalization and text wire contracts with documented Rust adaptations.

```toml
[dependencies]
semantic-operators = "=0.1.0-alpha.1"
```

Default features provide Boolean, Choice, Score and Box questions, typed operators,
async provider contracts, uncertainty-only cascades, traced flows, filtering,
reranking, evaluation and Python-compatible text wire translation. No network
client or Tokio runtime is required by default.

| Feature | Adds |
|---|---|
| default (empty) | Core types, providers, operators, flows, recipes, evaluation and text wire |
| `typesafe` | Explicit-config HTTP provider; includes `tokio` |
| `images` | Assets, frames and localization |
| `tokio` | Deadline wrapper using a caller-owned runtime |
| `blocking` | Synchronous facade with its own runtime; includes `tokio` |

For hosted images select both `typesafe` and `images`. Edition 2024; minimum
supported Rust version 1.85, tested with stable Rust on Linux, macOS and Windows.

```rust
use semantic_operators::{Boolean, Operator};

fn main() -> semantic_operators::Result<()> {
    let complaint = Operator::new(
        "complaint",
        Boolean::new("Is the customer reporting a problem?")?.with_min_confidence(0.8)?,
    )?;
    assert_eq!(complaint.name(), "complaint");
    Ok(())
}
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

Pin an exact prerelease while evaluating the alpha. The
[versioning policy](https://github.com/jasonduncan/semantic-operators-rs/blob/main/VERSIONING.md)
explains API and compiler compatibility; repository main may include unreleased changes.

Original project code uses the [MIT License](https://github.com/jasonduncan/semantic-operators-rs/blob/main/LICENSE).
CPython-derived numeric code retains [Python-2.0 terms](https://github.com/jasonduncan/semantic-operators-rs/blob/main/LICENSE-PYTHON);
the current source declares the aggregate license `MIT AND Python-2.0`. See
[NOTICE.md](https://github.com/jasonduncan/semantic-operators-rs/blob/main/NOTICE.md).
New contributions use MIT terms; preserve upstream licenses and notices.
See [Contributing](https://github.com/jasonduncan/semantic-operators-rs/blob/main/CONTRIBUTING.md)
and [Governance](https://github.com/jasonduncan/semantic-operators-rs/blob/main/GOVERNANCE.md).
