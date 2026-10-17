# Development plan

Implementation candidate: 0.1.0-alpha.1. M1–M6 are implemented and verified offline and by hosted platform CI. Requirement evidence is mapped in
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
