# Release procedure

## Alpha.2 release candidate — 2026-10-17

Jason explicitly requested a new release after the open-source review. The selected
next version is **0.1.0-alpha.2**, containing the cancellation fix, CPython notices
and combined MIT AND Python-2.0 metadata, API docs and repository maintenance work.
Publication is authorized; upload/registry/docs.rs verification is pending. Existing
Cargo credential-provider publication is retained for this release. Trusted-publishing
and main-protection proposals remain inactive; unverified ownership/account/policy
facts remain follow-ups, not falsely recorded as completed sign-offs.

## Previous published alpha.1

**semantic-operators 0.1.0-alpha.1 is published.** Jason explicitly authorized the
upload. Registry receipt: 2026-10-18T00:02:35.550910Z (2026-10-17 America/Chicago).
The published alpha metadata declares MIT; edition 2024, Rust 1.85 minimum; API remains unstable.

- [Crates.io version](https://crates.io/crates/semantic-operators/0.1.0-alpha.1)
- [Rust API docs](https://docs.rs/semantic-operators/0.1.0-alpha.1/semantic_operators/)
- [Release tag](https://github.com/jasonduncan/semantic-operators-rs/tree/v0.1.0-alpha.1)
- Published revision: `30b827deebc5f25e4b31a9a9843437488f79458e`.
- Archive SHA-256: `89da5d2536113c2cc55b2a645a95e29da1411415f93d53cb7cd14c77a8c7c8a4`.

Registry checksum, downloaded archive and locally verified archive match. Package
VCS metadata matches the tag. Fresh registry consumers run with default and optional
features. docs.rs serves the main page plus TypeSafe, image, blocking and timeout
APIs. Publication does not establish hosted model quality or performance.

## Prepared package

The manifest includes source, synthetic examples, Cargo metadata/lockfile, CRATE.md,
CHANGELOG.md, LICENSE, LICENSE-PYTHON and NOTICE.md. It excludes status, source inventories, qualification data,
integrations, tool scripts and runtime artifacts. The package checker enforces that
allowlist and runs downstream consumers with default and optional features.
Keywords/category support registry discovery; docs.rs is configured to build all
features so optional modules are documented. The hosted documentation and optional API pages were verified after upload.

## Verification before upload

Run on the exact clean candidate revision:

```sh
python3 tools/check_docs.py
python3 tools/check_package.py
cargo publish --dry-run --locked --all-features
```

The dry-run performs packaging/build verification without upload. It reads the
crates.io index; this new-crate dry-run did not work with --offline, even with cached
dependencies. The separate package/consumer checker uses cached dependencies offline. Hosted CI
also runs the dry-run and the existing platform/MSRV/feature/package checks.
Cargo may report that repository-only test targets were excluded: this is intentional;
those tests run in CI before packaging. Do not suppress a build or validation failure.

Inspect the archive list and SHA-256 in the validation receipt. Any later change to
packaged files requires a fresh dry-run. Check the release name/version immediately
before upload. A name lookup is not a reservation and dry-run does not prove publish
authentication or registry acceptance.

## Future publication

The first alpha is already published; never repeat its upload. For an authorized
new version, substitute its version and tag in the following procedure:

1. Confirm clean/pushed Git state and passing candidate CI, recheck crates.io name
   and version, and configure the owner's Cargo credential provider. Never print or
   put tokens in commands, docs or chat.
2. Mark the changelog with the release date and replace unpublished wording in
   README.md/CRATE.md. Commit, repeat the package dry-run and verify changed docs.
3. Run `cargo publish --locked` on that exact revision. If upload outcome is unknown,
   inspect the registry before attempting anything again; do not assume failure.
4. Verify the registry version and package checksum, docs.rs build and exported
   feature documentation, then create the selected new version tag at the published revision.
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

## Open-source readiness corrections in alpha.2

Current source retains original MIT terms and adds CPython attribution/license
notices for the accurate-summation routines. Its aggregate SPDX expression is
`MIT AND Python-2.0`. The alpha.2 archive includes NOTICE.md and LICENSE-PYTHON; the
immutable alpha.1 archive and registry metadata do not gain these corrections.
Do not represent current-source checks as changes to that published artifact.

The [maintainer setup packet](release/SETUP.md) contains concrete branch-protection
and trusted-publishing proposals. They need the named reviewers/account setup and
readiness sign-offs before activation. Publication still requires an exact new
version. Changelog entries are maintained by hand; no auto-release on merge.
