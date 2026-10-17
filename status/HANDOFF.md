# Development handoff

2026-10-17 · M1–M5 complete; M6 local gates complete, hosted CI pending.

1. Read [CURRENT](CURRENT.md), [VALIDATION](VALIDATION.md),
   [implementation mapping](../docs/IMPLEMENTATION.md) and
   [compatibility](../docs/COMPATIBILITY.md).
2. Inspect Git state and remote main before continuing; preserve unrelated edits.
3. Finish/verify the implementation CI run and record the exact revision and jobs.
   Repeat relevant local checks from [DEVELOPMENT](../docs/DEVELOPMENT.md) for fixes.
4. Select the next application integration or scoped provider qualification. Do not
   restart M1 or replace the frozen fixtures. Public API remains alpha/unstable.
5. Publication is separate: recheck crate availability, inspect package/API and use
   [RELEASE](../docs/RELEASE.md) under explicit publication direction.

Jason's “BUILD IT!” authorized the full implementation and subagent work. Offline
and loopback verification use only synthetic inputs. No credentials, running
services, paid requests, reservations or deployment need recovery. Ordinary builds
do not require Python or sibling checkouts; pinned sources are needed only to
regenerate golden fixtures. Benchmark runners invoke their supplied providers and
may add warmup calls, so select inputs/provider deliberately.

Known evidence limits: header-only image validation, documented core/service image
differences, text-only wire v1, bounded parity corpus, no measured model quality or
performance claim. The complete next-phase scope is retained in DEV_PLAN.md.
