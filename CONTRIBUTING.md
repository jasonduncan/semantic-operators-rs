# Contributing

Begin with the [development guide](docs/DEVELOPMENT.md) and
[current handoff](status/HANDOFF.md). Each change should complete a bounded item
from the [development plan](status/DEV_PLAN.md), carry relevant offline evidence,
and update any changed compatibility or API contract.

Use synthetic fixtures. Preserve upstream attribution and licenses when porting
code. Explain deliberate behavior changes rather than silently correcting the
baseline. This public repository ships an initial alpha; its API is not stable.

Use standard rustfmt defaults and Rust naming conventions. Public fallible APIs
need `# Errors` documentation; runnable examples should use synthetic providers.
Avoid logging from library code and keep credentials/input bodies out of errors.
The crate forbids unsafe code. Preserve explicit uncertainty and error outcomes.

Dependency changes must pass both lockfile audits and the stable/MSRV matrix.
Weekly Dependabot updates are reviewed through CI, never automatically merged.
The independent triage app intentionally keeps its own manifest and lockfile to
exercise the library as a downstream consumer; do not fold it into a workspace
merely for consistency. See the development guide for exact checks.

