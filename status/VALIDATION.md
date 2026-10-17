# Validation evidence

2026-10-17, macOS aarch64, rustc 1.98.1 / cargo 1.98.1.
Implementation snapshot: bootstrap source from initial commit 4cf2497; subsequent
handoff documentation does not add semantic runtime code.

## Local bootstrap

- cargo check --locked --offline --all-targets: passed.
- cargo clippy --locked --offline --all-targets -- -D warnings: passed.
- cargo test --locked --offline --all-targets: passed with **0 tests**.
- cargo test --locked --offline --doc: passed with **0 doctests**.
- cargo doc --locked --offline --no-deps: passed.
- The first fmt check found a trailing blank line; cargo fmt corrected it.
  Final cargo fmt --all -- --check: passed.

## Reference evidence

The pinned clean Python checkout generated 23 answer cases, 5 complement cases,
10 wire decode cases. Fixture file records exact source revision and generator
hash. These are Python outputs, not Rust parity results. Full Python/C++ test
suites and live providers were not run.

## External state and limits

The private GitHub repository was created. CI covers Linux/macOS/Windows on stable
and 1.85.0, plus documentation checks; configuration alone is not a passing run.
No semantic implementation, Rust parity reader, live smoke, performance benchmark,
model qualification, package publication or crate reservation exists. Crates.io
API lookup returned 403 and remains inconclusive. Future release gates are open.

## Handoff checks before push

- Documentation checker: 22 Markdown files and 39 local file links resolve;
  fixture revision and exporter hash match. This is file-link validation, not
  Markdown anchor or external-URL availability checking.
- Status manifest validate --git: valid; pre-push dirty/unpushed state expected.
- GitHub repository visibility readback: PRIVATE.
- cargo package --list --allow-dirty: inspected. Anchored include patterns exclude
  status/docs/fixtures and retain only Cargo metadata/lock, README, LICENSE and src.
  This is a contents check, not a published or full package verification.
- New local research dossier/topic links resolve.

Hosted CI and final clean/pushed readback are checked after the first push and
recorded in a subsequent evidence update; no earlier claim of a passing run.
