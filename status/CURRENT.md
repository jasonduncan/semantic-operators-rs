# Current state

2026-10-17 · portfolio active · execution complete · M7 consumer integration.

The native Rust `semantic-operators` 0.1.0-alpha.1 library and a standalone JSONL
triage consumer are implemented, privately pushed and verified. Main remains private
and Cargo publication disabled. The alpha API remains unstable.

M1–M6 cover validated questions/answers, providers/operators, batching, cascade/
deadlines, traced flows, recipes/evaluation, wire v1, images/localization, TypeSafe
HTTP/multipart and blocking. Core has no default network/runtime dependencies.

M7 adds integrations/triage-app, an independent Cargo consumer using public APIs.
It validates the full input batch and explicit call limit before dispatch; batches
department and urgency once per ticket; emits decisions and probability/call evidence;
routes uncertainty to review; stops on failures while preserving completed rows.
See [its usage and input/output contract](../integrations/triage-app/README.md).

Local end-to-end checks passed 32 scenarios (31 on Windows, excluding Unix argv).
All eight hosted jobs passed on feb00f2, including the consumer on Linux/macOS/
Windows, stable and Rust 1.85.0. [Validation](VALIDATION.md) records exact receipts.
The library source, fixtures and package surface are unchanged by M7.

Next: select an existing application to integrate or a bounded live-provider brief
with dataset, exact model and call/cost limits. No hosted inference, private corpus
upload, paid calls, deployment, crate reservation or publication occurred. Generic
CLI/MCP, local inference and other explicitly deferred surfaces remain deferred.
