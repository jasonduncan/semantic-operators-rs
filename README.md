# Semantic Operators for Rust

[![crates.io](https://img.shields.io/crates/v/semantic-operators.svg)](https://crates.io/crates/semantic-operators)
[![docs.rs](https://docs.rs/semantic-operators/badge.svg)](https://docs.rs/semantic-operators)
[![CI](https://github.com/jasonduncan/semantic-operators-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/jasonduncan/semantic-operators-rs/actions/workflows/ci.yml)
[![License: MIT and Python-2.0](https://img.shields.io/badge/license-MIT%20AND%20Python--2.0-blue.svg)](NOTICE.md)
[![Rust: 1.85+](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](VERSIONING.md)

Define semantic questions once, ask interchangeable providers, and retain an
explicit undecided answer when confidence is insufficient. Native Rust counterpart
to Python semantic-operators 0.12 and C++ libsemop.

**Initial alpha; API unstable.** Boolean, Choice, Score, Box, typed
operators, async providers, cascade, deadlines, flows, filtering, reranking,
evaluation, JSON wire translation, images, and TypeSafe transport are implemented.
Verification uses frozen Python outputs, scripted providers and localhost HTTP;
no hosted inference or model-quality qualification has been performed.

- Cargo package: `semantic-operators`; import: `semantic_operators`.
- Edition 2024, MSRV Rust 1.85. Original code is MIT; CPython-derived portions retain Python-2.0 terms.
- Default features have no network or Tokio dependency.
- Repository: [jasonduncan/semantic-operators-rs](https://github.com/jasonduncan/semantic-operators-rs).
- Release: [0.1.0-alpha.1 on crates.io](https://crates.io/crates/semantic-operators/0.1.0-alpha.1).
- [Rust API documentation](https://docs.rs/semantic-operators/0.1.0-alpha.1/semantic_operators/).

## Install

```toml
[dependencies]
semantic-operators = "=0.1.0-alpha.1"
```

Add the dependency above to a Cargo binary project and paste this complete
program into `src/main.rs`. It checks a scripted response locally; it makes no
model request and needs no credentials or async runtime.

```rust
use semantic_operators::{AnswerValue, Boolean, IndexMap, Question, make_answer};

fn main() -> semantic_operators::Result<()> {
    let question = Question::from(
        Boolean::new("Is the customer reporting a problem?")?
            .with_min_confidence(0.8)?,
    );
    let probabilities = IndexMap::from([
        ("true".into(), 0.7),
        ("false".into(), 0.3),
    ]);
    let answer = make_answer(
        &question, AnswerValue::Boolean(true), probabilities, None,
    )?.typed::<bool>()?;

    assert_eq!(answer.value(), None); // Below the threshold: ask a person.
    assert_eq!(answer.probabilities()["true"], 0.7); // Evidence is retained.
    Ok(())
}
```

For inference, bind a question to an `Operator`, then call
`operator.ask(&provider, &state).await` using your application's provider and
executor. `Answer<bool>` contains `Some(true)`, `Some(false)`, or `None` for
abstention; provider failures remain `Result` errors. The
[API guide](docs/API.md) covers providers and typed operators.

To run the complete synthetic provider examples, clone this repository first:

```sh
git clone https://github.com/jasonduncan/semantic-operators-rs.git
cd semantic-operators-rs
```

```sh
cargo run --example hello
cargo run --example triage
cargo run --example filter
cargo run --example rerank
cargo run --features images --example images
```

For a standalone application with its own manifest, JSON Lines input/output and
real HTTP adapter integration, see [the triage consumer](integrations/triage-app/README.md).
Its end-to-end verification uses a local fixture server:

```sh
python3 tools/check_triage_app.py
```

The [bounded provider-check packet](qualification/README.md) supplies 12 synthetic
tickets, a default-off runner, and an offline report scorer. Execution needs an
explicit model and call limit; preparing this packet makes no provider requests.

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

Default and optional-feature suites are offline except localhost HTTP tests. The generic
wire CLI/MCP server, Laya runtime, native Rank, caching, and agent framework remain deferred.
Provider confidence is not a guarantee of correctness; measure it on your task.

## Compatibility and support

The published `0.1.0-alpha.1` API is experimental. Pin an exact alpha version
while evaluating it; inspect the [changelog](CHANGELOG.md) before upgrading.
Rust 1.85 is the minimum supported compiler, checked alongside stable Rust on
Linux, macOS and Windows. [Versioning](VERSIONING.md) describes breaking changes,
MSRV updates and deprecation expectations. The main branch can contain changes
not yet published to crates.io.

Bug reports, feature proposals and usage questions belong in
[GitHub issues](https://github.com/jasonduncan/semantic-operators-rs/issues).
See [Contributing](CONTRIBUTING.md) for setup, checks and pull requests, and
[Governance](GOVERNANCE.md) for maintainership and support expectations.
Report vulnerabilities through the private channel in [Security](SECURITY.md).

## License

Original project code is licensed under [MIT](LICENSE). CPython-derived numeric
code retains the [Python-2.0 license and notices](LICENSE-PYTHON); the current
source declares the aggregate license `MIT AND Python-2.0`. See [NOTICE.md](NOTICE.md) for provenance.
New contributions submitted for inclusion use MIT terms; upstream portions retain
their own terms and notices. Only contribute material you have permission to
license. See [contribution terms](CONTRIBUTING.md#contribution-terms).
