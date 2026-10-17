# Semantic Operators for Rust

A native Rust counterpart to Python **semantic-operators** and C++ **libsemop**:
define a semantic question once, ask interchangeable providers, and retain an
explicit undecided answer when confidence is insufficient.

**Status: design and project bootstrap, 2026-10-17.** The Cargo crate builds but
exports no semantic API yet. Examples in the design documents are proposed APIs.
No inference provider, CLI, or Rust parity suite is implemented. Nothing has been
published to crates.io, and the crate name is not reserved.

| Identity | Value |
|---|---|
| Cargo package | `semantic-operators` |
| Rust import | `semantic_operators` (consumers may alias it to `semops`) |
| Repository | [jasonduncan/semantic-operators-rs](https://github.com/jasonduncan/semantic-operators-rs), private |
| Working version | `0.1.0-dev.0`, publication disabled |
| Rust | Edition 2024; proposed MSRV 1.85; see validation for tested toolchains |
| License | MIT, matching the sibling libraries |

## Start here

- [Design](docs/DESIGN.md): scope, architecture, invariants, and module boundaries.
- [Proposed API](docs/API.md): typed operators, provider contract, async execution, flows.
- [Compatibility](docs/COMPATIBILITY.md): exact Python behavior, C++ differences, parity gates.
- [Providers and images](docs/PROVIDERS.md): transport ownership, cancellation, images.
- [Development guide](docs/DEVELOPMENT.md) and [test strategy](docs/TESTING.md).
- [Decisions](docs/DECISIONS.md), [source baseline](docs/SOURCE_BASELINE.md), [release guide](docs/RELEASE.md).
- [Current state](status/CURRENT.md), [development plan](status/DEV_PLAN.md), [handoff](status/HANDOFF.md).

## Check the bootstrap

```sh
cargo fmt --all -- --check
cargo check --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo test --locked --doc
cargo doc --locked --no-deps
python3 tools/check_docs.py
```

These checks currently establish build and documentation hygiene only. There are
no semantic implementation tests yet. Start M1 with the frozen Python-generated
[parity cases](tests/fixtures/python-parity.json) and the development plan.

