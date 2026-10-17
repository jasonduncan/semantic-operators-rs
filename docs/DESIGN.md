# Rust library design

Working design v0.1 · 2026-10-17 · implementation pending.
The project and specification were requested by Jason. Engineering choices below
are the initial working design, not claims of user review or API stability.

## 1. Product contract

An application defines provider-independent questions and receives validated,
typed answers with probabilities, confidence, and reported call metadata. It can
reuse those definitions across providers, batch questions about one state, compose
ordinary Rust control flow, and compare results with labeled offline benchmarks.

Target users are Rust service, command-line, and desktop developers who would
otherwise reimplement Python semops semantics or cross an FFI boundary. Value is
behavioral portability and idiomatic integration; performance and adoption benefits
are hypotheses until measured. This is a library, not an agent framework.

Requirements:

| ID | Requirement | Acceptance |
|---|---|---|
| R1 | Boolean, Choice, Score, Box; validated questions and responses | Frozen normalization fixtures plus Rust edge cases |
| R2 | Named, typed operators and heterogeneous one-call apply | Compile tests and a scripted provider showing one invocation |
| R3 | Async provider abstraction with optional TypeSafe adapter | Contract tests; no HTTP/Tokio in default dependency tree |
| R4 | Explicit undecided answers, timeout errors, cascade | Deterministic failure/cancellation tests |
| R5 | Plain Rust flows with full trace and explicit uncertainty propagation | Unused uncertainty, read uncertainty, handled uncertainty, provider failure |
| R6 | Filtering, reranking, reweighting, bounded per-item concurrency | Stable ordering and all item outcomes accounted for |
| R7 | Evaluation: coverage, conditional accuracy, failures, NDCG, stability | Match reference metrics on fixed reports |
| R8 | Python text wire format schema 1 | Valid/invalid golden requests and encoded responses |
| R9 | Images, Region, frames, locate with explicit assets | Offline bytes/limits/multipart fixtures; separate live qualification |
| R10 | Reusable Rust package and optional integrations | Consumer/package checks and platform/MSRV CI |

The first complete 0.1 candidate includes R1–R10. Milestones deliver text first,
then recipes, transport, and images; the initial scaffold is not an MVP claim.
CLI/MCP, native Laya inference, dynamic choice construction at call time, Rank,
caching, multi-state server batching, graph languages, FFI, WASM and no_std are
deferred. Reranking is independent Score calls, not a native Rank question.

## 2. Architecture

One Cargo library initially; split packages only if dependency pressure warrants it.
No native linkage to libsemop or embedding Python. Planned core dependencies:
`serde` and `serde_json` with order preservation; an ordered-map implementation
if needed by the public collections. Exact versions are selected and locked in M1.
The bootstrap deliberately installs none of these yet.

| Module | Responsibility | Must not own |
|---|---|---|
| `question` | Validated Boolean/Choice/Score/Box and ordered names | Provider SDK types |
| `answer` | Typed/dynamic answers, one normalization implementation, Call | Transport and inference |
| `state` | Text/object/array state; request-local assets | Hidden process-wide image registry |
| `provider` | Object-safe async contract, request/response validation | Runtime creation, implicit retries |
| `operator` | Typed named questions and heterogeneous apply | Provider configuration |
| `cascade` | Re-ask uncertain subset, preserve metadata/order | Error failover |
| `flow` | Ask context, typed reads, stop/error trace | Graph DSL or background scheduler |
| `each`, `filter`, `rerank` | Bounded per-item calls and stable results | File I/O or automatic text truncation |
| `bench` | Pure report scoring and optional runner | Automatic paid evaluation |
| `wire` | Strict Python schema-1 text translation | Provider selection, image schema extension |
| `image`, `locate` | Explicit image handles, Region helpers, presence + box | Automatic upload or image correction |
| `providers::typesafe` | Optional request translation and transport | Answer decision rules |

Proposed features: default empty; `tokio` for timeout helpers, `typesafe` for the
HTTP adapter (implies Tokio and a selected HTTP client), `images` for asset parsing,
`blocking` for an explicit owned-runtime facade (implies Tokio). Features are not
declared until implementation exists. Recipes use a runtime-neutral futures
combinator library if needed; they must not spawn a Tokio task just to poll work.

## 3. Data and invariants

Question builders return `Result`; validated fields are private. Deserialization
must use those validators. Ordered unique question names, choices, score levels,
and response entries are part of the contract. JSON object insertion order is
retained when constructing provider state; never sort semantic inputs silently.

`Answer<T>` has `Option<T>` value, ordered probabilities, finite confidence,
optional `Arc<Call>`, and optional raw JSON. `false`, score zero, and an empty
application-level value are not undecided. `Call` reports provider/model/usage;
unknown metadata stays `None`, not zero or the configured model alias. Box's
unreported confidence remains zero specifically to match Python.

The dynamic boundary uses `Question` and `AnswerValue` enums. Typed operators map
Boolean to bool, Choice to String (optional enum helper later), Score to f64, Box
to Region. Typed conversion checks the question variant; it cannot cast silently.
Both typed and dynamic APIs share normalization. Provider outputs must cover the
requested names exactly; missing/extra responses are protocol errors.

Errors distinguish invalid question, invalid argument/state/image, invalid wire
request with field path, malformed provider response, provider failure, timeout,
unsupported capability, and runtime misuse. Errors implement standard Error and
Display with a source where safe. No panic for caller/provider input; programming
bugs remain bugs, not synthetic abstentions. Do not expose request bodies or keys
through error formatting or Debug by default.

## 4. Execution and ownership

Async is canonical. `Provider: Send + Sync` returns a boxed Send future borrowing
the provider and request; it is usable through `Arc<dyn Provider>`. Calling the
method must not start network I/O before the future is polled. Caller drives the
executor; the library does not create global threads or runtimes. See [API](API.md).

Dropping a request future stops local polling and releases owned work. It does
not prove remote cancellation or non-billing. No detached retries/tasks. Timeout
and cascade compose as providers. A whole-cascade deadline differs from per-hop
deadlines and is documented/tested. Wrappers propagate transport failures.

Blocking support is a later explicit facade for sync callers. It owns a runtime
and rejects calls from an active Tokio runtime before blocking. Do not try to
reproduce C++ `start/pump/cancel` as the Rust public model.

## 5. Composition and evaluation

Ask independent questions together in one request. A second flow call is useful
when its state/question/provider depends on an earlier answer. A flow returns
`Result<FlowOutcome<T>, Error>`; reading an undecided answer through `require`
produces a structured stop containing the operator and answer, while `peek`
permits explicit handling. An unread undecided result does not stop execution.
Provider failures propagate separately; traces survive failed execution for
inspection and are never logged automatically.

Filtering classifies each item as match/no/unsure/failed. Reranking collects
decided scores, keeps others as unscored, sorts descending with exact ties in
input order, and allows finite strictly increasing reweights without new calls.
Never place all candidates into one shared model state as an optimization.

Confidence is model-reported evidence, not calibrated probability of correctness.
Report coverage with accuracy, and failures separately from uncertainty. Keep
offline conformance, live transport smoke, model quality, and release acceptance
as distinct evidence. No performance or quality claims in 0.1 without measurements.

## 6. Main risks and decisions still to test

- Numeric parity: Python fsum, ties-to-even rounding, and significant digits need
  differential tests rather than naive Rust round substitutions.
- SDK choice: no Rust TypeSafe SDK was verified in this setup. First implement a
  scripted transport; select SDK reuse versus narrow HTTP adapter during M4.
- Image limits and model capability are a dated sibling-source contract, not a
  verified current service promise. Refresh official provider documentation in M5.
- Box support exceeds Python's text wire schema. Keep wire v1 text-only.
- A two-crate core/provider split may become useful later; a single feature-gated
  crate is the smallest initial implementation.
- Async flow lifetime ergonomics need a compiling prototype in M3 before freezing
  convenience functions or macros. The explicit context API is the fallback.

