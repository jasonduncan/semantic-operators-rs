# Contributing

Bug fixes, documentation, examples and compatibility reports are welcome. Start
with [GitHub issues](https://github.com/jasonduncan/semantic-operators-rs/issues)
to check for existing work. For a substantial API or behavior change, describe the
use case and proposed contract before implementing it. Usage questions can also
be filed as issues; include a small synthetic example. For vulnerabilities, use
the private reporting channel in [Security](SECURITY.md), not a public issue.

## Development setup

Install Rust 1.85 or newer and Python 3.12 or newer for repository checks. Then:

```sh
git clone https://github.com/jasonduncan/semantic-operators-rs.git
cd semantic-operators-rs
cargo fetch --locked
cargo run --example hello
```

The examples use scripted providers. Tests use synthetic inputs and local HTTP
fixtures; credentials and hosted inference are unnecessary. Once dependencies
are cached, add `--offline` to Cargo checks. Keep runtime artifacts in ignored
`var/` or `target/`, and never commit keys, private documents or raw provider logs.

Read the [development guide](docs/DEVELOPMENT.md), [design](docs/DESIGN.md),
[compatibility contract](docs/COMPATIBILITY.md), and [current handoff](status/HANDOFF.md)
for project boundaries. The development plan records existing work; a new issue
can propose work beyond it. API and compiler compatibility follow
[Versioning](VERSIONING.md).

## Checks before a pull request

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-features --all-targets -- -D warnings
cargo test --locked --all-targets
cargo test --locked --all-features --all-targets
cargo test --locked --all-features --doc
cargo doc --locked --all-features --no-deps
python3 tools/check_docs.py
```

Use `RUSTDOCFLAGS="-D warnings"` for strict documentation checking, or set that
environment variable using your shell's syntax. The development guide lists
package checks, the standalone consumer test and dependency audits. CI also
checks feature combinations and Rust 1.85 on Linux, macOS and Windows. Report
which relevant checks you ran, and which you could not run, in the pull request.

Use standard rustfmt defaults and Rust naming conventions. Public fallible APIs
need `# Errors` documentation; runnable examples should use synthetic providers.
Avoid logging from library code and keep credentials/input bodies out of errors.
The crate forbids unsafe code. Preserve explicit uncertainty and error outcomes.

Parity fixtures must remain derived from the pinned Python source, never from
Rust's output. Explain deliberate behavior changes and update the compatibility
contract. Preserve source attribution and licenses when porting code.

Dependency changes must pass both lockfile audits and the stable/MSRV matrix.
Weekly Dependabot updates are reviewed through CI, never automatically merged.
The independent triage app intentionally keeps its own manifest and lockfile to
exercise the library as a downstream consumer; do not fold it into a workspace
merely for consistency.

## Pull requests

Keep a change focused. Explain the problem, resulting behavior, compatibility
impact and supporting checks. Add a regression test for a behavior fix; documentation
changes should keep examples and links valid. Update the changelog for user-visible
changes. Commit titles should describe the change in plain language; no special
prefix convention is required.

Hosted provider calls, release uploads and repository administration are separate
maintainer decisions. An ordinary pull request does not need any of them. Do not
include a live call in default tests or assume a release will be made when a
change merges. [Governance](GOVERNANCE.md) describes review and support expectations.

## Contribution terms

Original project code uses the [MIT License](LICENSE). By intentionally submitting
a new contribution for inclusion, you provide it under those same MIT terms.
Upstream-derived portions retain their own licenses and notices, including the
CPython-derived code listed in [NOTICE.md](NOTICE.md). Only
submit material you have permission to contribute, and retain required notices
for third-party work. This does not transfer your copyright to the maintainer.

This repository currently has no separate CLA or DCO sign-off requirement. Any
future contributor agreement is a maintainer policy decision and must be stated
before it applies to contributions.
