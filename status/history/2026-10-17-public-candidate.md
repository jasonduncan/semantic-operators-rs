# Public alpha candidate verified

## Public candidate verification complete — 2026-10-17

[CI 39079995335](https://github.com/jasonduncan/semantic-operators-rs/actions/runs/39079995335) passed all eight jobs at
0c311756b9572fbf44a45c2ad4b3c8b669a11f99, including the corrected UTF-8 fixture on
Windows stable/1.85.0. Nineteen tool tests, existing Rust/consumer checks, individual
features and package consumers all pass. Local package audit independently passed
31-file archive/default dependency isolation/both extracted consumers. Crates.io API
and hyphen/underscore sparse entries returned 404; serde control returned 200.

Public distribution is intentional; no visibility blocker remains. Portfolio active,
execution ready for release preparation. Optional live-model scope is still unset,
not a package-engineering blocker. publish=false remains; no upload, live inference
or credential inspection occurred. Closing updates only change documentation/status;
source/tests/workflow match the passing run above.
