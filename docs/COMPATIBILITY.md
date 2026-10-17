# Compatibility contract

Baseline: Python 0.12.0 at the exact revision in [source inventory](source-baseline.json).
The C++ snapshot is supporting evidence. None of the Rust behavior below is
implemented yet. Source tests are specifications to port, not a claim they were
rerun wholesale during setup.

## Normalization (mandatory parity)

1. Validate instructions (nonblank), confidence (finite [0,1]), and at least two
   unique nonempty Choice names / Score levels. Whitespace-only labels are allowed
   by Python; do not silently trim. Preserve supplied text and order.
2. Non-Box probabilities must cover exactly the outcomes; each is finite [0,1].
   Sum tolerance is `0.005 * number_of_outcomes + 1e-9`. Use accurate summation
   sufficient to reproduce Python `math.fsum` decisions at the boundary. Do not
   renormalize probabilities during answer construction.
3. Boolean value must be bool and agree with the higher probability unless tied.
   Confidence is max probability; exact ties are undecided. Complement is
   Python-compatible `round(1-p, 10)`, including decimal/tie boundary fixtures.
4. Choice value is one of the options and has maximum probability. Multiple exact
   maxima are undecided. No epsilon tie detection or arbitrary winning label.
5. Score is a finite expected score in `[0, n-1]`, not a chosen integer level.
   Compare with `sum(i*p[i])`; permitted discrepancy is
   `0.005 * (1 + sum(0..n-1))`. Confidence is the probability of level
   `min(floor(value + 0.5), n-1)`, not maximum probability. Score has no tie rule.
6. Box value is Region; probabilities must be empty. Explicit confidence must be
   finite [0,1]; omitted confidence becomes 0. No synthetic distribution.
7. Abstain for ties or `confidence < min_confidence`; equality passes. Keep the
   distribution, confidence, call, and raw data even when value becomes None.
8. Region edges are finite [0,1] with top <= bottom and left <= right. Zero-area
   boxes are permitted by the baseline. Do not add minimum area implicitly.
9. Invalid responses are provider/protocol errors, never ordinary undecided answers.

## Feature coverage and source map

| Area | Python source/tests | Rust milestone |
|---|---|---|
| Questions, answers, Region | `types.py`; `test_answers.py`, `test_validation.py` | M1 |
| Metadata, errors | `types.py`, `errors.py`; `test_calls.py`, `test_errors.py` | M1–M2 |
| Operators, batching | `operators.py`; `test_operators.py` | M2 |
| Provider timeout/cascade | `provider.py`, `cascade.py`; `test_timeouts.py`, `test_cascade.py` | M2 |
| Flows | `flows.py`; `test_flows.py` | M3 |
| Filter/rerank | `filtering.py`, `rerank.py`, `_each.py`; matching tests | M3 |
| Benchmarks | `bench.py`; benchmark-related source tests / fixed reports | M3 |
| TypeSafe | `providers/typesafe.py`; mocked provider tests | M4 |
| Images/frames/locate | `images.py`, `locate.py`; `test_images.py` | M5 |
| Wire | `interfaces/wire.py`; `test_interfaces.py` | M1/M4 |
| CLI/MCP | `interfaces/cli.py`, `filter_cli.py`, `mcp_server.py` | Deferred |
| Laya local runtime | `providers/laya.py` | Deferred |

## Wire schema 1

Decode the same text/object/array state; nonempty questions array; ordered unique
names; boolean/choice/score only; optional string request_id (null accepted as
absent); optional schema_version exactly string `"1"`. Reject unknown fields at
each schema-owned level, including options, with field paths. Arbitrary state
object keys remain allowed. Preserve option string versus described option semantics.

Encode answer type, value (null for undecided), decided, confidence, probabilities,
and call. Round floats to four significant digits, retaining bool/string/null.
Do not include raw response data. Error codes remain `invalid_request`,
`provider_error`, `timeout`; path/message compatibility gets frozen fixtures.
Byte ordering of JSON keys is not a semantic requirement, but ordered arrays,
probability presentation order, exact request contents, and numeric results are.

Python's wire encoder uses the first answer's call, even though cascades can have
mixed calls. Rust schema 1 must reproduce this compatibility projection and
document its information loss; rich library answers keep per-answer metadata.
Do not extend schema 1 to images or Box even though the library supports them.
Any richer wire representation needs a separately versioned contract.

## Evaluation and recipe semantics

Reference benchmark decisions use Boolean/Choice values directly and round Score
half up (`floor(value + 0.5)`) to a level index; Box benchmarking is rejected.
For each labeled question: answered = total - undecided, accuracy = correct/total,
conditional accuracy = correct/answered, with zero for empty denominators. Mean
probability of the correct outcome is retained. Undecided results are not misses.
Threshold sweeps can only remove decisions; they never restore an already
undecided result. Provider failures abort the reference runner; a future Rust
failure-summary report must be a labeled extension, not silently alter denominators.
The reference label-to-probability lookup does not support an abstention label;
must-abstain evaluation needs a separate explicit extension with its own metrics.

NDCG uses gain `2^grade - 1` discounted by `log2(position + 2)`, missing IDs grade
zero, default k=10, and None when the ideal gain is zero. Stability is the fraction
of cases with identical decisions across reports, including identical abstentions;
it says nothing about correctness. Validate empty/misaligned reports instead of
porting Python indexing errors. Benchmark warm-up defaults to one extra call;
make that cost visible and count it in any authorized budget. Async benchmark
concurrency defaults to four (per-item recipe default is eight).

Reranking uses exact descending score and original position for ties. Reweighting
uses `sum(weight[i]*p[i])/sum(p[i])`, finite strictly increasing weights, and no
new provider calls. Filtering preserves exactly one match/no/unsure/failed outcome
per item. Input IDs are metadata, not part of provider state.

## Deliberate Rust adaptations

| Adaptation | Reason / impact |
|---|---|
| Async future rather than C++ pump callbacks or Python sync/async twins | Native Rust executor integration; same answer semantics |
| Fallible constructors and private validated fields | Cannot construct invalid public values accidentally |
| Exact response question-set validation, including extra keys | Stricter than Python adapter's selection of requested keys; inherited from C++ provider checks |
| Explicit request-owned assets rather than global image lookup | Lifetime/thread safety; request body semantics retained |
| Structured Result errors and flow exits | No Python exception emulation or implicit bool conversion |
| Empty apply and empty operator names rejected | Tighten ambiguous Python library cases; wire already requires names/nonempty questions |
| Flow provider failures latch until finish | Prevent caught error from becoming apparent success; documented C++ behavior |
| Checked finite padding / positive pixel dimensions | Rust convenience validation; valid reference inputs retain results |
| No implicit SDK retries | Clear call accounting; Python SDK retries depend on caller configuration |

No automatic adoption of stale C++ design statements: its original 0.11 baseline
predates images; its DESIGN also contains historical RTTI/build claims. Source
revision and relevant tests take precedence over old prose.

## How parity is proved

Commit Python-generated golden cases with generator and exact baseline revision.
Rust reads the same cases; it must not generate its own expected answers. Test
invalid input classification and fields separately from numeric tolerance; freeze
boundary cases where tiny float changes affect decisions. Extend fixtures for
wire and transport before those milestones close. C++ comparisons are a useful
second check; disagreements are documented and resolved against Python or an
explicit design amendment. Live model stochastic outputs cannot prove bit parity.

