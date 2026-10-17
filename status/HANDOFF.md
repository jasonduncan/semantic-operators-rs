# Development handoff

2026-10-17 · prepared alpha 0.1.0-alpha.1; all eight final CI jobs passed; upload direction pending.

1. Inspect Git and read CURRENT.md/VALIDATION.md. Public visibility is intentional.
2. [CI 39080731154](https://github.com/jasonduncan/semantic-operators-rs/actions/runs/39080731154)
   passed all eight jobs at 2640da883ee6b5100526d4052d55917bf50396a9, including
   publish dry-run and platform/MSRV/features/package checks. Closing edits are
   documentation/status only, with unchanged tested package inputs and workflow.
3. Local package checker, doctest and dry-run pass. Runtime receipt/logs are under
   ignored target/release-check; the recorded candidate archive has 32 files,
   54439 bytes and SHA-256 a7496cd603c3e9009b73d01647372e6172f9d8fe833ce4804b325d45e08d3c90.
   This hash identifies that dry-run build, not a future publication archive.
4. Actual upload still needs direction for **semantic-operators 0.1.0-alpha.1**.
   Once selected, follow [RELEASE](../docs/RELEASE.md): update unreleased wording,
   commit/verify the exact archive and upload, verify registry/docs.rs, then tag.
   Do not infer that dry-run tested owner credentials or reserved the crate name.
5. The optional live-provider packet remains separately scoped by model/call limit.
   Do not use a library release as authorization for hosted inference.

Manifest publish=false was replaced with the crates.io-only allowlist for release
preparation. This does not introduce automatic publication. No upload, tag, hosted
model request or credential value was introduced. Closing documentation changes
should record the final CI receipt and refresh the clean/pushed registry pointer.
