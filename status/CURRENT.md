# Current state

2026-10-17 · portfolio active · execution ready: crates.io release preparation.

The native Rust `semantic-operators` 0.1.0-alpha.1 library (M1–M6), standalone
JSONL triage consumer (M7), and bounded-check tooling are implemented and verified.
Jason confirmed GitHub PUBLIC visibility is intentional for crates.io distribution.
Keep it public; the earlier visibility hold is resolved. Cargo publication remains
disabled and the alpha API unstable; no registry release exists.

[CI 39079995335](https://github.com/jasonduncan/semantic-operators-rs/actions/runs/39079995335) passed all eight jobs at
0c311756b9572fbf44a45c2ad4b3c8b669a11f99, including Linux/macOS/Windows on stable
and Rust 1.85.0. The Windows Unicode-fixture correction is verified. All 19 Python
tool tests pass; package audit and default/optional extracted consumers pass.
Closing updates only record evidence and handoff; source/tests/workflow match that run.

Next is the [crates.io release procedure](../docs/RELEASE.md): version/changelog,
publication setting, final package dry-run and registry release. Name checks now
support apparent availability but do not reserve it. No credential was read for
release checks and no upload occurred.

The [bounded provider-check packet](../qualification/README.md) is separately ready:
12 synthetic tickets, 10 assistant-authored clear labels and 2 exploratory review
probes; explicit runner; strict offline scorer; ignored run receipts. Executing it
still requires an exact TypeSafe model and call ceiling. No hosted inference, model
acceptance or private corpus upload occurred. This optional scope is not a blocker
for package engineering work. Deferred library surfaces remain deferred.
