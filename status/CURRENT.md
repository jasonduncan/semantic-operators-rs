# Current state

2026-10-17 · portfolio active · execution ready · next phase M1.

The requested specification and repository bootstrap are complete. The crate is
`semantic-operators` 0.1.0-dev.0, import `semantic_operators`, with publication
disabled. The GitHub repository is private, default branch main.

Available: design/API/compatibility/provider/development/testing/release documents,
source revisions and hashes, 38 Python-generated synthetic reference cases, a
buildable empty Rust library, platform/MSRV CI configuration, and this handoff.

Not implemented: questions/answers, providers, operators, wrappers, recipes,
benchmarks, images, wire codecs, CLI, or MCP. No Rust semantic tests exist yet.
Build success is bootstrap evidence only. See [validation](VALIDATION.md).

No blocker prevents M1. Crate name availability could not be verified (HTTP 403);
both hyphen/underscore names are absent from the accessible official Git index,
so the name appears unregistered. Guaranteed availability/reservation remains a
future release concern, not a development blocker. TypeSafe Rust SDK
selection and current service image capabilities remain M4/M5 investigations.

Next: implement validated questions and answer normalization with frozen reference
fixtures, then expand the numeric boundary corpus. Full scope and exits are in
[DEV_PLAN](DEV_PLAN.md).
