# Semantic Operators Rust: contributor instructions

Read `status/README.md`, `status/HANDOFF.md`, `docs/DESIGN.md`, and
`docs/COMPATIBILITY.md` before implementation. Current scope is documented in
`status/DEV_PLAN.md`; distinguish proposed APIs from shipped behavior.

- Port the frozen **semantic-operators-lib 0.12.0** Python checkout recorded in
  `docs/source-baseline.json`. The older observation-oriented Python library is
  not this project's baseline. C++ libsemop is a second reference, not an FFI dependency.
- Core semantics must compile with no default features and no network/runtime dependency.
- Fallible constructors, finite numeric validation, ordered questions and outcomes,
  explicit undecided values, `Result` errors; no unsafe code or panic-based input handling.
- Normalize provider output centrally. Never convert errors to abstentions or use
  cascade as silent error failover. Never invent confidence, usage, or model identity.
- Preserve Python numeric and wire behavior with offline fixtures; record deviations
  in `docs/COMPATIBILITY.md`. Never regenerate expected values from the Rust implementation.
- Default checks use synthetic inputs and scripted transports. Live provider calls
  require a separately scoped request with dataset, model, call/cost limit, and retained evidence.
- Do not print credentials or raw private inputs. Keep generated runtime material in ignored `var/`.
- Run the development checks and update status/evidence after meaningful work.
- Repository creation and bootstrap do not authorize a release, crates.io publication,
  public visibility, or acceptance of model quality. `publish = false` remains until release work.

