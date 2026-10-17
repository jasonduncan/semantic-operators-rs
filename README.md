# Semantic Operators for Rust

Define semantic questions once, ask interchangeable providers, and retain an
explicit undecided answer when confidence is insufficient. Native Rust counterpart
to Python semantic-operators 0.12 and C++ libsemop.

**Implementation candidate; not published.** Boolean, Choice, Score, Box, typed
operators, async providers, cascade, deadlines, flows, filtering, reranking,
evaluation, JSON wire translation, images, and TypeSafe transport are implemented.
Verification uses frozen Python outputs, scripted providers and localhost HTTP;
no hosted inference or model-quality qualification has been performed.

- Cargo package: `semantic-operators`; import: `semantic_operators`.
- Edition 2024, MSRV Rust 1.85, MIT license.
- Default features have no network or Tokio dependency.
- Private repository: [jasonduncan/semantic-operators-rs](https://github.com/jasonduncan/semantic-operators-rs).
- Release publication remains disabled. Crate name appears unregistered, not reserved.

## Use it locally

```toml
[dependencies]
semantic-operators = { path = "../semantic-operators-rs" }
```

Define a typed operation and ask a provider supplied by your application:

```rust,ignore
use semantic_operators::{Boolean, Operator, State};

let complaint = Operator::new(
    "complaint",
    Boolean::new("Is the customer reporting a problem?")?.with_min_confidence(0.8)?,
)?;
let answer = complaint.ask(&provider, &State::text(message)).await?;
match answer.value() {
    Some(true) => { /* triage */ }
    Some(false) => { /* ordinary message */ }
    None => { /* ask a person */ }
}
```

The snippet assumes an application provider, message and async Result context.
For complete runnable programs, use the explicit synthetic examples:

```sh
cargo run --example hello
cargo run --example triage
cargo run --example filter
cargo run --example rerank
cargo run --features images --example images
```

These examples make no hosted calls. Their answers demonstrate control flow, not
model accuracy. `Answer::raw()` is opt-in for the TypeSafe adapter and is never
included in ordinary Debug or wire output.

## Features

| Feature | Adds |
|---|---|
| default (empty) | Validated types, async provider trait, operators, cascade, flows, recipes, evaluation, text wire |
| `tokio` | `TimeLimited` deadlines using a caller-owned time-enabled Tokio runtime |
| `typesafe` | Explicit-config TypeSafe HTTP provider using rustls; includes `tokio` |
| `images` | Image inspection, request-owned assets, frames and locate |
| `blocking` | Explicit synchronous facade with its own runtime; includes `tokio` |

Enable `typesafe` and `images` together for hosted image requests. Core image
validation follows the pinned Python behavior; the adapter additionally checks
current service restrictions. See [image boundaries](docs/IMAGES.md).

## Read and develop

- [API guide](docs/API.md): actual API, async ownership, uncertainty, recipes.
- [Design](docs/DESIGN.md) and [compatibility](docs/COMPATIBILITY.md): requirements and exact parity rules.
- [TypeSafe transport](docs/TYPE_SAFE_TRANSPORT.md): explicit credentials, endpoint, retries, HTTP limits.
- [Development](docs/DEVELOPMENT.md), [test strategy](docs/TESTING.md), [source baseline](docs/SOURCE_BASELINE.md).
- [Current evidence](status/VALIDATION.md), [handoff](status/HANDOFF.md), [release procedure](docs/RELEASE.md).

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-features --all-targets -- -D warnings
cargo test --locked --all-targets
cargo test --locked --all-features --all-targets
cargo test --locked --all-features --doc
cargo doc --locked --all-features --no-deps
python3 tools/check_docs.py
python3 tools/check_package.py
```

Default and optional-feature suites are offline except localhost HTTP tests. No
CLI/MCP server, Laya runtime, native Rank, caching, or agent framework is included.
Provider confidence is not a guarantee of correctness; measure it on your task.
