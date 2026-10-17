# TypeSafe transport implementation

Implemented 2026-10-17 behind `typesafe`; image attachments additionally require
`images`. This is offline conformance evidence, not live-service qualification.
No API keys or inference requests were used during implementation.

## Selection and source evidence

The [official OpenAPI](https://api.typesafe.ai/openapi.json) and
[API reference](https://api.typesafe.ai/redoc), inspected 2026-10-17, document
`POST https://api.typesafe.ai/v1/systemone`, bearer authentication, named questions,
reported model and usage, and JSON responses. They now explicitly document Box,
`TypeSafe-Danger-Zone-Images: true`, and multipart: JSON first in `request`, then
named image parts referenced by each asset's `part` field. This adapter implements
that contract with configurable endpoint and model. It does not guess model
capabilities from aliases.

A Rust SDK does exist: [typesafe-sdk 0.2.0 source and README](https://github.com/codeitlikemiley/typesafe-sdk-rust)
were inspected. It uses MIT licensing, Rust 1.85/edition 2024, reqwest 0.12,
configurable timeouts, and retries by default. Its cached README states Python SDK 0.6.0, while current source includes Box and
multipart modules. Direct inspection of `src/question.rs` found that Noul criteria
omit absent sides (`skip_serializing_if`), unlike this project's required explicit
null. Its documented redirect and retry defaults also differ. The selected implementation is a narrow reqwest
0.12 adapter, with the resolved dependency version in Cargo.lock. No SDK/FFI or
external runtime is introduced into the default core dependency tree.

Translation is grounded in the frozen Python 0.12 semops adapter recorded in
`source-baseline.json` and inspected C++ libtypesafe `src/wire.cpp` multipart
encoding. Neither local sibling is a build dependency. Expected semantics:
Boolean criteria absent when both descriptions are absent; when either is present,
both keys remain and the missing description is JSON null. Choice names and null
descriptions preserve order. Score rubric indices map back to descriptions.
Answers normalize through the core. Missing/extra names, wrong types, malformed
numbers/distributions, and inconsistent answer values fail.

## Public use and privacy

Create `Config::new(api_key, model)?`, optionally configure the complete endpoint,
timeout and response byte limit, then `TypeSafe::new(config)?`. Call through the
normal checked provider boundary. A Tokio runtime with timers enabled is required.
Credentials are explicit; the adapter never reads environment variables for keys.
Reqwest may honor the application's standard network proxy environment.

`TypeSafe::with_transport(config, Arc<dyn Transport>)` permits synthetic deterministic
fixtures and application networking. A transport receives sensitive `HttpRequest`
data through explicit fields; it is a trusted application boundary. All adapter
Debug output excludes keys, endpoint, model, input/response bodies, and image bytes.
Errors from the built-in transport omit external error messages and service bodies.
Custom transports must follow the same error/logging contract.

Raw answer JSON is discarded unless `with_raw_answers(true)` opts into in-memory
inspection. All answers share the same `Arc<Call>`. Model identity comes only from
the response; malformed/absent model or usage stays absent. Negative, fractional,
and Boolean usage counts are never coerced to unsigned counts.

## Bounds and cancellation

The complete transport await has a configured deadline, including injected
transports. Timeout produces `Timeout`, HTTP errors produce `Provider`, and
malformed successful responses produce `InvalidResponse`. HTTP status is retained
in a safe message; response bodies and arbitrary request IDs are not copied into
errors. There is no retry scheduler and no retained Retry-After metadata.

The default HTTP client disables redirects and reqwest protocol retries explicitly.
No tasks are spawned. Dropping an in-flight ask drops its transport future; this is
local cancellation and cannot establish cancellation or billing at the server.

Responses default to at most 4 MiB, checked against Content-Length and incrementally
while reading. JSON parsing occurs only after the bound. Complete JSON or multipart
requests are checked against 32 MiB before dispatch, including framing and text.
Multipart boundaries are chosen absent from every part. Binary image parts are
uploaded once per handle while the image registry accounts for every reference.

The local image library preserves pinned Python dimensions; the current adapter
also checks the [current image documentation](https://docs.typesafe.ai/images)
minimum side of 280 pixels. It rejects placeholders in object keys/question IDs/
Choice names because the current service does not replace keys. See `IMAGES.md`.
Encoded byte bounds do not prove compliance with the service's processed-image
limits: image decoding/expansion is not implemented here. Hosted capabilities and
model quality need separate scoped qualification before production use.

## Additional checked service constraints

The adapter rejects Score rubrics longer than 10 levels and Choice sets longer
than 255 options before dispatch, following the current
[Score documentation](https://docs.typesafe.ai/primitives/score) and OpenAPI Choice
schema. Core question constructors remain provider-neutral. Box requires at least
one referenced attached image; enabling a header alone does not establish that.

Choice/Score response confidence and Score legend are required HTTP fields in the
inspected Python SDK `_schemas/models.py` (`Field(...)`) and strict public response
types in `_core/response_types.py`. Their shape is checked, but reported confidence
does not replace the core's distribution-derived confidence. Legend descriptions
are not compared to question wording. This avoids inventing a semantic equivalence
check for echoed descriptions. Model and usage remain optional metadata as required
by this library's adapter contract.

Boundary search is capped at 16 attempts and JSON serialization is byte-bounded.
Runtime misuse, including Tokio runtimes with disabled timer/I/O drivers, becomes
`Runtime` rather than an escaping panic. The built-in reqwest future alone catches
runtime-driver panics; custom transports remain responsible for their implementation.
