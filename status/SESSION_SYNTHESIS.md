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

Candidate closure: all eight hosted jobs passed at b92ddf0 in run 39068153607,
covering Linux/macOS/Windows stable and 1.85.0, feature isolation and extracted
package consumers. Initial lint differences across toolchains were corrected;
semantic tests remained green. M1–M6 complete; portfolio stays active, execution
complete for this phase. Next work is selected consumer integration or separately
bounded live qualification. No publication was performed.

## 2026-10-17 — standalone consumer integration

On “continue”, selected the prepared consumer-integration path. No existing target
application was named, so added integrations/triage-app as a self-contained Rust
consumer rather than changing another product. It depends on the public crate,
uses an explicit provider endpoint/model/call limit and emits evidence-bearing
JSON Lines. Both questions are batched once per ticket; uncertainty goes to review.
Input validates before dispatch; failures stop later calls with earlier rows retained.

The independent subprocess/loopback harness verifies application and HTTP adapter
together. No hosted provider is called. Rust 1.85 build and Clippy pass. Review found
non-UTF8 argv could panic; args_os with fallible UTF8 conversion now returns a
safe configuration error. The harness clears proxy environment settings to keep
its synthetic traffic local. The final harness passes 32 Unix scenarios (31 Windows); all eight hosted CI jobs
passed at feb00f2 in run 39074395130. M7 is complete. Library source/API and package
surface are unchanged. Closing handoff keeps portfolio active and marks this phase
complete; next work needs a selected existing application or bounded live brief.

## 2026-10-17 — bounded live-check preparation

On continuation after M7, prepared the next explicit qualification path. Built a
12-ticket synthetic packet, runner and strict offline scorer. Asked for exact model
and call ceiling as required by AGENTS.md; no answer or live scope has been received.
The packet separates 10 author-labeled clear cases from 2 exploratory review probes.
No quality acceptance threshold is invented. Coverage, abstentions, incorrect
joint labels, missing output and observed usage remain distinct.

Subagent scoring and independent review caught source/dataset read races and a
premature process-start claim. The runner now validates one dataset snapshot,
compares source hashes before/after its offline build, hashes runner/scorer, kills
and reaps interrupted/timed-out children, and preserves malformed output without
silently dropping rows. Input serialization respects the consumer's total byte cap.
Nineteen offline tests pass, and the workflow includes them across supported OSs.

Library and consumer Rust code are unchanged. No hosted request, real-document
transmission, paid inference or publication occurred. Preparation is complete;
portfolio remains active and execution is blocked only on model/call scope.

A final explicit repo/API readback found PUBLIC visibility, contrary to the prior
private record. No visibility setting was changed by this work. Preparation commit
c78bd05 had already been pushed. Asked Jason whether public visibility is intentional
or should be restored; further pushes are held while local status corrections and
CI receipts are completed. Origin was rechecked as the expected repository.

Hosted CI at c78bd05 passed six jobs; Windows failed one new Python fixture writer
using cp1252. All Rust and consumer checks passed. Added explicit UTF-8 locally and
reran all 19 tool tests successfully. The corrected hosted gate awaits the visibility
answer and subsequent push; do not claim all-platform prep acceptance yet.

## Public visibility resolved — 2026-10-17

Jason confirmed public visibility is intentional for crates.io distribution.
The visibility blocker and push hold are removed; no setting change is needed.
Proceed with the UTF-8 correction push and hosted verification. Cargo publication
remains disabled during release preparation; no package upload or live call occurred.
