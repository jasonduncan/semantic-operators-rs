# Security policy

## Reporting a vulnerability

Report suspected vulnerabilities privately through
[GitHub private vulnerability reporting](https://github.com/jasonduncan/semantic-operators-rs/security/advisories/new).
This channel is enabled for the repository and is received by its maintainers.
Do not open a public issue containing exploit details, API keys, or private inputs.
Include the crate version, enabled features, affected behavior, and a minimal
synthetic reproducer. No paid provider request is needed to file a report.

## Supported versions and response

The project is an experimental alpha. Reports against the latest published alpha
and main are welcome. Fixes will normally target a future release; there is no
promised response deadline, maintenance window, or backport policy.

## Boundaries

The core has no network client by default. Optional adapters send caller-selected
inputs to configured providers; callers remain responsible for data authorization,
endpoint trust, credentials and provider costs. Dropping a local future does not
prove remote cancellation. Confidence is not a guarantee of answer correctness.

Automated dependency/license and history scans supplement review; passing them
is not a complete security assessment. See CONTRIBUTING.md for verification.
