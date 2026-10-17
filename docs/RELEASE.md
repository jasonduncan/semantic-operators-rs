# Release procedure

Nothing is released. Version `0.1.0-alpha.1` denotes the implemented alpha candidate and
`publish = false` prevents accidental crates.io publication.

On 2026-10-17 the crates.io API lookup for `semantic-operators` returned HTTP 403;
the web lookup could not retrieve it either. This is **unverified availability**,
not evidence that the name is taken or free. Creating the GitHub repository does
not reserve the crate. Do not publish an empty placeholder to claim the name.


A secondary read-only check of the official `rust-lang/crates.io-index` GitHub
repository returned 404 for both `se/ma/semantic-operators` and
`se/ma/semantic_operators`, while the known `se/rd/serde` entry was accessible.
The name therefore appears unregistered in that index at this observation, but
this is not a registry availability guarantee or reservation. Recheck at release.

Before a release candidate:

1. Finish M1–M6 gates, including parity, package/consumer and supported-platform CI.
2. Review README/examples against actual exports; identify supported provider/model
   capabilities and live verification limits. Keep source attribution/license.
3. Recheck crates.io for hyphen/underscore conflicts and available package name.
4. Inspect `cargo package --list` and package contents, dependencies, license and
   sensitive-file exclusions. Keep private status/evidence outside the package.
5. Choose version, minimum Rust version, changelog, and visibility with the owner.
6. With explicit publication authorization, remove publish=false, run the package
   checks and `cargo publish --dry-run`, then publish and verify registry/docs.rs.
7. Tag the exact verified revision and record receipts and final compatibility scope.

Jason confirmed on 2026-10-17 that the GitHub repository is intentionally PUBLIC
for eventual crates.io distribution. Keep that visibility. The implementation and
CI are development evidence, not a published registry release. There are no
release workflows, registry credentials, or automatic publishing in this project.


## Current release preparation

Candidate: semantic-operators 0.1.0-alpha.1, MIT, edition 2024, Rust 1.85 minimum.
The archive explicitly includes src, examples, Cargo metadata, CRATE.md and LICENSE;
project status, qualification datasets, local paths in design/source inventory,
and run artifacts are excluded. Retain the current alpha designation until an
explicit version decision; semantic model qualification remains unperformed.

Before upload, confirm the version/changelog, remove publish=false for the selected
release, rerun the package audit and `cargo publish --dry-run`, then publish the
verified revision using the owner's Cargo credential provider. Never place a token
in a command, document or chat. Verify the resulting crates.io/docs.rs pages and tag.
The [official Cargo publishing guide](https://doc.rust-lang.org/cargo/reference/publishing.html)
was rechecked on 2026-10-17; packaging/dry-run verifies the archive before upload.

## Name and package recheck — 2026-10-17

The crates.io API at /api/v1/crates/semantic-operators returned 404. Sparse index
entries for semantic-operators and semantic_operators also returned 404, while
serde returned 200. These current observations support apparent availability,
not reservation. Recheck immediately before upload.

`python3 tools/check_package.py` passed: 31 files, 202.4 KiB unpacked / 52.4 KiB
compressed; default dependency isolation and both extracted-package consumers
(default and optional features). `publish=false` is still set; no upload occurred.
