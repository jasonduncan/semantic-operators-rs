# Development plan

Published version: 0.1.0-alpha.2. M1–M7 and subsequent best-practices maintenance are implemented, verified and released. Historical phase records follow. Requirement evidence is mapped in
[IMPLEMENTATION](../docs/IMPLEMENTATION.md).

Original milestone scope and acceptance criteria follow.

Working scope: native Rust counterpart to the current Python library, including
images and Box, with optional TypeSafe transport. No deadline or effort estimate
is promised. Each phase has an offline exit before any optional live qualification.

## M0 — specification and project bootstrap (complete)

Deliver source inventory, compatibility contract, proposed API and architecture,
Cargo skeleton, license, contributor guidance, reference fixture generator, CI,
private repository, and resumable status. Verify bootstrap build/docs and push.
No semantic implementation claim.

## M1 — core values and normalization (complete)

Implement error types, ordered collections, State, validated Boolean/Choice/Score/
Box and Region, Answer/Call, complement and central normalization. Select minimal
serde/order dependencies compatible with MSRV. Add a Rust reader for the 38 seed
fixtures; expand invalid-question, nonfinite, tolerance and rounding boundaries.

Exit: all fixtures match; false/zero remain decided; ties and threshold equality
match; private fields and deserialization cannot bypass validation; no provider,
network or runtime dependency in default cargo tree. Region valid inputs match
Python, with documented Rust convenience validation. Compile basic API examples.

## M2 — operators and provider execution (complete)

Implement Request/Answers validation, dyn-safe Provider, typed Operator and dynamic
apply, scripted provider, cascade, and optional Tokio timeout wrapper. Add Send/Sync
and future borrowing compile checks, lazy dispatch, concurrency/cancellation tests.

Exit: independent questions use one call; every requested name exactly once; bad
outputs fail; typed/dynamic decisions match; uncertain-only escalation; no failover
on errors; all deadline compositions and metadata preservation verified offline.

## M3 — flows, per-item recipes, evaluation (complete)

Implement FlowContext and explicit require/peek/finish, trace retention and latched
errors. Add ask_each, filter_items, rerank/top/reweighted with bounded concurrency.
Port pure benchmark scoring, threshold sweeps, NDCG, stability, and flow reports.
Keep reference metric denominators and Box exclusion; document extensions.

Exit: synthetic triage/filter/rerank examples run without credentials; arbitrary
completion order preserves output order; no dropped/duplicated items; exact ties
stable; no rerun for reweighting or threshold changes; complete fixed reports match
Python. Trace/error and unused/handled/read uncertainty cases pass.

## M4 — text transport and wire compatibility (complete)

Inspect current official TypeSafe contract and available Rust SDKs; record the
SDK-versus-HTTP decision. Implement optional TypeSafe text adapter with injectable
transport and no implicit retries. Implement wire schema 1 strictly, including
4-significant-digit encoding and known first-call projection for mixed calls.

Exit: request/response/error golden cases, null criteria side, index-to-level
mapping, usage validation, malformed/partial responses, auth/rate-limit/timeout
handling, text wire decode/encode parity, and no secrets in diagnostics. No live
calls required for this exit. A separately authorized bounded smoke is optional.

## M5 — image parity and localization (complete)

Implement image validation/diagnostics, explicit asset registry, frames, multipart,
Box adapter, Region helpers and locate. Refresh provider capability/limit evidence.
Select reviewed parsing dependencies; do not implement an unbounded image decoder.

Exit: reference dimension/area/byte boundaries, repeated-use totals, deduplication,
first-use ordering, stale/missing handles, malformed images, JPEG warnings,
frames shape and all locate outcomes pass offline. Wire v1 still rejects Box.
Live image behavior remains separately qualified with synthetic public-safe assets.

## M6 — package and release candidate (complete)

Add explicit blocking facade only if the preceding API warrants it; reject nested
runtime misuse. Complete feature matrix, downstream-consumer checks, cargo package
inspection, compiled examples, rustdoc, license review and requirement coverage.
Verify stable/MSRV on Linux/macOS/Windows and update public API docs to actual code.

Exit: reviewable 0.1 candidate with source/parity/CI evidence and documented limits.
Publication is a separate owner-authorized step in [RELEASE](../docs/RELEASE.md).
Crate name availability and package contents must be rechecked then.

## Deferred scope

CLI/MCP (separate binary package if selected), Laya local Python/torch inference,
call-time Choice convenience API, native Rank, caching, server-side multi-state
batching, graph language, no_std/WASM and C ABI. Do not turn the deferred list into
implicit implementation authorization or reduce R1–R10 to the bootstrap surface.

## M7 — standalone consumer integration (complete)

On “continue”, select a self-contained JSON Lines triage application consuming only
public library APIs. Keep its manifest/lockfile independent. Batch department and
urgency questions once per ticket, preserve answer evidence, route uncertainty to
review, and stop on execution failure. Validate the whole finite input batch and
explicit call limit before I/O; flush successful rows before another request.

Exit: compile/lint on stable and MSRV, real subprocess/localhost HTTP scenarios for
routing, uncertainty, threshold equality, provenance, call budget, malformed input,
partial output and fatal errors. Include portable CI and concrete usage docs.
The application is a reference consumer; generic library CLI/MCP remains deferred.

Next after M7: select an existing application integration or a bounded live-provider
qualification brief. Publication still needs a crate-name recheck, API/release review
and explicit publication direction.

M7 acceptance: 32 local scenarios passed; hosted run 39074395130 passed all eight
jobs at feb00f2, including independent app lint/build/HTTP checks on all six
OS/toolchain pairs. Windows runs 31 cases, excluding Unix non-UTF8 argv.

## Bounded live-check preparation (complete; execution scope pending)

On continuation, prepared qualification/triage-v1.json (12 synthetic tickets),
explicit runner, strict offline scorer and reproducible artifact retention. The
10 clear labels are assistant-authored; 2 ambiguity probes are exploratory. Freeze
confidence thresholds and route policy before outcomes. Local runner/scorer tests
pass 19 cases, covering partial/error/timeout/interruption evidence and metric
partitioning. CI includes those checks on all six OS/toolchain combinations.

Execution is blocked on Jason specifying the exact TypeSafe model and call ceiling.
Proposed complete run: 12 requests, no retries/warm-up, 30-second per-request timeout.
Once scoped, invoke once, retain outputs/hashes/receipts, report descriptive outcomes
and actual unknowns. No model acceptance, cost prediction or publication is implied.

Preparation platform gate: CI 39076027736 passed six jobs; Windows caught an implicit
fixture encoding. Fixed locally to UTF-8; 19 local tests pass. Corrected hosted
Windows result remains pending because further pushes are held on the unexpected
PUBLIC visibility readback. Resolve that state, push the retained fix, then verify.

## Public visibility resolved — 2026-10-17

Jason confirmed public visibility is intentional for crates.io distribution.
The visibility blocker and push hold are removed; no setting change is needed.
Proceed with the UTF-8 correction push and hosted verification. Cargo publication
remains disabled during release preparation; no package upload or live call occurred.

## Public candidate verification complete — 2026-10-17

[CI 39079995335](https://github.com/jasonduncan/semantic-operators-rs/actions/runs/39079995335) passed all eight jobs at
0c311756b9572fbf44a45c2ad4b3c8b669a11f99, including the corrected UTF-8 fixture on
Windows stable/1.85.0. Nineteen tool tests, existing Rust/consumer checks, individual
features and package consumers all pass. Local package audit independently passed
31-file archive/default dependency isolation/both extracted consumers. Crates.io API
and hyphen/underscore sparse entries returned 404; serde control returned 200.

Public distribution is intentional; no visibility blocker remains. Portfolio active,
execution ready for release preparation. Optional live-model scope is still unset,
not a package-engineering blocker. publish=false remains; no upload, live inference
or credential inspection occurred. Closing updates only change documentation/status;
source/tests/workflow match the passing run above.

## Crates.io alpha preparation — 2026-10-17

Retained candidate 0.1.0-alpha.1, added registry discovery/docs.rs metadata, packaged
CHANGELOG.md, and replaced publish=false with a crates.io-only allowlist for dry-run.
Package checker now rejects all unexpected archive paths. All optional features
will be built by docs.rs. No source API/behavior change; corrected a frame-timestamp
documentation mismatch found by independent review.

Local package/extracted consumer checks, one all-feature doctest, and online
`cargo publish --dry-run --locked --all-features` passed at 2640da883ee6b5100526d4052d55917bf50396a9.
The initial offline dry-run failed because registry metadata was unavailable;
updated CI/docs use the verified online command, which still suppresses upload.
32-file archive, 54439 bytes, SHA-256
`a7496cd603c3e9009b73d01647372e6172f9d8fe833ce4804b325d45e08d3c90`.
Logs/receipt are under ignored target/release-check. Final hosted CI is 39080731154,
result pending when this entry was written. Actual upload/version tag and registry/
docs.rs acceptance remain unperformed. Credential authentication is unverified.

## Final alpha-preparation CI receipt — 2026-10-17

[CI 39080731154](https://github.com/jasonduncan/semantic-operators-rs/actions/runs/39080731154)
passed all eight jobs at 2640da883ee6b5100526d4052d55917bf50396a9: Linux/macOS/Windows
stable and 1.85.0, documentation, feature/package checks and the new publish dry-run.
Preparation is complete. Portfolio active; execution ready for exact-version upload
direction. No registry upload, release tag or live model request occurred. Closing
edits record status/evidence only; tested package inputs and workflow are unchanged.

## Published alpha 0.1.0-alpha.1 — 2026-10-17

Jason explicitly directed publication. Updated release wording, committed
30b827deebc5f25e4b31a9a9843437488f79458e, reran the archive/downstream checks,
all-feature doctest and publish dry-run successfully, then ran cargo publish --locked.
Cargo reported Published; registry created_at is 2026-10-18T00:02:35.550910Z.
The registry, downloaded archive and local archive share SHA-256
89da5d2536113c2cc55b2a645a95e29da1411415f93d53cb7cd14c77a8c7c8a4 (32 files, 54434 bytes).
Downloaded .cargo_vcs_info.json matches the published commit. Pushed annotated
v0.1.0-alpha.1 at that revision. Fresh crates.io consumers passed with default and
optional TypeSafe/images/blocking features; no model invocation was involved.

docs.rs initially returned 404 while processing; subsequent main/TypeSafe/image/
blocking/TimeLimited pages returned 200. Existing Cargo credentials were used through
Cargo; no credential value was printed or added to artifacts. Release CI 39081286724
is completing; exact final result will be appended. No hosted inference or model
quality acceptance occurred. The immutable release is public and installable.

## Publication closed — 2026-10-17

[Release CI 39081286724](https://github.com/jasonduncan/semantic-operators-rs/actions/runs/39081286724)
passed all eight jobs at published revision 30b827deebc5f25e4b31a9a9843437488f79458e.
Remote annotated tag v0.1.0-alpha.1 dereferences to that same SHA. Registry package,
checksum, docs.rs optional APIs and fresh default/optional registry consumers are
verified. Publication is complete; portfolio remains active. Closing changes only
record documentation/status and do not alter the immutable published package.

## Rust best-practices review — 2026-10-17

The requested review adds cancellation safety, public error contracts/examples,
focused lints and automated dependency maintenance. Acceptance requires regression
suites, strict stable/MSRV checks, Miri core/wire tests, both advisory scans,
packaged consumers and hosted CI. Published alpha remains immutable. Full Unicode
repr diagnostic parity is deferred; next release version is not selected.

Review acceptance complete: local checks and all nine hosted jobs passed at
46cda9a. Follow-up scope is a selected issue or new release version; no further
work is required for this review. Unicode diagnostic parity remains deferred.

## Open-source readiness follow-up — 2026-10-17

Technical remediation and the 87-item assessment are complete; all ten hosted
jobs passed on the reviewed source. Owner decisions remain explicit in the assessment:
rights/trademark, contribution agreement, conduct contact, second owner/reviewer,
2FA and branch/trusted-publisher activation. A next release needs an exact new
version. The already-published alpha, previous milestones and deferred product scope
remain unchanged. No automatic publication or model qualification is implied.

## Alpha.2 publication complete — 2026-10-17

Jason authorized a new release. Version 0.1.0-alpha.2 is published and verified:
exact-source CI, registry archive identity, fresh default/optional consumers,
docs.rs and the annotated tag/GitHub prerelease all pass. No release work remains.
Next scope is a selected product issue or the owner/governance follow-ups above.
No live-model qualification or performance measurement was added.
