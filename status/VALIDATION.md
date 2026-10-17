# Validation evidence

## Implementation candidate — 2026-10-17

0.1.0-alpha.1, macOS aarch64. Rust 1.98.1 and Rust 1.85.0 minimum toolchain.
These checks exercise implemented library behavior; they do not qualify model quality.

| Check | Observed result |
|---|---|
| cargo fmt --all -- --check | pass |
| cargo clippy --locked --offline --all-features --all-targets -- -D warnings | pass |
| cargo test --locked --offline --all-targets | pass, default core and examples compile |
| cargo test --locked --offline --all-features --all-targets | pass, 86 Rust tests |
| cargo +1.85.0 test --locked --offline --all-features --all-targets | pass, 86 Rust tests on final implementation |
| cargo test --locked --offline --all-features --doc | pass, 1 compiled package example |
| RUSTDOCFLAGS=-Dwarnings cargo doc --locked --offline --all-features --no-deps | pass |
| python3 tools/check_package.py | pass: 31-file actual archive, default graph isolation, independent default/all-feature consumers |
| python3 tools/check_docs.py | pass: local file links and all 4 generator/revision provenance records |
| Five offline examples | pass; actual image example run with images feature |

The differential fixtures contain 213 expected scenarios: 172 core/wire, 25 recipe, 
16 benchmark reports/scenarios. CPython 3.13.13 generated expectations from pinned
Python source; Rust consumes them. This is finite differential evidence, not a
claim of exhaustive equivalence. Full sibling Python/C++ suites were not rerun.

Review regressions cover never-restored abstentions, Python special whitespace,
redacted Debug, float roundtrip and compensated sums, extreme-weight overflow,
disabled Tokio drivers, cancellation, bounded multipart collision search, and
per-case flow validation retaining failure traces. TypeSafe evidence comprises
13 integration tests plus 2 internal multipart tests; requests use synthetic
scripted transports and an ephemeral localhost HTTP server. Initial sandbox
socket denials required running those tests outside the sandbox; that rerun passed.
Lint found two test MutexGuard lifetimes across awaits; lexical scopes fixed them.
Rustdoc found one numeric interval mistaken for a link; code formatting fixed it.

The package intentionally excludes private status, source manifests, fixtures and
development docs; it includes the standalone CRATE.md, source, license, Cargo
metadata and examples. Cargo warns about excluded integration tests during archive
creation; consumers build the extracted package successfully. No crate published.

Hosted CI for this implementation is pending the first implementation push.
It covers 6 OS/toolchain jobs (Linux/macOS/Windows, stable/1.85.0), documentation,
and individual-feature/package verification. Results will be appended with exact
commit/run links before closing the candidate handoff.

No live model calls, paid evaluation, real-document upload, public visibility,
release tag, registry publication or name reservation. Image tests parse headers;
full decoding and service acceptance remain unqualified. Performance unmeasured.

## Earlier setup evidence (historical)


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
