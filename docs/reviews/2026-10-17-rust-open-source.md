# Rust open-source readiness assessment — 2026-10-17

Applied owner-supplied **Rust Open-Source Release Readiness Checklist**, SHA-256
`748c4bb28c33a0e22a563110d7909e6676125c582fa4fab0643fdc5474818ba8`.
Starting HEAD: `48006c80bd7d281958acc0d383671af150f55948`.

**Assessment: technical remediation implemented; full release sign-off remains pending.**
This is a post-publication assessment of main. Already-published alpha.1 is immutable;
new attribution, metadata, documentation and CI changes are unreleased. No new crate
upload or public launch announcement is part of this review.

Checked means evidence obtained or an explicit engineering alternative documented.
Unchecked means unresolved. N/A and engineering waivers are shown explicitly; they
do not impersonate legal/account-owner approval. Jason owns final release approval;
Codex performed technical verification on 2026-10-17. All ten hosted jobs passed on source d493ac9; exact receipts are in
[VALIDATION](../../status/VALIDATION.md).

## Decisions needed before a subsequent launch

- Confirm code/fixture rights and trademark review; select DCO/CLA or an explicit waiver.
- Name a second crate owner/reviewer and verify owner 2FA.
- Select a real private conduct contact/policy.
- Approve/activate the [branch and trusted-publishing setup](../release/SETUP.md),
  then choose an exact new version and complete release sign-offs.

## Legal and licensing

Owner: Jason Duncan; technical reviewer: Codex.

- [ ] **1. Confirm the company owns all code in the repo: employee and contractor agreements cover it, and nothing came from a prior employer or client codebase.**
  PENDING: Jason: confirm rights and employee/contractor/client restrictions. Source license files cannot establish agreements.

- [x] **2. License decided and approved. If dual MIT/Apache-2.0: add `LICENSE-MIT` and `LICENSE-APACHE` at the repo root.**
  DONE: Original project MIT decision retained from the authorized alpha. LICENSE-PYTHON/NOTICE preserve CPython portion terms; this is not a dual-Apache relicensing.

- [x] **3. `license = "MIT OR Apache-2.0"` (SPDX expression) set in every published `Cargo.toml`.**
  DONE: Alternative chosen license: current combined package is MIT AND Python-2.0; the unpublished consumer is MIT.

- [x] **4. README has a License section and the standard inbound contribution clause (contributions are dual-licensed under the same terms).**
  DONE: README License and CONTRIBUTING Contribution terms use MIT for new original contributions and preserve upstream terms.

- [ ] **5. Decide contributor terms: DCO sign-off (lighter, common) or a CLA (only if relicensing may be needed later).**
  PENDING: No separate DCO/CLA currently required; Jason must choose DCO, CLA, or explicitly waive this checklist requirement.

- [x] **6. Audit dependency licenses with `cargo deny check licenses`; allow-list only licenses compatible with ours. Flag any GPL/AGPL or unknown licenses.**
  DONE: cargo-deny 0.20.2 passed both manifests. Eight explicit license IDs; no GPL-only/AGPL/unknown package accepted. Alternative LGPL branches are not selected.

- [x] **7. Vendored or copied code (snippets, test fixtures, assets) has its original license and attribution preserved.**
  DONE: NOTICE.md and complete LICENSE-PYTHON preserve CPython attribution; both reference semops licenses are MIT/Jason2026. Current source/package corrected; old alpha archive unchanged.

- [ ] **8. Project and crate name checked for trademark conflicts and confirmed available on crates.io, GitHub, and any domain we want.**
  PENDING: Crate and repository are already owned/published. Trademark clearance is not established. No domain requested.

## Repository hygiene and secrets

Owner: Jason Duncan; technical reviewer: Codex.

- [x] **9. Run a secrets scanner over full history (`gitleaks detect` or `trufflehog git`), including all branches and tags.**
  DONE: Gitleaks 8.30.1 scanned all fetched branches/tags/full history, initially 22 commits, zero findings. CI repeats with full fetch, redaction and pinned binary hash.

- [x] **10. Any secret ever committed is rotated, even if later deleted. Removing it from history does not make it safe.**
  NA: No secret identified by the scan; no rotation performed or claim that every possible secret is detectable.

- [x] **11. Decide history strategy: publish from a fresh repo with a squashed initial commit (simplest, safest), or rewrite with `git filter-repo` if history is worth keeping.**
  WAIVED: Preserve already-public history and immutable published tag. No finding justifies destructive rewriting; a fresh-history prelaunch strategy no longer applies.

- [x] **12. Remove internal references: hostnames, IPs, internal URLs, ticket links, customer or partner names, employee names in TODOs, internal Slack/Jira jargon.**
  DONE: Removed machine-specific checkout paths from current source inventory/local-state document. No other internal host/customer/ticket references found in tracked-tree search. Historical paths remain public; this is not a history erasure.

- [x] **13. Review commit author names and emails; switch to GitHub noreply addresses if needed.**
  DONE: Reviewed all author identities: one existing maintainer identity. Preserved already-public attribution; did not rewrite history or silently change user Git identity.

- [x] **14. Delete stale branches, experimental code, dead feature flags, and large binaries not needed by users.**
  DONE: Only main and active Dependabot branches; no stale branch deletion justified. Largest tracked file is a 107433-byte synthetic fixture; no large binary payloads.

- [x] **15. `.gitignore` covers `target/`, editor files, `.env`, and local config.**
  DONE: .gitignore covers build output, credentials/env, editor directories, swap files and local configuration; independent consumer has its own target ignore.

- [x] **16. No hardcoded paths, credentials, or company-specific defaults in config, examples, or tests.**
  DONE: Examples/config/tests use synthetic input and explicit configuration. Source checkout paths removed from inventory; fixture regeneration takes --source.

- [x] **17. Repo lives under the intended GitHub org with the final name; description, topics, and website fields filled in.**
  DONE: Retain established jasonduncan/semantic-operators-rs public repository. Updated stale description, docs.rs homepage and five topics; no organization transfer inferred.

## Cargo.toml metadata and crate structure

Owner: Jason Duncan; technical reviewer: Codex.

- [x] **18. All fields above filled; `categories` use slugs from the official crates.io category list.**
  DONE: Metadata includes discovery fields, edition2024/MSRV1.85, repo/docs, packaged CRATE.md and composite SPDX. Homepage omitted from Cargo because repository/docs already supplied.

- [x] **19. `include` (or `exclude`) set so the package has no test data dumps, CI files, or internal docs. Verify with `cargo package --list`.**
  DONE: Explicit allowlist and actual-archive checker exclude private status, fixtures, tools and CI; require LICENSE, LICENSE-PYTHON and NOTICE.md.

- [x] **20. Package size is reasonable (crates.io limit is 10 MB; aim far lower).**
  DONE: Verified package remains far below the 10 MB threshold; exact current archive size in validation.

- [x] **21. MSRV chosen deliberately and written in `rust-version` and the README.**
  DONE: Cargo/README/VERSIONING state Rust 1.85; CI tests it on all three platforms.

- [x] **22. Features: sensible `default` set, each feature documented, no feature that is required for the crate to compile. Optional deps use `dep:` syntax.**
  DONE: Empty defaults; all optional features documented; dep syntax retained; all 10 distinct feature closures checked.

- [x] **23. Workspace: internal-only crates marked `publish = false`; every path dependency on a published crate also has a `version`.**
  DONE: Single published library; standalone consumer has publish=false and a local path dependency. That unpublished integration intentionally does not need a registry version.

- [x] **24. Workspace crates that will be published have their names reserved and a known publish order (leaf crates first).**
  NA: One published crate; no workspace publish ordering or placeholder name reservation.

- [x] **25. `Cargo.lock` committed (current Cargo guidance for both libraries and binaries).**
  DONE: Both library and independent consumer locks committed.

- [x] **26. Unused dependencies removed (`cargo machete` or `cargo udeps`).**
  DONE: cargo-machete 0.9.2 recursive scan found no unused dependencies.

## Code quality and public API

Owner: Jason Duncan; technical reviewer: Codex.

- [x] **27. `cargo fmt --check` clean; a `rustfmt.toml` committed if we deviate from defaults.**
  DONE: Default rustfmt clean; no custom rustfmt config needed.

- [x] **28. `cargo clippy --all-targets --all-features -- -D warnings` clean.**
  DONE: Strict all-target/all-feature Clippy passes.

- [x] **29. Crate-level lints set: `#![warn(missing_docs)]`, and `#![forbid(unsafe_code)]` if the crate has no `unsafe`.**
  DONE: Cargo lint policy denies missing_docs and forbids unsafe_code; focused crate attributes enforce errors/panics docs and no accidental printing.

- [x] **30. Every `unsafe` block has a `// SAFETY:` comment explaining the invariant; reviewed by a second engineer; tested under Miri where possible.**
  NA: No unsafe blocks; no fabricated second-engineer safety sign-off.

- [x] **31. Public API reviewed item by item (`cargo public-api` lists it). Anything not intended for users becomes `pub(crate)` or moves behind a `#[doc(hidden)]` internal module.**
  DONE: Reviewed source and generated nightly rustdoc JSON: 47 structs, 9 enums, 4 extension traits,6 aliases,7 constants, 19 modules, 199 local functions/methods including generated/derived, 47 public fields. No unintended public helper found. cargo-public-api not used.

- [x] **32. Public enums and structs that may grow are `#[non_exhaustive]`.**
  WAIVED: Preserve existing exhaustive enums and constructible public records: retrofitting non_exhaustive breaks the published API. Reconsider at a deliberate breaking release.

- [x] **33. Traits users should not implement are sealed.**
  DONE: Provider, Transport, TypedQuestion and AnswerType are intentional application extension points, so remain open.

- [x] **34. Third-party types are not exposed in the public API unless deliberate (their major bumps would force ours).**
  DONE: IndexMap ordering/reexport and serde_json::Value state/wire/raw/context are deliberate public dependencies; serde traits are deliberate. Their breaking changes require compatibility review.

- [x] **35. Errors use a proper error type (e.g. `thiserror`) implementing `std::error::Error`; no `anyhow` or `Box<dyn Error>` in library signatures.**
  DONE: Structured Error and FlowFailure implement std::error::Error. No anyhow/Box<dyn Error> library signatures; thiserror is not required.

- [x] **36. No `unwrap`/`expect`/`panic!` on paths reachable from user input; documented `# Panics` where panics are intended.**
  DONE: Previous source review retained only two constant NonZeroUsize expects (4/8); recoverable caller errors use Result. No new runtime code in this pass.

- [x] **37. Library code logs through `log` or `tracing`, never `println!`.**
  DONE: Library prints/logs no user data; no logging facade is needed solely to satisfy this item. Examples may print.

- [x] **38. Naming follows the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) (`as_`/`to_`/`into_`, constructors, builders).**
  DONE: Public constructors/accessors/builders reviewed against Rust naming conventions.

## Documentation

Owner: Jason Duncan; technical reviewer: Codex.

- [x] **39. One-paragraph pitch: what it is, who it is for, why it exists.**
  DONE: README opening explains typed semantic operations, uncertainty and intended Rust consumers.

- [x] **40. Badges: crates.io version, docs.rs, CI status, license, MSRV.**
  DONE: README version/docs/CI/license/MSRV badges present.

- [x] **41. Install snippet (`cargo add crate-name`, or `cargo install` for CLIs).**
  DONE: Cargo add plus exact-prerelease manifest instructions supplied.

- [x] **42. A minimal working example that compiles as-is.**
  DONE: Complete standalone README example compiled and ran offline.

- [x] **43. Feature flags table, MSRV policy, project status (experimental / stable), and links to docs, CONTRIBUTING, and License.**
  DONE: Features table, alpha status, MSRV policy and contributor/license links supplied.

- [x] **44. Crate-level docs in `lib.rs` (or `#![doc = include_str!("../README.md")]` so the README examples become doctests).**
  DONE: lib.rs includes packaged CRATE.md; examples execute as doctests.

- [x] **45. Every public item documented; key types have an `# Examples` section; fallible functions have `# Errors`.**
  DONE: Public docs enforced; selected answer/operator/region examples and fallible API Errors contracts present. Four doctests pass.

- [x] **46. `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features` builds clean (catches broken intra-doc links).**
  DONE: Warnings-denied rustdoc builds; docs checker validates local links/provenance.

- [x] **47. Feature-gated items labeled on docs.rs via `#![cfg_attr(docsrs, feature(doc_cfg))]`.**
  DONE: doc_cfg labels on gated APIs; nightly docs.rs HTML/JSON checked and labels inspected; CI now reproduces that configuration.

- [x] **48. `examples/` directory with 1 to 3 realistic programs, run in CI.**
  DONE: Five focused synthetic programs retained instead of reducing useful coverage to three; CI now runs every one.

- [x] **49. `CHANGELOG.md` started (Keep a Changelog format), with an entry for the first release.**
  DONE: Dated first-release and Unreleased sections retained with corrections/change notes.

- [x] **50. Architecture or design notes for contributors, if the codebase is non-trivial.**
  DONE: DESIGN/API/COMPATIBILITY/DEVELOPMENT/TESTING explain architecture and contributor workflow.

## Testing and CI

Owner: Jason Duncan; technical reviewer: Codex.

- [x] **51. Unit and integration tests cover the core paths; doctests run (`cargo test --all-features` includes them).**
  DONE: Prior source review passed 54 default / 88 all-feature tests and 4 doctests; updated CI passed full suites.

- [x] **52. Test matrix: stable on Linux, macOS, and Windows, plus the declared MSRV on Linux. Add beta or nightly as non-blocking if useful.**
  DONE: Linux/macOS/Windows stable and1.85 matrix; nightly docs build additionally validates feature labels.

- [x] **53. Feature combinations checked: `cargo hack check --feature-powerset` (or `--each-feature` if the powerset is too large), plus `--no-default-features`.**
  DONE: Explicit powerset closure equivalent covers all 10 effective sets from 16 raw subsets; empty features included. Local all-target checks and hosted tests for each passed.

- [x] **54. `cargo semver-checks` runs on PRs once 0.1.0 is out, to catch accidental breaking changes.**
  NA: Only0.1.0-alpha.1 exists; no stable0.1.0 baseline yet. Automated semver gate deferred until non-prerelease baseline, not falsely claimed.

- [x] **55. Miri job for any crate with `unsafe`.**
  NA: Unsafe forbidden; prior bounded 12-test Miri run passed. No mandatory unsafe-specific Miri job applies.

- [x] **56. Examples and benchmarks compile in CI (`cargo check --all-targets`).**
  DONE: All-target checks compile examples; synthetic examples also run. No performance benchmark claim.

- [x] **57. GitHub Actions pinned to commit SHAs; workflows use least-privilege `permissions:`.**
  DONE: All active uses entries pinned to verified40hex SHAs; read-only permissions. Proposed publish job alone requests id-token write behind environment approval.

- [x] **58. CI does not depend on internal infrastructure, private registries, or secrets that forks cannot access.**
  DONE: Public dependencies/runners and synthetic fixtures; no CI secrets or internal infrastructure required.

- [x] **59. Tests do not hit live external services, or those tests are gated behind an opt-in flag.**
  DONE: Normal tests use scripted or loopback transports. Live runner remains default-off with separate model/budget scope.

- [x] **60. Flaky tests fixed or quarantined before launch.**
  DONE: Previous stale local artifact issue resolved and reproduced successfully in fresh CI. Cancellation regression now fails immediately rather than hanging. No known flaky test is suppressed.

## Security and supply chain

Owner: Jason Duncan; technical reviewer: Codex.

- [x] **61. `cargo deny check` (advisories, bans, licenses, sources) passes; `deny.toml` committed and run in CI.**
  DONE: Full fresh cargo-deny advisories/bans/licenses/sources passed both locks. Duplicate base64/windows-sys warnings remain explicit; no security/license ignores.

- [x] **62. `cargo audit` (RustSec advisories) clean, also scheduled weekly in CI.**
  DONE: Fresh strict cargo-audit passes both locks; weekly/change-triggered workflow retained.

- [x] **63. Dependabot or Renovate enabled for Cargo and GitHub Actions.**
  DONE: Dependabot covers both Cargo manifests and actions; active update PRs observed; no auto-merge.

- [x] **64. `SECURITY.md` explains how to report vulnerabilities privately; GitHub private vulnerability reporting turned on.**
  DONE: SECURITY.md added; GitHub private vulnerability reporting enabled/read back. Secret scanning and push protection also enabled.

- [ ] **65. crates.io trusted publishing configured for the GitHub repo and release workflow; no `CARGO_REGISTRY_TOKEN` stored in repo secrets.**
  PENDING: No repository Actions secrets/environments observed. Inert pinned trusted-publishing workflow and exact setup tuple prepared in docs/release; owner account/environment setup not performed.

- [ ] **66. At least two crate owners (or a GitHub team owner) so the crate is not tied to one person.**
  PENDING: Registry reports sole owner jasonduncan. Need a named trusted second owner/team; none invented or invited.

- [ ] **67. 2FA required on the GitHub org and enabled on every crates.io owner's GitHub account.**
  PENDING: Personal repo makes org policy N/A; owner 2FA was not exposed by API and is unverified.

- [ ] **68. `main` branch protected: required reviews, required CI checks, no force pushes.**
  PENDING: Main currently unprotected. Concrete required-review/CI/no-force-push policy prepared; second reviewer needed before activation to avoid blocking sole-maintainer contributions.

- [x] **69. Release tags protected; consider signed tags.**
  DONE: Active Immutable release tags ruleset 25325227 prevents updates/deletion of v* tags with no bypass. Existing annotated tag preserved; signing not claimed.

## Versioning and release process

Owner: Jason Duncan; technical reviewer: Codex.

- [x] **70. Initial version agreed (`0.1.0` recommended) and the stability promise stated in the README.**
  DONE: Already authorized/published 0.1.0-alpha.1; README states experimental API. No new version selected/uploaded.

- [x] **71. Versioning policy written down: what counts as breaking, how MSRV bumps are treated (commonly a minor bump), deprecation period.**
  DONE: VERSIONING.md records prerelease pinning, breaking changes, MSRV increases and absence of a fixed deprecation window.

- [ ] **72. Release automation chosen: `release-plz` (opens release PRs, updates CHANGELOG, publishes on merge) or `cargo-release` (manual, local).**
  PENDING: Current explicit manual Cargo procedure retained. OIDC workflow proposal prepared; choosing/activating replacement automation remains a maintainer decision, not auto-release on merge.

- [x] **73. Changelog generation decided: hand-written, or generated from Conventional Commits with `git-cliff`.**
  DONE: Hand-maintained CHANGELOG; plain informative commits allowed.

- [ ] **74. Tag format agreed (`v0.1.0`; for workspaces, `crate-name-v0.1.0`) and GitHub Releases created from tags with notes.**
  PENDING: v<version> format established; draft GitHub release prepared for existing alpha tag, not publicly launched before remaining sign-offs.

- [ ] **75. Release workflow publishes via trusted publishing and has been dry-run end to end (`cargo publish --dry-run`).**
  PENDING: Package consumers and Cargo dry-run validated in CI; trusted-publishing template is syntax-reviewed only, not end-to-end authenticated.

- [x] **76. Publishing is permanent: versions can be yanked but never deleted or overwritten. Everyone involved understands this.**
  DONE: RELEASE/VERSIONING document immutable versions, verify-before-retry and no tag replacement. No assertion about unobserved human understanding.

- [x] **77. Prebuilt binaries for target platforms via `dist` (cargo-dist) or a custom workflow, attached to GitHub Releases with checksums.**
  NA: Published artifact is a library; standalone triage app is an unpublished integration example.

- [x] **78. `cargo binstall` metadata so users can skip compiling.**
  NA: No installable published binary.

- [x] **79. Optional: Homebrew tap, shell installer, or container image.**
  NA: No Homebrew/container distribution requested.

## Community and governance files

Owner: Jason Duncan; technical reviewer: Codex.

- [x] **80. `CONTRIBUTING.md`: dev setup, how to run tests and lints, PR expectations, DCO/CLA requirement, commit message style.**
  DONE: CONTRIBUTING includes setup/checks/PR expectations/commit style and actual current contribution terms; formal DCO/CLA choice remains item5.

- [ ] **81. `CODE_OF_CONDUCT.md` (Contributor Covenant or the Rust Code of Conduct) with a real enforcement contact.**
  PENDING: No real private conduct-report contact supplied. Do not fabricate CODE_OF_CONDUCT enforcement or repurpose security reporting; owner must select contact/policy.

- [x] **82. Issue templates (bug report, feature request) and a PR template with a checklist.**
  DONE: Bug/feature templates and PR checklist present; config routes vulnerabilities privately and questions to Discussions.

- [x] **83. `CODEOWNERS` so PRs auto-request the right reviewers.**
  DONE: CODEOWNERS assigns @jasonduncan; it does not substitute for enforced review.

- [x] **84. Labels created: `good first issue`, `help wanted`, `bug`, `enhancement`, `breaking-change`; a few starter issues seeded.**
  DONE: Required labels exist; seeded focused starter issue #3 for Unicode diagnostic parity. No artificial backlog generated.

- [x] **85. GitHub Discussions enabled (or a stated alternative channel) so questions stay out of issues.**
  DONE: Discussions enabled and linked for usage/design questions; question issue template removed.

- [x] **86. Governance and maintainer expectations stated: who decides, expected triage time, what the company commits to supporting.**
  DONE: GOVERNANCE names the maintainer and decision path, explicitly promises no SLA/support window/company staffing commitment.

- [x] **87. Optional: `FUNDING.yml`, roadmap issue or doc.**
  NA: No funding requested; existing development plan serves roadmap needs.

## Launch-day runbook disposition

The existing alpha upload, tag, registry consumer and docs.rs receipts remain in
[RELEASE](../RELEASE.md). They do not satisfy missing legal/account sign-offs by
retroactive inference. No new freeze/version/tag/upload/announcement occurred.
The existing alpha GitHub release remains a draft. No 48–72-hour triage staffing
commitment was made. The next launch must use the exact approved candidate and
repeat scan/CI/dry-run/registry/docs.rs checks in the supplied skill order.

## Sign-off ledger

| Area | Accountable owner | Technical evidence | Owner sign-off |
|---|---|---|---|
| Legal/licensing | Jason Duncan | Dependency/license notices checked | Pending rights, contributor terms and trademark confirmation |
| History/secrets | Jason Duncan | Codex: full-history scanner/current-tree review | Pending owner acceptance of existing public attribution/history |
| Cargo/structure | Jason Duncan | Codex: package and metadata verified | Pending final release sign-off |
| Code/API | Jason Duncan | Codex: manual/rustdoc API review, compatibility retained | Pending final release sign-off |
| Documentation | Jason Duncan | Codex: examples/docs/link checks | Pending final release sign-off |
| Testing/CI | Jason Duncan | Codex: local checks and all ten hosted jobs passed | Pending final release sign-off |
| Security/supply chain | Jason Duncan | Scans pass; reporting/secrets/tag settings read back | Pending owners, 2FA,branch/OIDC setup |
| Release process | Jason Duncan | Existing alpha verified; new setup proposal inert | Pending exact new version and authenticated workflow |
| Community/governance | Jason Duncan | Onboarding/templates/discussions/starter issue | Pending conduct contact/policy |
| Final go/no-go | Jason Duncan | **No-go for a new launch under this checklist** | Not signed |

## External changes and source references

Enabled private vulnerability reporting, GitHub secret scanning/push protection,
Discussions and immutable version-tag rules. Updated repository description/topics/
docs homepage and added breaking-change label. Created [starter issue 3](https://github.com/jasonduncan/semantic-operators-rs/issues/3)
and a draft release for the existing alpha tag. No new owner, branch protection,
credential, public release announcement or crates.io upload was created.

- [CPython pinned source and notices](https://github.com/python/cpython/tree/01104ce1beb3135c2e0c01ec835b994c1f55a1c0): accurate summation attribution.
- [Rustdoc feature labels](https://doc.rust-lang.org/nightly/unstable-book/language-features/doc-cfg.html).
- [GitHub rulesets](https://docs.github.com/en/rest/repos/rules).
- [Private vulnerability reporting](https://docs.github.com/en/code-security/how-tos/report-and-fix-vulnerabilities/configure-vulnerability-reporting/configure-for-a-repository).
- [Cargo compatibility](https://doc.rust-lang.org/cargo/reference/semver.html).
- [Trusted publishing](https://crates.io/docs/trusted-publishing).
