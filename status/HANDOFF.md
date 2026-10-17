# Development handoff

2026-10-17 · M1–M7 and provider-check tooling complete; public alpha verified.

Jason confirmed PUBLIC repository visibility is intentional for crates.io. Keep
that state and proceed with normal project pushes. No visibility decision is pending.

1. Inspect Git state/remote main and preserve unrelated work. Read CURRENT.md and
   VALIDATION.md for measured boundaries.
2. [CI 39079995335](https://github.com/jasonduncan/semantic-operators-rs/actions/runs/39079995335) passed all eight jobs at
   0c311756b9572fbf44a45c2ad4b3c8b669a11f99. The previous cp1252 fixture failure is
   fixed and verified on both Windows toolchains. Closing changes are docs/status only.
3. Follow [RELEASE](../docs/RELEASE.md) for crates.io preparation. The current version
   is 0.1.0-alpha.1; publish=false remains. Confirm release version/changelog, rerun
   archive checks and publish dry-run, then perform the selected registry release.
   Name checks support apparent availability; recheck before upload. Credential
   configuration has not been inspected and no package upload has occurred.
4. Live qualification is optional and separate. The
   [packet](../qualification/README.md) is concrete: 12 synthetic tickets, one request
   each, no retries/warm-up. It still needs Jason's exact model and call ceiling.
   Use its explicit --execute path only after that scope is supplied; preserve
   outputs/receipts and distinguish completion from semantic label agreement.
5. For tool changes, run `python3 -m unittest discover -s tests -p 'test_triage_*.py'`.
   The 19 checks mock processes/use synthetic output and do not call a model.

No service remains running. Library/consumer Rust source and Python parity fixtures
are unchanged by the bounded-check and visibility work. The registry points here;
update and validate this package after material work. No publication or live request
was performed during verification.
