# Local state

Project: `/Users/jason/PDLabsShare/Projects/semantic-operators-rs`.
Remote: `https://github.com/jasonduncan/semantic-operators-rs.git`; private; main.

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
