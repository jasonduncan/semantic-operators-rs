# Development handoff

2026-10-17 · M0 complete; M1 ready; no implementation blocker.

1. Read [CURRENT](CURRENT.md), [plan](DEV_PLAN.md), [design](../docs/DESIGN.md),
   [compatibility](../docs/COMPATIBILITY.md), and [source inventory](../docs/SOURCE_BASELINE.md).
2. Inspect `git status --short --branch`, `git log -3 --oneline`, and origin.
   Preserve unrelated changes; inspect divergence before updating the baseline.
3. Run the local checks in [development](../docs/DEVELOPMENT.md). Zero Rust tests
   at bootstrap is expected, not evidence that semantic behavior works.
4. Implement M1: fallible questions and checked answers, with a Rust fixture reader.
   Keep the frozen Python expectations and add targeted boundary cases before
   claiming normalizer parity. Do not start HTTP integration as the first slice.
5. Once the API exists, compile its examples, expand feature checks, and record
   actual results and deviations. Update status and handoff_version, then validate
   the manifest, commit/push authorized work, and refresh the project registry.

Python reference local path and hashes are in source-baseline.json; fixture
regeneration requires that exact clean snapshot. The Rust repo builds independently
of sibling paths. There are no running services, remote jobs, provider reservations,
keys, paid requests, or outstanding releases to recover.

Owner request authorized specification, folder/setup and repository creation.
This handoff identifies the next development task; new implementation work follows
subsequent task direction. Live qualification, public visibility and crates.io
publication require their own scope. Do not treat name availability as confirmed.
