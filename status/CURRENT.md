# Current state

2026-10-17 · portfolio active · execution ready: verified alpha, upload direction pending.

Prepared **semantic-operators 0.1.0-alpha.1** for crates.io. The version retains its
alpha designation and unstable API. Manifest publication is now restricted to
crates.io (`publish = ["crates-io"]`); this is a local package setting, not an upload.
Public GitHub distribution was explicitly confirmed by Jason.

Release notes are included in the 32-file package. Registry keywords/category and
docs.rs all-feature configuration are set. The package checker enforces the archive
allowlist and exercises extracted default/optional consumers. Local package audit,
all-feature doctest and `cargo publish --dry-run --locked --all-features` passed at
2640da883ee6b5100526d4052d55917bf50396a9. No registry upload or release tag exists.

The dry-run needs registry index access; --offline failed before packaging and was
removed from that command. Ordinary package checks still use cached dependencies.
Independent release review corrected one frame-timestamp compatibility statement;
no Rust behavior changed. [CI 39080731154](https://github.com/jasonduncan/semantic-operators-rs/actions/runs/39080731154)
passed all eight jobs at the same 2640da8 revision. Closing edits only record
status/evidence; package inputs, tests and workflow match that passing run.

Next: get direction for the exact alpha upload, then follow
[RELEASE](../docs/RELEASE.md). The release procedure includes publication wording,
final clean-package check, registry/checksum/docs.rs verification and version tag.
Credential authentication and registry acceptance have not been tested.

M1–M7 remain complete. The optional 12-ticket live-provider packet still needs exact
model/call scope; no hosted inference or semantic acceptance is claimed. It is not
a blocker for this package release preparation.
