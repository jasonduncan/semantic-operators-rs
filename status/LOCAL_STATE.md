# Local state

Project: `/Users/jason/PDLabsShare/Projects/semantic-operators-rs`.
Remote: `https://github.com/jasonduncan/semantic-operators-rs.git`; PUBLIC, confirmed intentional by Jason for crates.io; main.

Local toolchains: Rust 1.98.1 and installed Rust 1.85.0 (MSRV). CPython 3.13.13 generated
reference fixtures; CI tools use Python 3.12. Source siblings are pinned in
`docs/source-baseline.json` and are not Rust build dependencies. Cargo.lock is
committed; build/cache/archive/consumer output is ignored under target/.

No keys, environment file, live provider process, runtime database, paid budget,
reservation or deployment was created. Tests use temporary synthetic image files
and ephemeral localhost sockets and clean them on success. Runtime-driver guard
tests can print the default panic hook while catching and returning a typed error;
this is expected test evidence, not an escaping library panic.

Reference checkouts were read only. Dossier and session logs live outside this
repository under PD Labs Research/system-one and logs. The active registry is a
pointer to this status package. No new global runtime/service is needed to resume.

M7 adds integrations/triage-app/Cargo.lock and an independent application manifest.
The harness builds into ignored target/triage-app; direct cargo commands default
to integrations/triage-app/target (also ignored). It starts an ephemeral loopback
fixture server and terminates it on completion. No real credentials are read by
the harness; it overrides the child token with a fixed synthetic value and strips
proxies. The app itself reads TYPESAFE_API_KEY only when explicitly invoked.

The qualification runner/scorer are standard-library Python tools. Default planning
reads no credentials and makes no calls. The execution path checks the environment
key only after explicit --execute; no secret value was retained during preparation.
Run artifacts belong in ignored var/triage-check-* with owner-only POSIX permissions.
The runner builds with cached locked dependencies and records source/tool/binary
hashes. New tests mock processes and use temporary synthetic files, cleaned at exit.
No live run directory was created during preparation; model and call cap are unset.

Release preparation logs and archive receipt: ignored target/release-check/.
Publish dry-run reads registry metadata but uploads nothing. The Cargo credential
provider has not been inspected; no secret values were read or persisted.

## Published alpha 0.1.0-alpha.1 — 2026-10-17

Jason explicitly directed publication. Updated release wording, committed
30b827deebc5f25e4b31a9a9843437488f79458e, reran the archive/downstream checks,
all-feature doctest and publish dry-run successfully, then ran cargo publish --locked.
Cargo reported Published; registry created_at is 2026-10-18T00:02:35.550910Z.
The registry, downloaded archive and local archive share SHA-256
89da5d2536113c2cc55b2a645a95e29da1411415f93d53cb7cd14c77a8c7c8a4 (32 files, 54434 bytes).
Downloaded .cargo_vcs_info.json matches the published commit. Pushed annotated
v0.1.0-alpha.1 at that revision. Fresh crates.io consumers passed with default and
optional TypeSafe/images/blocking features; no model invocation was involved.

docs.rs initially returned 404 while processing; subsequent main/TypeSafe/image/
blocking/TimeLimited pages returned 200. Existing Cargo credentials were used through
Cargo; no credential value was printed or added to artifacts. Release CI 39081286724
is completing; exact final result will be appended. No hosted inference or model
quality acceptance occurred. The immutable release is public and installable.

## Rust review tooling — 2026-10-17

Raw review/audit/test logs reside in ignored `target/best-practices/`. Installed
nightly-aarch64-apple-darwin with Miri/rust-src (rustc 1.101.0-nightly,
5d8fec066 2026-10-16); default toolchain unchanged. cargo-audit 0.22.2 scans both
committed locks without advisory ignores. No hosted inference or new crate upload.
