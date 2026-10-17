# Proposed Rust API

Design sketch, not exported by the bootstrap. Names may change during M1–M3.
Use `semantic_operators` normally; an application can write
`use semantic_operators as semops;`. No second crate name is needed.

## Questions and operators

```rust,ignore
use semantic_operators::{Boolean, Operator};

let complaint = Operator::new(
    "complaint",
    Boolean::builder("Is the customer reporting a problem?")
        .min_confidence(0.8)
        .build()?,
)?;
let answer = complaint.ask(&provider, &request).await?; // Answer<bool>
match answer.value() {
    Some(true) => { /* triage */ }
    Some(false) => { /* ordinary message */ }
    None => { /* ask a person */ }
}
```

`Choice::builder(instructions).option(name, description)` preserves insertion
order and rejects duplicate names. Score takes ordered unique level descriptions.
Box returns Region. Constructors reject empty instructions and nonfinite or
out-of-range confidence. Top-level state accepts text, object, or array; scalar
JSON values can occur inside an object/array. `State::try_from(Value)` validates
that boundary. Questions and a request borrow safely across await; no references
to a temporary request are kept after completion.

`Operator<Q>` contains a nonempty name and validated question, never a provider.
An erased `NamedQuestion` allows heterogeneous batches. `apply(provider, request,
&[complaint.erased(), urgency.erased()])` makes one provider call and returns an
ordered `Answers` collection. `answers.get(&complaint)` returns a checked typed
view; name/type mismatches are errors. Empty apply is rejected before I/O, an
explicit tightening over Python's unspecified empty operator case.

## Provider extension point

```rust,ignore
use std::{future::Future, pin::Pin};
pub type ProviderFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Answers, Error>> + Send + 'a>>;

pub trait Provider: Send + Sync {
    fn ask<'a>(&'a self, request: &'a Request) -> ProviderFuture<'a>;
}
```

`Request` owns validated state, ordered questions, and (when enabled) explicit
assets. A request with multiple questions means one semantic call. The object-safe
boxed future supports heterogeneous cascade providers without an async-trait
macro; native async trait methods alone do not give this dyn interface. Custom
providers should build answers through the central checked constructors. Library
entry points additionally check names, variants, distributions, and metadata shape.

`FunctionProvider`/`ScriptedProvider` belongs in test support or examples initially;
no fake service result is used by default in the actual TypeSafe adapter.

## Answer inspection

```rust,ignore
pub struct Answer<T> { /* private checked fields */ }
// Proposed accessors:
// value() -> Option<&T>
// decided() -> bool
// confidence() -> f64
// probabilities() -> ordered view of (label, f64)
// call() -> Option<&Call>
// raw() -> Option<&serde_json::Value>
```

`Call` carries provider, optional reported model, optional unsigned input/output
token counts. Share one `Arc<Call>` across a provider response. Cascades may return
answers from different calls. Do not collapse that into an invented single model.

## Wrappers and recipes

`Cascade::new(Vec<Arc<dyn Provider>>, escalate_below)` needs at least two providers
and finite threshold in [0,1]. `TimeLimited::new(provider, Duration)` requires a
positive duration and the Tokio feature. Default retry count is zero in the
proposed HTTP adapter. Transport configuration remains explicit.

`filter_items` takes ordered ID/state pairs and a Boolean; `rerank` takes query,
ordered ID/document pairs, Score rubric, optional context, and concurrency.
Use `NonZeroUsize` for concurrency; default 8. Duplicate/empty IDs are rejected,
IDs stay local, empty input produces an empty result with zero calls. Results
retain input position and all failures. Recipe functions return `Result` for
invalid configuration and place provider errors in per-item records.

## Flows

Use an explicit `FlowContext` first, avoiding a fragile async-closure lifetime API:

```rust,ignore
async fn triage(ctx: &mut FlowContext<'_>, state: &State)
    -> Result<String, FlowExit>
{
    let answers = ctx.ask(state, &[department.erased(), urgency.erased()]).await?;
    let team = answers.require(&department)?;
    if team == "technical" {
        let answer = ctx.ask(state, &[outage.erased()]).await?;
        if *answer.require(&outage)? { return Ok("page on-call".into()); }
    }
    Ok(team.clone())
}
```

The operators in this sketch are application definitions. `FlowExit` distinguishes
`Undecided { name, answer }` from `Error`. `FlowContext::finish(result)` produces
either a decided/stopped outcome or a flow error, preserving its trace in each
case. Reading with `peek` returns the full answer so callers can handle uncertainty
explicitly. Context latches provider failure; subsequent asks do no work and
finish cannot turn that failure into success. This deliberately follows the C++
error-preservation rule; Python user code can otherwise catch exceptions.

## Images and localization

`Image::from_bytes` validates bytes and returns an owned `Arc<[u8]>` plus format,
dimensions and diagnostics. `request.attach(image)` returns an explicit handle
whose placeholder can appear in state or question text; the request owns the
association. No Python-style weak global registry. Missing referenced assets
fail locally. The helper `frames` builds oldest-first state entries.

`locate(provider, request, target, options)` asks Boolean presence and Box together.
It returns `Located { found: Option<bool>, region: Option<Region>, presence, box }`.
A confident presence answer can be true while the region is absent due to its
own confidence threshold. Always retain both answers. `Region` validates finite
ordered fractional edges; padding clips to bounds; pixel conversion matches
Python ties-to-even rounding with checked positive image dimensions.

