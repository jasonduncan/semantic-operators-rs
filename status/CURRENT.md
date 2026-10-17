# Current state

2026-10-17 · portfolio active · execution blocked on live model/budget selection.

The native Rust `semantic-operators` 0.1.0-alpha.1 library (M1–M6) and standalone
JSONL triage consumer (M7) are implemented and verified. Cargo publication remains disabled and the alpha API unstable.
GitHub visibility unexpectedly reads PUBLIC; see the visibility note below. Core has no default networking
or runtime dependencies. Prior platform acceptance remains in [VALIDATION](VALIDATION.md).

The [bounded provider-check packet](../qualification/README.md) is now prepared:
12 synthetic tickets, 10 assistant-authored clear labels and 2 exploratory review
probes; default-off runner; offline strict scorer; private ignored run receipts.
It records coverage, joint label agreement, abstentions, missing results and known/
unknown usage without a pass grade. Nineteen offline tests pass; no live check ran. Hosted preparation CI passed six
jobs; Windows exposed a test-fixture encoding bug, fixed locally with explicit UTF-8.
The corrected test awaits hosted verification after the visibility decision.

The next action is one 12-request run, two questions per request, no retries or
warm-up, 30 seconds per request. Jason still needs to select the exact TypeSafe
model and call ceiling. This blocks live execution; the dataset and
retention/evaluation protocol are concrete and ready for review. Do not infer
approval from a generic continuation or from a credential being present.

No hosted inference, private corpus upload, paid evaluation, deployment, crate
reservation or publication has occurred. Generic CLI/MCP, local inference and other
explicitly deferred surfaces remain deferred. Library and consumer Rust code are
unchanged by this preparation phase.

## Repository visibility readback

After the preparation push, explicit GitHub repo/API checks returned PUBLIC for
jasonduncan/semantic-operators-rs. Earlier status recorded PRIVATE. This work did
not change visibility; who changed it and when are unknown. c78bd05 was already
pushed before the discrepancy was detected. It contains code/docs/synthetic data,
no credentials or live outputs. Asked Jason whether public visibility is intentional
or private should be restored. Further pushes are held pending that answer; local
status corrections are retained. Do not repeat the old private claim as current.
