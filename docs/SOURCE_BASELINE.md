# Source baseline and provenance

Observed 2026-10-17. [Machine-readable inventory](source-baseline.json) records
local source paths, sanitized remotes, exact revisions, tracked cleanliness, and
SHA-256 hashes. Originals remain in their existing repositories; no source
repository was changed or bulk copied into this project.

| Source | Frozen revision | Role |
|---|---|---|
| Python semantic-operators-lib, package 0.12.0 | `07fb1f3109dac2e69a7dafe9fa349d0daa52012d` | Primary semantics, image behavior, wire format |
| C++ libsemop | `dc3b28f2d22ac6ca27ddcf0b7529a7082de56d82` | Typed API and provider lifecycle reference |

Both checkouts were clean at inspection. Python's package metadata says 0.12.0
while CHANGELOG labels it unreleased; this specification uses the local commit
and makes no package publication claim. C++ historical design validation is
reported evidence from its author, not tests rerun during Rust setup.

Inspected material: both README/design/instruction surfaces; Python manifest,
exports, types/normalization, provider, operators, cascade, flows, filtering,
reranking, wire, TypeSafe translation, locate, image constants, benchmark entry
points and test inventory. Hash inventory is broader than detailed inspection.
Reference fixture execution covers only the synthetic cases in the exporter;
the full sibling suites were not run.

The earlier observation-oriented Python project is excluded. Prior memory helped
locate the sibling relationship, but version/API decisions were verified against
current local files.

## External technical references

- [Cargo manifest reference](https://doc.rust-lang.org/cargo/reference/manifest.html),
  accessed 2026-10-17: package/library naming, edition, rust-version, feature and
  publication metadata. Used for the bootstrap manifest.
- [Rust trait reference](https://doc.rust-lang.org/reference/items/traits.html),
  accessed 2026-10-17: dyn compatibility. Used to choose an object-safe boxed-future
  provider contract rather than an async method directly on a dyn trait.
- [crates.io package API](https://crates.io/api/v1/crates/semantic-operators),
  accessed 2026-10-17: returned HTTP 403 in the command-line check. Name availability
  is unresolved; no package was published or reserved.

External references were consulted online, not archived as full documents. No
private source text was sent in web queries. Future provider endpoint/SDK/image
decisions require their own current official documentation/source check.


## Package-index cross-check

GitHub contents API for the official
[rust-lang/crates.io-index](https://github.com/rust-lang/crates.io-index) returned
404 for both semantic-operators and semantic_operators index paths; a serde
control lookup succeeded. This supports apparently unregistered, not guaranteed
availability. Direct crates.io access remained blocked as recorded above.
