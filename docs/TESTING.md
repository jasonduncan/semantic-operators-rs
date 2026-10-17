# Verification and qualification plan

## Evidence levels

| Level | Proves | Does not prove |
|---|---|---|
| Bootstrap checks | Manifest, compilation, formatting, documentation links | Semantic functionality |
| Offline Rust unit/property tests | Local validation and execution invariants | Provider compatibility without transport tests |
| Python differential fixtures | Agreement with the frozen reference cases | Completeness or real model quality |
| Scripted transport tests | Exact request/response translation and error handling | Hosted service behavior |
| Bounded live smoke | One authorized service/model/configuration works | General accuracy, calibration, production readiness |
| Independent labeled evaluation | Measured performance on that dataset/split | Generalization outside the measured task |

## Required case families

- Questions: blank instructions; duplicate/empty labels; finite/nonfinite/out-of-range
  thresholds; order and Unicode; Bool/number type distinctions; deserialization bypass.
- Answers: exact/partial/mismatched distributions; NaN/inf; sum margin boundaries;
  ties and threshold equality; complemented 0.07 -> 0.93; score expected-value
  tolerance, half-level confidence, near-boundary rounding; Box missing confidence,
  invalid edges, empty distribution. Raw and metadata survive abstention.
- Providers: response coverage; object-safe trait usage; concurrent asks; validation
  before dispatch; invocation count; futures lazy until polling; cancellation drops
  pending work; no callback/detached task after cancellation; redacted errors.
- Wrappers: two or more hops; subset ordering; per-answer call identity; last-hop
  uncertainty; invalid threshold; no escalation after errors; nested timeout order.
  Use a deterministic clock/runtime, not flaky wall-clock sleeps.
- Flows: multi-question single call, dependent second call, unused undecided answer,
  require stops, handled uncertainty, latched error, trace on every exit.
- Recipes: empty inputs/no calls; duplicate IDs; capacity=1/8; peak concurrency;
  arbitrary completion order; input-order filter outcomes; stable rerank ties;
  errors unscored; normalization of reweighted expectations; invalid weights.
- Bench: coverage/conditional accuracy/failures, expected-score-to-level conversion,
  abstention labels, zero denominators, NDCG empty/zero ideal and stability cases.
  Compare complete reports against Python, not just aggregate headline accuracy.
- Wire: all request types; absent/null request_id; rejected extras; exact error paths;
  4-significant-digit floats; bool stays bool; mixed-call projection; Box rejected.
- Images: valid boundaries, malformed/truncated bytes, APNG, JPEG warnings, reused
  handles, missing handles, repeated-use total, asset order, frames timestamp shape,
  presence false/unknown/true with/without usable Box. Use synthetic tiny fixtures
  with dimensions at the reference limits, not private photos.

## Fixture scope at bootstrap

The checked-in JSON contains normalization, complement, and wire decode cases
computed by the pinned Python implementation, with revision and generator hash.
It is a seed corpus, not an exhaustive parity suite. There is no Rust consumer
yet. Add wire encode, TypeSafe payload, concurrency, recipes, and metrics fixtures
at their respective milestones. Keep expected numeric values literal and record
the rule for comparisons per case family; never apply one broad epsilon to all
assertions because it can hide different decided/undecided results.

## Release gates

Linux/macOS/Windows on stable and MSRV; all supported features; docs compile;
downstream example compiles; package contents reviewed; default dependency tree
has no transport/runtime; no unexpected network activity in ordinary tests.
M6 requires a complete requirement/fixture mapping. Optional live verification
must state model, synthetic dataset, invocation limit, retry policy, observed
outcomes and cost if reported. Do not infer quality from transport success.

