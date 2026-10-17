# Release procedure

Nothing is released. Version `0.1.0-alpha.1` denotes the implemented alpha candidate and
`publish = false` prevents accidental crates.io publication.

On 2026-10-17 the crates.io API lookup for `semantic-operators` returned HTTP 403;
the web lookup could not retrieve it either. This is **unverified availability**,
not evidence that the name is taken or free. Creating the GitHub repository does
not reserve the crate. Do not publish an empty placeholder to claim the name.


A secondary read-only check of the official `rust-lang/crates.io-index` GitHub
repository returned 404 for both `se/ma/semantic-operators` and
`se/ma/semantic_operators`, while the known `se/rd/serde` entry was accessible.
The name therefore appears unregistered in that index at this observation, but
this is not a registry availability guarantee or reservation. Recheck at release.

Before a release candidate:

1. Finish M1–M6 gates, including parity, package/consumer and supported-platform CI.
2. Review README/examples against actual exports; identify supported provider/model
   capabilities and live verification limits. Keep source attribution/license.
3. Recheck crates.io for hyphen/underscore conflicts and available package name.
4. Inspect `cargo package --list` and package contents, dependencies, license and
   sensitive-file exclusions. Keep private status/evidence outside the package.
5. Choose version, minimum Rust version, changelog, and visibility with the owner.
6. With explicit publication authorization, remove publish=false, run the package
   checks and `cargo publish --dry-run`, then publish and verify registry/docs.rs.
7. Tag the exact verified revision and record receipts and final compatibility scope.

The private implementation and CI are development evidence, not release acceptance. There are no
release workflows, registry credentials, or automatic publishing in this project.

