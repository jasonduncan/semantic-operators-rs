# Release procedure

Candidate: **semantic-operators 0.1.0-alpha.1**, MIT, edition 2024, Rust 1.85 minimum.
This is the existing alpha version, retained for the first release. Public GitHub
visibility is intentional. The manifest now allows crates.io only; no upload or
release tag has occurred. API stability, hosted model quality and performance are
not claimed. [Release notes](../CHANGELOG.md) remain marked unreleased until upload.

## Prepared package

The manifest includes source, synthetic examples, Cargo metadata/lockfile, CRATE.md,
CHANGELOG.md and LICENSE. It excludes status, source inventories, qualification data,
integrations, tool scripts and runtime artifacts. The package checker enforces that
allowlist and runs downstream consumers with default and optional features.
Keywords/category support registry discovery; docs.rs is configured to build all
features so optional modules are documented. The docs.rs URL is a publication target,
not evidence that a hosted page already exists.

## Verification before upload

Run on the exact clean candidate revision:

```sh
python3 tools/check_docs.py
python3 tools/check_package.py
cargo publish --dry-run --locked --offline --all-features
```

The offline dry-run performs packaging/build verification without upload. It needs
cached dependencies; fetch locked dependencies first on a fresh checkout. Hosted CI
also runs the dry-run and the existing platform/MSRV/feature/package checks.
Cargo may report that repository-only test targets were excluded: this is intentional;
those tests run in CI before packaging. Do not suppress a build or validation failure.

Inspect the archive list and SHA-256 in the validation receipt. Any later change to
packaged files requires a fresh dry-run. Check the release name/version immediately
before upload. A name lookup is not a reservation and dry-run does not prove publish
authentication or registry acceptance.

## Publication

The reviewable target is semantic-operators **0.1.0-alpha.1**. Once Jason directs
publication of that version:

1. Confirm clean/pushed Git state and passing candidate CI, recheck crates.io name
   and version, and configure the owner's Cargo credential provider. Never print or
   put tokens in commands, docs or chat.
2. Mark the changelog with the release date and replace unpublished wording in
   README.md/CRATE.md. Commit, repeat the package dry-run and verify changed docs.
3. Run `cargo publish --locked` on that exact revision. If upload outcome is unknown,
   inspect the registry before attempting anything again; do not assume failure.
4. Verify the registry version and package checksum, docs.rs build and exported
   feature documentation, then create tag `v0.1.0-alpha.1` at the published revision.
5. Record registry/docs.rs/tag receipts and refresh status. A hosted model check is
   separately scoped and is not implied by a published crate.

No automatic publish workflow or release credentials are introduced by this project.

## Reference and name checks

The official [Cargo publishing guide](https://doc.rust-lang.org/cargo/reference/publishing.html),
[manifest reference](https://doc.rust-lang.org/cargo/reference/manifest.html#the-publish-field)
and [docs.rs metadata guide](https://docs.rs/about/metadata) informed this setup.
On 2026-10-17 the crates.io API for semantic-operators and sparse index entries for
semantic-operators/semantic_operators returned 404, with serde 200 as a control.
This supports apparent availability, not a reserved name. Earlier API 403 results
were inconclusive; no placeholder was published to claim the name.
