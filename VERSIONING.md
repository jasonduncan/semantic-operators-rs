# Versioning and compatibility

`semantic-operators` is currently `0.1.0-alpha.2`. Its API is experimental;
publication and passing tests do not imply qualified model accuracy or suitability
for a particular application.

## Alpha releases

Alpha releases may change public APIs or behavior. Pin an exact prerelease while
evaluating the crate (`semantic-operators = "=0.1.0-alpha.2"`) and review the
[changelog](CHANGELOG.md) before updating. Breaking changes should include migration
notes. There is no fixed deprecation period during alpha development.

Repository main may contain unreleased changes. Registry versions are immutable;
a new upload needs a new version. Tags use `v<version>`, for example
`v0.1.0-alpha.2`. Release checks and publication are documented in the
[release procedure](docs/RELEASE.md).

## Compatibility policy after alpha

For non-prerelease `0.x` versions, a breaking API or supported-behavior change
requires a minor-version increase, such as `0.1` to `0.2`. Compatible fixes and
additions can use patch releases. Once `1.0` is reached, breaking changes require
a major-version increase. This follows
[Cargo's compatibility conventions](https://doc.rust-lang.org/cargo/reference/semver.html).

Changes to question validation, probability normalization, abstention behavior,
wire contracts and feature availability are reviewed as compatibility changes,
not only changes that cause compilation failures. Corrections to unintended
behavior must explain their effect in the changelog. Deprecations should document
a replacement where one exists; no calendar-based support window is promised.

## Minimum supported Rust version

The current MSRV is Rust 1.85, declared in Cargo.toml and checked alongside stable
Rust in CI on Linux, macOS and Windows. Raising the declared MSRV requires release
notes and a breaking-version change under the policy above; during alpha it must
be explicit in the next prerelease's notes. Dependency updates must preserve the
current MSRV unless the release intentionally raises it.

## Provider and model behavior

Provider services and models can change independently of crate releases. The
[compatibility contract](docs/COMPATIBILITY.md) states the frozen reference and
Rust adaptations. Offline parity and transport checks do not guarantee that a
hosted model produces correct answers for an application's inputs. Evaluate and
monitor the task separately from upgrading the library.
