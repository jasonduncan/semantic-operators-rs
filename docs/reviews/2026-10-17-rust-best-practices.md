# Rust best-practices review — 2026-10-17

Applied the owner-supplied Rust Best Practices Guide v1.0 to the published-alpha
source and repository maintenance workflow. Guide SHA-256:
`665ad7c75f0b31a888085c2b751af655e103eaf43ed9978fe8ba16f328b5e1d6`.
Starting HEAD: ac5323c1ffab3d8d485d2f1077f6ba84d3a446ed. Changes in this review
are unreleased; crates.io 0.1.0-alpha.1 and its tag are unchanged.

## Findings and changes

| Priority | Finding | Resolution |
|---|---|---|
| P2 | Dropping a polled FlowContext request lost its attempt and allowed finish to report success | Register an interrupted attempt before awaiting; replace it on normal completion; retain/latch Runtime on cancellation. Regression tests cover unpolled drop, prior successful trace, pending-work cleanup and no later dispatch. |
| P3 | Public fallible APIs often omitted explicit error contracts | Add accurate rustdoc Errors sections and executable offline examples; enforce error/panic documentation through Clippy. |
| P3 | No automated dependency advisory or update checks | Add a weekly/change-triggered audit for both lockfiles, pinned cargo-audit, weekly Dependabot updates and a review checklist. No automatic merges. |
| P3, retained | Duplicate-name error text does not exactly match Python repr for every Unicode class | Record the diagnostic limitation in COMPATIBILITY.md. NBSP and zero-width space differ; error classifications/paths and semantic decisions are unaffected. Full Unicode repr parity remains deferred. |

The cancellation fix uses the existing Runtime error category for local interruption,
not a synthetic provider timeout or abstention. Calls counts flow attempts, including
pre-dispatch validation failures; neither a trace nor cancellation proves remote
completion, cancellation or billing. No public enum variant or provider API changed.

## Applicable practices

- Existing rustfmt, Clippy warnings-as-errors, stable/MSRV and three-platform CI are
  retained. Explicit lints now catch missing error/panic docs and accidental library
  stdout/stderr/dbg output. No blanket pedantic lint churn.
- Source forbids unsafe code and uses validated private fields, structured Error,
  Result and explicit uncertainty. The two production expect calls construct known
  nonzero constants (4 and 8); they are not caller-controlled panic paths.
- Provider futures borrow requests, avoid spawned/detached work, and keep runtime/
  network dependencies opt-in. Cancellation has direct regression coverage.
- All 12 focused Miri normalization/wire tests passed, excluding networking and
  runtime integration. Native tests cover the latter; Miri is not a security proof.
- The standalone consumer deliberately has an independent lockfile to exercise
  downstream integration. A workspace conversion would remove that check.
- Keep the existing lightweight error implementation; adding thiserror/anyhow is
  not needed to obtain structured errors. Avoid unmeasured inlining, SIMD, caches,
  allocation pools or broad module reshuffling. No performance claim is made.
- Published API/tag/archive remain immutable. A future release must use a new version
  and have separate publication direction; this review does not upload another crate.

## Evidence

`cargo-audit` 0.22.2 checked both lockfiles, each with 137 dependency records, against
RustSec database commit `999ec89d5d3f10f20be14750ec8c2d8b44e0ac9b` (1338 advisories).
Both scans reported zero matching vulnerabilities and zero warnings, with no ignored
advisories. Strict --deny warnings runs also passed. These are current-known-advisory
checks, not comprehensive dependency review.

Final source/doc/Miri/CI results are recorded in [VALIDATION](../../status/VALIDATION.md).
All eight CI jobs and the separate dependency audit passed at commit 46cda9a.
Local final native suites passed 54 default and 88 all-feature tests; four doctests
and 12 Miri tests passed.
Local raw logs reside under ignored target/best-practices. The published release
receipt remains separate in [RELEASE](../RELEASE.md).

## References

- [RustSec cargo-audit](https://rustsec.org/): known-advisory scanning.
- [Cargo compatibility guidance](https://doc.rust-lang.org/cargo/reference/semver.html): avoid unnecessary public API changes.
- [Dependabot options](https://docs.github.com/en/code-security/reference/supply-chain-security/dependabot-options-reference): separate manifest/update schedules.
