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

Initial clean/pushed registry check passed at 6e257e5 after the first push.

## CI configuration refresh

Initial hosted run flagged deprecated Node 20 action versions. Official GitHub
release API readback on 2026-10-17 identified actions/checkout v7.0.1 and
actions/setup-python v7.0.0; workflow updated to those exact tags. A new hosted
run verifies the revised workflow. Added explicit LF attributes for portable
source/fixture hashing. Seed fixture regeneration was byte-identical and both
reference source checkouts remained clean.

## Observed hosted results

- [Initial run 39064650863](https://github.com/jasonduncan/semantic-operators-rs/actions/runs/39064650863)
  at 6e257e502e04f1b43614a9e5fb2276508238fffc: all seven jobs passed.
- [Updated run 39064742276](https://github.com/jasonduncan/semantic-operators-rs/actions/runs/39064742276)
  at 0363816f09a44d79ee30e320110f8b1b4b316f40: all seven jobs passed with current
  action versions. This covers Linux, macOS, Windows on stable and 1.85.0 plus
  the documentation job. Rust suites contain zero tests; no semantic qualification.
- The closing commit changes documentation/status only. The Rust source, Cargo
  manifest, fixtures and CI workflow match the updated passing run; local document
  checks are repeated after those text edits. Do not imply hosted CI ran on a
  later documentation commit before its actual result is observed.
- Official crates.io Git index cross-check: both hyphen/underscore entries absent,
  known serde entry accessible. Name appears unregistered; not reserved.
