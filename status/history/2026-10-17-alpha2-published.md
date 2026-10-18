# Alpha.2 release closed

Portfolio active; execution complete; handoff version 13.

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

Next: selected product issue or owner/governance follow-up.
