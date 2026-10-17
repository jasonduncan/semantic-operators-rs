# Validation evidence

## M7 standalone consumer — 2026-10-17

The library source, frozen tests and Cargo package are unchanged. Added an
independent JSONL triage application, its Cargo.lock, a subprocess/HTTP harness,
and consumer checks to every OS/toolchain CI job.

- Stable and Rust 1.85 consumer builds/Clippy with warnings denied passed locally.
- 32 JSONL/HTTP scenarios passed on the Rust 1.85 binary, including decided and
  review outcomes, exact threshold equality, normalized evidence, reported call
  metadata, call count, partial output on fatal errors, preflight rejection and
  input size/budget boundaries. Windows runs 31 scenarios, excluding the Unix-only invalid-argv check.
- The initial local socket bind was denied by the sandbox. The approved localhost
  rerun passed. Tests use synthetic responses and a fixed fake credential, with
  inherited proxy settings removed. No hosted model calls.
- Independent review found a non-UTF8 argv panic; generic fallible argument parsing
  now gives exit 2. The app's remote opt-in is a configuration guard, not a network
  sandbox; its inherited proxy behavior is documented.
- Documentation links/provenance and formatting pass. Cargo package --list still
  contains 31 files, excluding the integration application and private status.
- Hosted consumer CI passed as recorded below. Prior alpha CI on the exact
  previous handoff commit c8a64d7 passed all eight jobs in
  [run 39068393148](https://github.com/jasonduncan/semantic-operators-rs/actions/runs/39068393148).


### M7 hosted receipt

[Run 39074395130](https://github.com/jasonduncan/semantic-operators-rs/actions/runs/39074395130)
passed all eight jobs at feb00f2ae3d58f9b75b4c98ef8e2a21aa1a5995c.
Each Linux/macOS/Windows stable/MSRV job now also formats, lints, builds and runs
the standalone consumer through its local HTTP harness. Unix runs 32 scenarios;
Windows runs 31, omitting the non-UTF8 Unix argv case. Existing library suites,
feature matrix and packaged consumers remain green.

Final closing edits only update handoff/status; app, harness, locks, library and
workflow match the passing commit. Local docs/manifest checks are repeated after
closing. No live provider/model evidence is implied by this test receipt.

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

Initial hosted run [39067871282](https://github.com/jasonduncan/semantic-operators-rs/actions/runs/39067871282)
at 78e18a5 passed documentation and the individual-feature/package job. Matrix
Clippy stopped on an older format-collection lint and a newer atomic-method
deprecation. The test builder now uses write!; the atomic call retains the MSRV
API with a narrow documented deprecation allowance. Local MSRV Clippy and relevant
tests pass after these corrections; the corrected hosted run passed as recorded below.
It covers 6 OS/toolchain jobs (Linux/macOS/Windows, stable/1.85.0), documentation,
and individual-feature/package verification. The final commit/run receipt is recorded below.

No live model calls, paid evaluation, real-document upload, public visibility,
release tag, registry publication or name reservation. Image tests parse headers;
full decoding and service acceptance remain unqualified. Performance unmeasured.

## Final candidate CI receipt

[Run 39068153607](https://github.com/jasonduncan/semantic-operators-rs/actions/runs/39068153607)
passed all eight jobs at b92ddf053282a6f9869f401008a8a2a7bfc099fb:
Linux, macOS and Windows each on stable and 1.85.0; documentation; individual
features and packaged consumers. Each platform checks formatting, compilation,
Clippy, default/all-feature tests, doctests and rustdoc. M1–M6 gates are complete.

The source siblings were rechecked clean at their pinned revisions. Final closing
changes touch documentation/status only; source, fixtures, dependencies and workflow
match this passing CI commit. Local documentation/manifest checks are repeated for
the closing commit. This distinction avoids claiming CI executed a later docs commit.

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

## Bounded live-check preparation — 2026-10-17

- `python3 -m unittest discover -s tests -p 'test_triage_*.py'`: 19 passed locally.
  Ten scorer tests cover metric partitions, null denominators, strict malformed
  evidence, policy consistency, shared usage and exact file hashes. Nine runner
  tests cover planning, budgets/bytes, source/dataset drift, key omission, complete/
  partial/malformed runs, process launch failure and timeout/interruption cleanup.
  Subprocesses are mocked; these are tool correctness checks, not provider outcomes.
- Default `python3 tools/run_triage_check.py`: plan only; 12 cases, 1722 input bytes,
  requested_model/max_calls null, ready_to_invoke false; no live artifacts or calls.
- Frozen dataset SHA-256:
  cf7d47ba462d56c647431f1a7881fcf316500cb509d354259506847004650fea.
- Documentation/file-link/provenance checker and git diff whitespace check passed.
- Existing closing HEAD 4735d9923d8b7b924af1ce046a64c701f99ba365 was verified clean,
  matching remote main, with all eight jobs passed in
  [CI 39074703366](https://github.com/jasonduncan/semantic-operators-rs/actions/runs/39074703366).
- New workflow adds the Python tests to each OS/toolchain job. Hosted results for
  this preparation change will be recorded separately after push.

No hosted inference occurred. The model and call ceiling remain unset. Synthetic
labels are assistant-authored and unreviewed; ambiguity probes are exploratory.

### Preparation CI result and local correction

[CI 39076027736](https://github.com/jasonduncan/semantic-operators-rs/actions/runs/39076027736)
at c78bd05078ad3e3f6af1a7ccaf2e70025fec71cb completed: six jobs passed, both Windows
jobs failed only the new scorer Unicode-fixture test. All Rust and consumer checks
passed. Python 3.12 on Windows used cp1252 for a test's Path.write_text; corrected
that fixture to specify encoding="utf-8". All 19 tests pass locally after the fix.
The corrected test has NOT yet run in hosted Windows CI.

Explicit GitHub repo/API readbacks after the preparation push returned PUBLIC,
contrary to the prior PRIVATE record. No visibility mutation was performed. Jason
was asked whether public is intentional or private should be restored. The UTF-8
fix and corrected status/CI receipt are committed locally but further pushes are
held pending that answer. No hosted model calls were made.
