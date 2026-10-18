# Development handoff

2026-10-17 · alpha 0.1.0-alpha.1 published; publication and verification complete.

1. Inspect Git and read CURRENT.md/VALIDATION.md. Keep the repository public.
2. The published version is **semantic-operators 0.1.0-alpha.1**. Never re-upload it.
   Tag v0.1.0-alpha.1 and package VCS metadata identify
   30b827deebc5f25e4b31a9a9843437488f79458e. Registry/download/local checksums match;
   see [RELEASE](../docs/RELEASE.md). docs.rs optional modules are verified.
3. Publication used existing Cargo credentials without exposing values. Fresh
   crates.io consumers passed with default and optional features. Logs/receipts
   remain ignored under target/release-check. [Release CI 39081286724](https://github.com/jasonduncan/semantic-operators-rs/actions/runs/39081286724)
   passed all eight jobs at the published revision. No release work remains.
4. Future development starts from a selected change or issue and uses a new version
   for any further registry publication. Preserve the existing release tag.
5. Live-provider evaluation remains separate: exact model and call ceiling are
   still unset for the 12-ticket packet. Publication does not authorize those calls.

No service remains running. The uploaded release only changed release wording from
the verified candidate; Rust source and dependency declarations stayed unchanged.
Closing repository documentation records outcomes without altering the published
archive. Update status/validation and the active-project pointer after future work.
