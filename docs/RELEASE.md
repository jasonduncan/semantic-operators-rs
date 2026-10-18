# Release procedure

## Alpha.2 publication receipt — 2026-10-17

Jason explicitly requested this new release. `cargo publish --locked` succeeded
at source `291c8cca0264ed341089af39001d52469e4dd1c9`; crates.io created_at is
`2026-10-18T01:34:47.109219Z`. Version **0.1.0-alpha.2** is not yanked, declares
`MIT AND Python-2.0`, and retains Rust 1.85/edition 2024.

- [Crates.io](https://crates.io/crates/semantic-operators/0.1.0-alpha.2)
- [API docs](https://docs.rs/semantic-operators/0.1.0-alpha.2/semantic_operators/)
- [GitHub prerelease](https://github.com/jasonduncan/semantic-operators-rs/releases/tag/v0.1.0-alpha.2)
- Archive: 34 files, 64598 bytes; SHA-256
  `24fdf2737704e518d87bb9174dc16436d7ac408c6eab12f32f82387ada59e8f3`.
- Annotated tag `v0.1.0-alpha.2`: object `b315364d356af21bf6e7ba975f6a428a3d770f00`,
  dereferencing to the published source SHA. Remote tag verified. GitHub release
  is published (not draft), marked prerelease, at `2026-10-18T01:35:24Z`.

Local candidate, registry checksum and downloaded archive match. Archive VCS
metadata matches the released revision without a dirty marker. Downloaded NOTICE.md
and LICENSE-PYTHON match source. Fresh crates.io consumers passed with default
features and with TypeSafe/images/blocking; no model dispatch. docs.rs initially
returned 404 while building, then all five checked main/TypeSafe/image/blocking/
TimeLimited pages returned 200. Initial curl metadata lookup returned 403; the
subsequent Python registry verification and Cargo upload succeeded.

Exact release-source hosted checks all passed (ten jobs total):

- [CI 39086636460](https://github.com/jasonduncan/semantic-operators-rs/actions/runs/39086636460):
  eight platform/MSRV/documentation/feature/package jobs.
- [Supply chain 39086636454](https://github.com/jasonduncan/semantic-operators-rs/actions/runs/39086636454):
  dependency advisories/licenses/bans/sources and unused dependencies.
- [Secret scan 39086636446](https://github.com/jasonduncan/semantic-operators-rs/actions/runs/39086636446):
  full-history scan.

Local four doctests, extracted package consumers, all-feature publish dry-run,
full-history Gitleaks, docs/provenance and whitespace checks passed before upload.
Existing Cargo credential provider was used; no credential value was printed.
No live model calls, performance qualification or additional governance/account
sign-off is implied. Main protection/trusted publishing remain proposals. Prior
alpha.1 archive/tag and its old GitHub draft are preserved. Closing commit changes
only docs/status; the hosted receipts above identify the exact release revision.

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
