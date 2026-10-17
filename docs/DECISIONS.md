# Initial decisions

2026-10-17. U1–U2 are user instructions; D1–D9 are working engineering choices
made to prepare the requested project, subject to API/design review during development.

| ID | Decision | Reason / alternative |
|---|---|---|
| U1 | Specify and create a Rust semantic-operators/semops project and repository | Jason's request; not an instruction to finish or publish the port now |
| U2 | Target Cargo name semantic-operators | User identified possible availability; verification inconclusive (403), not reserved |
| D1 | Folder/repo semantic-operators-rs, private under jasonduncan | Existing Python remote owns semantic-operators; private workspace default |
| D2 | Python 0.12.0 current local source is authoritative; C++ supporting | Current image additions matter; legacy Python API excluded |
| D3 | Native Rust, no FFI; single crate with optional integrations | Smallest portable packaging; split core/provider later if warranted |
| D4 | Async canonical, boxed Send futures and dyn providers | Heterogeneous cascade and Rust integration; C++ pump API not ported literally |
| D5 | Edition 2024, working MSRV 1.85, MIT | Contemporary edition with explicit minimum; retain sibling license |
| D6 | Typed answers with Option and Result; shared checked dynamic boundary | Explicit abstention/errors without duplicating semantics |
| D7 | Explicit request-owned image handles | Thread/lifetime safety; avoid hidden global placeholder registry |
| D8 | Keep wire v1 text-only and retain numeric parity | Interoperation; images/richer call provenance require a future schema |
| D9 | Buildable empty crate, publish=false, fixtures/docs/CI first | Honest setup scope; avoid pretending partial implementation is usable |

## Review triggers

- M1: validate numeric algorithm choices and public ordered collection ergonomics.
- M2/M3: compile dyn-provider and async-flow examples before freezing API.
- M4: choose Rust SDK versus direct HTTP using current source/docs evidence.
- M5: refresh provider image capabilities/limits and choose vetted parser dependencies.
- M6: inspect package, verify crate availability, decide publication/visibility.

Changing a working decision needs a dated rationale and compatibility impact, not
a rewrite implying it was always the design. Only consequential product/release
choices need owner decisions; routine implementation can follow these contracts.


## Implementation decisions — 2026-10-17

Jason explicitly requested “BUILD IT!” and allowed subagents, authorizing M1–M6
implementation and verification. U1's original setup-only boundary is superseded
for development; publication and model qualification remain separate.

| ID | Implemented decision | Reason |
|---|---|---|
| D10 | reqwest 0.12 + rustls optional narrow TypeSafe transport | Control exact request translation, bounded response/body handling, no redirects/retries; injectable offline tests |
| D11 | futures-util polling for recipes, no spawned tasks | Bounded concurrency with drop cancellation and no required core runtime |
| D12 | Header-only PNG/JPEG validation | Frozen reference behavior without an image decoding stack; renderability is not promised |
| D13 | Separate local image parity from service preflight | Core keeps 256-side reference limit; current hosted transport requires 280 and rejects asset placeholders in keys |
| D14 | serde_json float_roundtrip and Python compensated summation | Preserve threshold/expected-score behavior at numeric boundaries |
| D15 | Explicit Blocking runtime and checked Tokio deadlines | Typed runtime misuse errors instead of hidden runtime creation |
| D16 | Separate CRATE.md archive readme and packaged-consumer checks | Standalone usable package without private status/source inventories |

0.1.0-alpha.1 remains unpublished with `publish=false`. The default feature set is
empty and does not link Tokio, HTTP or TLS dependencies. See IMPLEMENTATION.md for
requirement coverage and COMPATIBILITY.md for deliberate deviations.
