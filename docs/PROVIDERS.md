# Providers, transport, and image contract

Proposed implementation contract, 2026-10-17. No provider is implemented yet.

## Provider boundary

The core owns question/answer validation; adapters translate validated data.
Use an object-safe async Provider and caller-driven futures. A contract suite must
exercise one and multiple questions, arbitrary supported state shapes, concurrent
calls, missing/extra answers, malformed distributions, errors, cancellation and
preserved call metadata. Check capabilities before I/O where known; a missing
capability is an error, not an abstention. Never infer support from a model alias.

Core never reads credentials, URLs, files, environment variables, or clocks.
Adapters receive explicit configuration and a reusable client. Examples may read
`TYPESAFE_API_KEY` at their boundary. Errors and Debug redact credentials and omit
input/response bodies by default. Traces are in-memory opt-in inspection artifacts.

## TypeSafe adapter selection gate (M4)

Existing references use Python typesafe-sdk >=0.7.5 and C++ libtypesafe >=0.2.
There is no verified Rust SDK dependency in this specification. Before selecting
one, inspect its source, maintenance, licensing, request shape, retry behavior,
timeouts, multipart implementation, and MSRV. Otherwise implement a narrow HTTP
adapter using an optional established Rust HTTP client. Do not port C++ networking
or add FFI merely to reuse semantic question serialization.

Expose an injectable transport at the adapter boundary so integration tests can
run without API keys. Keep endpoint/model/timeouts configurable; pin model IDs
in qualification runs. Validate official endpoint/authentication/response details
from current provider documentation during M4; no endpoint is invented here.

Reference request semantics:

- Boolean becomes `noul`; criteria are absent when both descriptions are absent.
  If one side is described, send both keys, preserving null on the undescribed side.
- Choice criteria preserve names, descriptions (including null), and order.
- Score criteria preserve level order. Map numeric/index response probabilities
  back to level descriptions before normalization.
- Box maps to Box. Boolean responses use p(true) plus rounded complement.
- Send model selection; record the model the response reports, not that selection.
- Malformed usage counts become absent. Unsigned Rust storage must not coerce
  negative/fractional/bool values to a token count.
- Reuse a Call record across answers in that provider invocation. Retries are
  explicit client policy, default disabled in the proposed adapter.

Test malformed JSON, authentication failures, rate limits, non-2xx, service timeouts,
partial responses, extra names, and cancellation using local/scripted transport.
An error category can retain status/retry-after but cannot silently schedule a retry.

## Deadline and cascade semantics

`TimeLimited(provider, duration)` bounds one awaited ask. Timer expiry becomes
Timeout; a provider's own timeout remains that error. Do not map arbitrary
transport errors to Timeout. Dropping a future is local cancellation only.

`TimeLimited(Cascade(...), d)` covers the entire chain;
`Cascade(TimeLimited(p1,d), TimeLimited(p2,d))` gives each attempted hop a deadline.
The latter may take more than d. Cascade stops on any provider error and does not
try the next provider. After a successful hop, re-ask only answers undecided or
below escalate_below. Final-hop answers stand even if still undecided. Preserve
original order and the actual answering call for every result.

## Images (M5)

The following limits are frozen **local Python 0.12 reference behavior**, not a
current hosted-service guarantee. Refresh official documentation before live use:

| Rule | Frozen value |
|---|---|
| Formats | PNG/JPEG still images; reject animated PNG |
| Size | At most 16 MiB per image |
| Dimensions | Each side 256–1568 px inclusive |
| Area | 78,400–2,458,624 px inclusive |
| Request total | At most 32 MiB, counting every reference/use |
| Diagnostics | EXIF rotation and non-baseline JPEG are reported, not silently corrected |
| Multipart | `assets` by image placeholder; image danger-zone header set |

Request-owned image handles carry immutable bytes, stable placeholder identity,
dimensions and diagnostics. Collect references from state and question text in
first-use order. Deduplicate upload parts by handle, but preserve repeated-use
accounting for the request limit. Fail on unknown placeholder references before
network I/O. Never load a path or URL merely because it appears in state text.

The Python request includes `TypeSafe-Danger-Zone-Images: true` whenever an image
or Box is present. The baseline docs say Box needs jev-1.14.0/preview; treat that
as a dated observation, reverify capability rather than hard-coding alias support.
`locate` batches presence and region; Box alone is not evidence of presence.
`frames` preserves oldest-first order and validates timestamps/lengths against
the reference. No image upload occurs during baseline fixture generation.

