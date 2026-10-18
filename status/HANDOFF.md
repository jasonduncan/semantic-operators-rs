# Development handoff

2026-10-17 · alpha.2 publication authorized and in progress.

1. Verify Git, CURRENT/VALIDATION and docs/RELEASE before resuming an interrupted upload.
2. Selected new version:0.1.0-alpha.2. Finish exact-commit CI, package consumers,
   publish dry-run and history scan, then upload using configured Cargo credentials.
3. If upload outcome is unknown, inspect crates.io before retrying. Verify archive
   checksum/VCS commit, fresh consumers and docs.rs; create immutable v0.1.0-alpha.2
   at the released revision and its GitHub prerelease. Preserve alpha.1/tag/draft.
4. Readiness unknowns remain documented; new release direction authorizes this
   upload without inventing legal/account sign-offs or activating OIDC proposals.
5. Close status/history and the registry pointer, with a clean/pushed check.

No live-model calls are part of this release. Logs remain in ignored target/release-alpha2.
