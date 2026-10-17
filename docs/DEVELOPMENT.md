# Development guide

Start at [status/HANDOFF](../status/HANDOFF.md). This is a new native Rust library;
the bootstrap intentionally contains no public API or placeholder provider.

## Local setup

Use a Rust installation with cargo, rustfmt and clippy; Python 3.11+ is used for
documentation checks and reference fixture generation. Edition 2024 and Rust 1.85
are the working minimum; CI tests 1.85.0 plus stable. The local verification
toolchain is recorded in [VALIDATION](../status/VALIDATION.md). There is no toolchain
file that causes an unexpected installation on opening the checkout.

```sh
cargo fmt --all -- --check
cargo check --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo test --locked --doc
cargo doc --locked --no-deps
python3 tools/check_docs.py
```

Commit Cargo.lock for reproducible project/CI checks. Once dependencies are added,
also test the package as a downstream consumer without relying on this lockfile.
No default check needs credentials or external model access. The scaffold has
zero Rust tests; passing it proves no semantic behavior.

## First implementation slice

Implement `question`, `answer`, `error`, ordered collections, and `state` in M1.
Keep constructors fallible, fields private, deserialization validated. Port
`make_answer` once and use it everywhere. Add a Rust fixture reader for
`tests/fixtures/python-parity.json`; implement cases without changing expected
outputs. Expand fixture coverage before claiming the whole normalizer complete.
See the [full plan](../status/DEV_PLAN.md) for milestone exit criteria.

## Reference fixture maintenance

```sh
python3 -B tools/export_python_fixtures.py \
  --source /path/to/semantic-operators-lib \
  --output tests/fixtures/python-parity.json
```

The exporter verifies the pinned source revision and tracked cleanliness. It
imports only local reference modules, runs synthetic cases, and makes no provider
calls. A baseline change is a compatibility decision: inspect the source diff,
update inventory/revision deliberately, regenerate, and explain changed cases.
Do not execute a source checkout merely obtained from an untrusted remote.

## Change discipline

Keep implementation commits bounded by module/behavior. Add tests for outcomes,
numeric boundaries, cancellation and ordering, not for incidental private layout.
Compile proposed API examples once the corresponding API exists. Document the
chosen dependency version, feature impact, license and MSRV when adding it.
Use `cargo tree -e features` to keep network/runtime dependencies out of core.

When features land, extend CI with no-default-features, each individual supported
feature combination, all-features, package/consumer tests, and deny warnings in
rustdoc. Never declare empty features as if an integration works. Keep live tests
outside default CI; a separate opt-in command needs an explicit bounded brief.

## Handoff

Record actual checks, snapshot, limitations and external effects in status.
Update the next action, development plan and cumulative synthesis; increment
handoff_version for material changes. Commit and push requested work, validate
the status manifest, and refresh the active-project pointer. No keys, live raw
requests, or private documents belong in fixture commits.

