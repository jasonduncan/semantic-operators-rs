# Current state

2026-10-17 · portfolio active · execution complete · phase M1–M6 native Rust alpha.

The Rust library is implemented as `semantic-operators` 0.1.0-alpha.1, import
`semantic_operators`. The repository is private, branch main, publication disabled.

Implemented: validated Boolean/Choice/Score/Box questions, typed answers, confidence
normalization, providers/operators, batched asks, cascade and deadlines, traced
flows, bounded filtering/reranking/reweighting, benchmark reports/runners, text wire
v1, image assets/frames/localization, TypeSafe HTTP/multipart, and blocking facade.
Core has no default runtime/network dependencies. Five offline examples are runnable.

Local tests, Python differential cases, package consumers, and documentation checks
provide implementation evidence. All eight hosted CI jobs passed at b92ddf0,
including Linux/macOS/Windows stable and Rust 1.85.0, feature matrix and package
consumers. M1–M6 are complete. See [VALIDATION](VALIDATION.md) and the
[requirement map](../docs/IMPLEMENTATION.md).

No hosted model qualification, real-document transmission, paid inference,
crates.io publication, reservation or public release occurred. CLI/MCP, local
inference and other explicitly deferred surfaces remain outside this candidate.
There is no implementation blocker; crate availability remains a release-time check.

Next: select an application integration or bounded live-provider qualification.
The API remains alpha; crates.io publication is a separate release decision.
