# Rust API guide

Implemented API, 2026-10-17. The complete examples under `examples/` compile and
run against explicitly synthetic providers. This replaces the bootstrap API sketches.

## Questions and typed operators

`Boolean::new(instructions)`, `Choice::new(instructions, IndexMap<String,
Option<String>>)`, `Score::new(instructions, Vec<String>)`, and
`BoxQuestion::new(instructions)` return Result. Each offers
`with_min_confidence(f64) -> Result<Self>`. Boolean additionally offers
`with_descriptions(Option<String>, Option<String>)`. Validated fields are private;
serde deserialization invokes the validators. Choice and score order is preserved.

`Operator::new(name, question)` binds a nonempty name. It owns no provider.
`op.ask(&provider, &state).await` returns `Answer<Q::Output>`: bool, String, f64 or
Region. `op.ask_request(&provider, &request)` retains image assets while replacing
questions. `op.erased()` produces a NamedQuestion for heterogeneous batches.

```rust,ignore
let questions = [complaint.erased(), urgency.erased()];
let answers = apply(&provider, &request, &questions).await?; // one semantic call
let complaint_answer: Answer<bool> = complaint.answer(&answers)?;
let urgency_answer: Answer<f64> = urgency.answer(&answers)?;
```

Duplicate names and empty batches fail before provider dispatch. Use
`operator::questions(&[NamedQuestion])` to construct a Questions map with duplicate
checking, or construct an ordered IndexMap directly if uniqueness is already known.

## State, requests and assets

`State::text(text)` accepts text; `State::try_from(serde_json::Value)` accepts text,
object or array at the top level. Nested JSON scalars remain allowed.
`State::as_value()` borrows the representation. State Debug hides all content.

`Request::new(state, Questions)` requires nonempty names and questions.
`request.with_questions(Questions)` clones state/assets and replaces questions.
With images enabled, `request.with_assets(Assets)` attaches an explicit registry.
Asset reference validation happens before dispatch, allowing assets to be attached
after construction. Request Debug shows counts rather than inputs or prompts.

## Provider contract

```rust,ignore
pub type ProviderFuture<'a> = Pin<Box<dyn Future<Output = Result<Answers>> + Send + 'a>>;
pub trait Provider: Send + Sync {
    fn ask<'a>(&'a self, request: &'a Request) -> ProviderFuture<'a>;
}
```

Use the free `ask(&provider, &request).await` function at call boundaries. It
checks assets before dispatch, then response coverage, answer variants,
distributions and requested thresholds, returning results in question order.
Custom providers return checked answers using `make_answer`. `Arc<dyn Provider>`
works directly. `FunctionProvider::new` wraps a function returning ProviderFuture;
no scripted or fake service is used unless the caller explicitly supplies it.

Async futures are lazy and borrowed: poll on the caller's executor, drop to cancel
local work. No core background tasks or automatic retries. Remote completion or
billing may continue after local cancellation.

## Answers, uncertainty, and errors

`Answer<T>` exposes `value() -> Option<&T>`, `decided()`, `confidence()`,
`probabilities()`, `call()`, and `raw()`. False and zero remain decided values.
`AnswerValue` is the dynamic Boolean/Choice/Score/Box enum. `answer.typed::<T>()`
checks the variant even for undecided answers. No unchecked Answer deserialization
is exposed.

`make_answer(&Question, AnswerValue, Probabilities, Option<f64>)` validates provider
output and applies ties/thresholds. Explicit confidence is for Box only.
`with_call(Arc<Call>)` and `with_raw(Value)` preserve optional evidence.
`with_min_confidence` and `abstain` only remove decisions. `revalidate` never
restores a prior abstention. All pathways share the same normalizer.

Call contains provider, optional reported model and optional integer usage. One
provider response can share an Arc; cascade results retain per-answer provenance.
Unknown usage/model stays absent. Box's absent confidence is zero for Python parity.

Error has a stable ErrorKind, message and optional wire path. Invalid inputs,
provider failures, timeouts, unsupported capabilities and malformed responses are
errors. Uncertainty is a successful Answer with no value. Debug and provider
error formatting omit private inputs and credentials; custom transports are
trusted application code and must maintain that discipline.

## Cascades and deadlines

`Cascade::new(Vec<Arc<dyn Provider>>, threshold)` requires at least two providers.
It re-asks only undecided/below-threshold answers, never retries provider errors,
and retains original order and actual answering models. Final uncertainty stands.

With `tokio`, `TimeLimited::new(provider, duration)` returns a provider wrapper.
It requires a positive duration and a runtime with timers. Runtime configuration
errors become ErrorKind::Runtime; Tokio exposes timer absence through a panic,
so timer construction is caught before polling the provider. Whole-cascade and
per-hop deadlines intentionally differ. The original provider's errors propagate.

With `blocking`, `blocking::Blocking::new(provider)` owns a private Tokio runtime;
`.ask(&request)` rejects calls inside an existing Tokio runtime. Its drop does
not block an async thread. Prefer direct async providers in async applications.

## Flows

```rust,ignore
let mut context = FlowContext::new(&provider);
let result: Result<String, FlowExit> = async {
    let answers = context.ask(&state, &[department.erased(), urgency.erased()]).await?;
    let team = answers.require(&department)?; // owned typed value or uncertainty stop
    Ok(team)
}.await;
let outcome = context.finish(result); // Result<FlowOutcome<String>, FlowFailure>
```

`peek` returns a full typed answer for explicit uncertainty handling. Unread
uncertainty does not stop a flow. `require` returns FlowExit::Undecided with the
operator and its normalized answer. Provider errors latch: later asks do no work,
and finish cannot turn a caught provider error into success. Every attempted
request/result is retained in memory, including failed attempts. FlowFailure owns
the error and trace. `ask_request` supports explicit image assets.

FlowOutcome has value, stopped_at, stopped_answer, trace, decided() and calls().
An application can return Option::None successfully without confusing it with an
uncertainty stop. Traces are not logged or persisted automatically.

## Filtering and reranking

`each::BatchOptions::default()` selects concurrency eight, no context, and an
empty asset registry when enabled. Concurrency uses NonZeroUsize. `Item::new`
validates ID/state pairs; `Candidate::new` accepts arbitrary JSON documents.

`filter::filter_items(provider, &Boolean, &[Item], &options)` returns ordered
verdicts and matched/rejected/unsure/failed views. IDs are never added to state.
`rerank::rerank(provider, &Score, query, &[Candidate], &options)` scores each item
independently and sorts descending; exact ties retain input position.
`Ranking::top(k, allow_partial)` refuses partial results unless allowed and always
refuses mixed reported models. `reweighted(&weights)` uses existing probabilities
with finite increasing weights, normalized by their total, and makes no new calls.

Provider, timeout and malformed-response failures stay with each item. Invalid
local state/assets and unsupported capabilities abort the operation and drop
pending futures. All local IDs/assets are checked before starting any item.
Empty item lists make no calls. Output order is independent of completion order.

## Wire, images, and evaluation

`wire::decode_request(Value)` validates Python schema 1 and returns WireRequest
with state/questions/request_id. `encode_response` preserves the text-only wire
format, four significant digits, and first-answer call projection; `encode_error`
returns the established error envelope. Box/images require the library API rather
than a silent extension of schema 1.

With images enabled, use Image::from_bytes/from_path/from_base64 and Assets::attach.
`frames(&mut assets, images, optional_seconds_ago)` creates ordered State.
`locate::locate(provider, &request, target, LocateOptions)` replaces the request's
questions with a presence Boolean and a Box in one call while retaining assets.
Located reports found, optional region, presence and box_answer. See
[image semantics](IMAGES.md) for frozen versus current service constraints.

The `bench` module provides pure report scoring, stricter-threshold rescoring,
NDCG, stability and bounded async runners. See generated rustdoc for exact
signatures and [testing](TESTING.md) for interpretation and qualification limits.

## Evaluation

`bench::Case::new(state, labels)` retains a subset of expected Boolean/Choice/Score
labels; labels are checked against questions before scoring or any runner warmup.
`bench::score(questions, cases, answers, latencies_ms, total_ms)` produces a Report.
`bench::at_min_confidence` recomputes that report without calls and cannot restore
an existing abstention. `bench::stability` compares aligned decisions;
`bench::ndcg` accepts relevance grades and an explicit cutoff (`ndcg_default` uses 10).

`bench::run(provider, questions, cases, options)` measures bounded concurrent calls.
RunOptions defaults to concurrency 4 and warmup 1; Report exposes warmup_calls and
calls so the extra invocation is visible. Provider errors abort and drop pending
work. `bench::run_flow` executes borrowing flow closures sequentially and preserves
traces in FlowReport or FlowFailure. Pure `score_flows` accepts completed outcomes.
See rustdoc for signatures and [implementation coverage](IMPLEMENTATION.md) for
frozen report comparisons.
