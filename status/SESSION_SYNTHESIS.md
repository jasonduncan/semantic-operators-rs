# Session synthesis

## 2026-10-17 — Rust specification and setup

Jason requested a Rust version of semantic-operators/semops, referenced the Python
and C++ repositories, suggested semantic-operators as an available crate name, and
asked for a project folder, design/development documents and repository.

Inspection established Python 0.12.0 as the current local baseline, including images,
Box and locate, rather than the older legacy observation API or C++'s original
0.11 design snapshot. Source revisions and hashes were retained without changing
those repositories. Python wire v1 remains text-only despite library image support.

Working choices: native Rust rather than FFI; typed answers and explicit Result /
Option; object-safe async providers; minimal core with optional integrations;
explicit request-owned assets; preserve numerical semantics and documented wire
limitations. Detailed decisions remain reviewable in docs/DECISIONS.md.

Created a private jasonduncan/semantic-operators-rs repository and local Cargo
bootstrap, documentation, CI, 38 deterministic Python reference cases and status
package. No semantic runtime was implemented. Crates.io lookup returned 403, so
availability is unknown. No package reservation, publication, live inference,
private corpus transmission or paid evaluation occurred. Only new project material
is pushed to the requested private GitHub repository.

The local research dossier PR-2026-009 preserves source/design rationale and links
this project through PD Labs discovery. The active registry points to this status
package; it is not a second project truth source. Next is M1 normalization and
validation, not provider networking. See VALIDATION for measured setup evidence.

Completion evidence: hosted CI passed seven jobs twice, including the refreshed
workflow on Linux/macOS/Windows and Rust stable/1.85.0. Official package Git index
checks support an apparently unused name; direct registry access stayed blocked.
The final commit records this evidence without extending implementation scope.

## 2026-10-17 — native Rust implementation

Jason explicitly said “BUILD IT!” and allowed subagents. Built M1–M6's native
library surface with core/wire, image/recipe and transport agents, integrating
execution/flows, packaging and documentation centrally. API is now actual Rust
rather than proposal. Version 0.1.0-alpha.1 remains private and unpublished.

Expanded frozen evidence to 213 Python expected scenarios: 172 core/wire, 25 recipes,
16 benchmark scenarios. Independent review caught numeric parsing/summation,
Python whitespace, Debug leakage, abstention restoration, extreme-weight overflow,
disabled-runtime handling, adversarial multipart boundaries, and delayed flow
validation/trace loss. Fixes have regressions. Default core remains executor-neutral;
Tokio, blocking, images and TypeSafe are opt-in. Direct HTTP was chosen after
inspecting SDK/source contracts; official current image preflight is kept separate
from frozen Python core limits.

All five examples use explicitly synthetic providers. The actual Cargo archive
was extracted and consumed independently with default and optional features.
No hosted inference, private corpus upload, public release, tag or crates.io
publication occurred. Localhost test servers terminate inside tests. Hosted CI
receipts and exact acceptance limits are maintained in VALIDATION.md.
