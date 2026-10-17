# Development guide

Start at [status/HANDOFF](../status/HANDOFF.md). This is a native Rust library with
an unstable 0.1 alpha API. Edition 2024, Rust 1.85 minimum; CI checks stable and
1.85.0 on Linux, macOS and Windows. Python 3.12+ runs the documentation/package
tools; reference fixtures record the CPython version that generated them.

## Local verification

```sh
cargo fmt --all -- --check
cargo check --locked --all-targets
cargo clippy --locked --all-features --all-targets -- -D warnings
cargo test --locked --all-targets
cargo test --locked --all-features --all-targets
cargo test --locked --all-features --doc
RUSTDOCFLAGS="-D warnings" cargo doc --locked --all-features --no-deps
python3 tools/check_docs.py
python3 tools/check_package.py
cargo publish --dry-run --locked --all-features
```

The package checker builds the actual Cargo archive, checks the default dependency
graph, and runs fresh downstream consumers with default and all optional features.
It uses cached dependencies offline; run `cargo fetch --locked` first on a fresh
checkout. Commit Cargo.lock. Ordinary checks use synthetic data, scripted providers,
and a localhost HTTP server; they do not need credentials or call hosted models.

Feature checks: `--no-default-features`, `--features tokio`, `--features images`,
`--features blocking`, `--features typesafe`, and `--all-features`. The blocking
facade and built-in HTTP transport require a configured Tokio runtime as documented
in rustdoc. The default provider API works with a caller-selected executor.

## Examples

Run `cargo run --example hello`, `triage`, `filter`, or `rerank`. The image example
is `cargo run --example images --features images`. Every example uses a clearly
labeled synthetic provider. They demonstrate library behavior, not inference.

## Reference fixtures

Four exporters live under tools/: `export_python_fixtures.py`,
`export_core_edges.py`, `export_recipe_fixtures.py`, and `export_bench_fixtures.py`.
Use their `--help` for source/output arguments. They verify the pinned source
revision and hashes and make no provider calls. Fixtures retain generator hashes;
the documentation checker catches stale provenance. The Python 3.13 reference
includes Python 3.12+ compensated summation behavior; changing that baseline is a
compatibility decision. Never derive expected parity values from Rust output.

## Change discipline

Add behavior tests for numeric boundaries, invalid inputs, ordering, cancellation,
and actual integration boundaries. Keep constructors fallible and normalize through
the central answer implementation. Record deliberate reference differences in
[COMPATIBILITY](COMPATIBILITY.md). Keep runtime/network dependencies optional.
Update source-facing API examples and package checks when exports change.

Update status, handoff_version, validation and cumulative synthesis after material
work. Commit/push authorized changes and refresh the active-project pointer. Live
qualification and publication follow their own separately scoped work; no keys or
private input bodies belong in fixtures, logs or repository history.

## Standalone consumer integration

The independent manifest in integrations/triage-app consumes only public library
APIs and has its own lockfile. `python3 tools/check_triage_app.py` builds it and
runs synthetic end-to-end cases through localhost HTTP. Use `--skip-build` to check
an existing `target/triage-app/debug/semops-triage` binary (platform suffix applied).
CI compiles/lints and exercises this consumer on all supported OS/toolchain pairs.
The script strips proxy environment settings and uses only a synthetic bearer token.
For app input/output and failure semantics, see
[its README](../integrations/triage-app/README.md).

## Bounded provider-check tooling

Run `python3 -m unittest discover -s tests -p 'test_triage_*.py'` for the offline
runner/scorer tests. Child processes are mocked; the scorer uses synthetic outputs.
CI runs these on all supported OS/toolchain pairs. The
[qualification packet](../qualification/README.md) documents planning, explicit
execution and evidence interpretation. Ordinary tests never run hosted inference.
