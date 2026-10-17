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

