# Implementation and requirement coverage

2026-10-17 · 0.1.0-alpha.1 · native Rust, unpublished.
This map links the original R1–R10 contract to shipped source and offline evidence.
Executed checks and hosted CI receipts belong in [VALIDATION](../status/VALIDATION.md).

| Requirement | Implementation | Evidence |
|---|---|---|
| R1 validated questions/answers | question.rs, answer.rs, state.rs, error.rs | core_parity.rs; 172 combined core/wire Python cases; false/zero, ties, thresholds, compensated sums, invalid inputs, serde validation |
| R2 named typed operators/apply | operator.rs, provider.rs | execution.rs; independent questions batched once, duplicates rejected before I/O, typed reads |
| R3 async providers/TypeSafe | provider.rs, providers/typesafe.rs | provider_edges.rs, typesafe.rs; trait objects/borrowing closures, runtime guards, scripted and localhost HTTP; package default dependency isolation |
| R4 uncertainty/cascade/deadlines | answer.rs, cascade.rs | execution.rs, provider_edges.rs; subset escalation, shared metadata, errors propagate, deterministic deadline and drop tests |
| R5 traced Rust flows | flow.rs | execution.rs; unused/read/handled uncertainty, latched errors, all attempted calls retained |
| R6 per-item recipes/ranking | each.rs, filter.rs, rerank.rs | recipes.rs, recipe_parity.rs, review_edges.rs; ordering, bounded concurrency, stable ties, partial/mixed-model handling, overflow and cancellation |
| R7 evaluation | bench.rs | bench.rs integration suite + python-bench.json; full reports, subset labels, zero denominators, threshold sweeps, half-up score decisions, stability, NDCG, flow reports, warmup accounting |
| R8 text wire v1 | wire.rs | wire_contract.rs and Python fixture reader; strict request shape, errors, 4-significant-digit encoding, call projection, Box rejection |
| R9 images/locate | image.rs, locate.rs, adapter multipart | images.rs, recipe_parity.rs, typesafe.rs + internal multipart tests; limits, diagnostics, references, dedup, frame order, presence/region decisions |
| R10 Rust package | Cargo features, blocking.rs, CRATE.md, CI | compiled examples/doctest, feature matrix, MSRV/platform CI, tools/check_package.py archive extraction and fresh consumers |

Source module paths are under src/; integration suites are under tests/.
The 25 recipe reference cases cover six filters, ten rankings/reweights, four frame
states and five Region transformations. The benchmark corpus contains 16 aggregate
reports/scenarios, including five threshold reports and seven NDCG cases. Combined
with 172 core/wire cases, the differential corpus contains 213 expected scenarios;
a single Rust test often checks many scenarios. Test function counts differ from
reference case counts and do not measure model quality.

## Supported surfaces and evidence limits

- Default core has no runtime/network/TLS dependency. Tokio deadlines, blocking,
  TypeSafe HTTP, and image helpers are separately enabled features.
- All five examples run against synthetic deterministic providers. They are
  executable usage examples, not claims about real inference.
- TypeSafe transport evidence is request/response mapping, explicit credential
  handling, limits, cancellation and local HTTP behavior. No hosted model was
  called. Current service rules and frozen-core differences are in
  [TYPE_SAFE_TRANSPORT](TYPE_SAFE_TRANSPORT.md) and [IMAGES](IMAGES.md).
- Image validation parses PNG/JPEG headers and enforces reference bounds. It does
  not decode pixel data or prove that a service accepts a particular encoding.
- Wire v1 is text-only and intentionally retains first-call metadata projection.
- Evaluation is opt-in: bench::run invokes the supplied provider and defaults to
  one extra warmup call. Pure scoring, threshold sweeps and reweighting make no
  provider calls. Performance and semantic quality require a measured dataset.
- CLI/MCP, native local inference, native Rank, caches, FFI, WASM and no_std remain
  deferred. Publication and live qualification are separate from this build.

## M7 consumer integration

integrations/triage-app is an independent Rust JSON Lines application using the
public TypeSafe adapter and typed operators. It preserves confidence/probabilities/
reported call metadata and distinguishes review from execution failure. The
subprocess/HTTP harness tools/check_triage_app.py verifies the complete consumer
path. This adds consumer evidence without modifying core semantics or qualifying a
hosted model. It is excluded from the library's Cargo archive.
