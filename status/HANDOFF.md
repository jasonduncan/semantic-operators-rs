# Development handoff

2026-10-17 · M1–M6 complete; M7 standalone consumer integration complete.

Read CURRENT.md, VALIDATION.md and the
[consumer README](../integrations/triage-app/README.md). The library source is
unchanged from the verified alpha; this phase adds an independent application,
localhost subprocess harness and CI coverage.

1. Inspect Git state and remote main; preserve unrelated changes.
2. Run `python3 tools/check_triage_app.py` from the repository root to build and
   exercise the consumer. It uses synthetic responses, a fake token and a local
   ephemeral server with proxy settings removed. No provider keys are needed.
3. Verified implementation: feb00f2ae3d58f9b75b4c98ef8e2a21aa1a5995c.
   [CI 39074395130](https://github.com/jasonduncan/semantic-operators-rs/actions/runs/39074395130)
   passed all eight jobs, including the consumer on Linux/macOS/Windows and both
   toolchains. The final closing commit changes status/docs only. Repeat relevant
   checks after implementation changes.
4. Next select an existing application integration or separately bounded live
   qualification with a dataset, exact model and call/cost limit. No existing
   product was selected in this turn; this consumer provides a concrete template.
5. Publication follows RELEASE.md under explicit publication direction. The crate
   name needs rechecking and the alpha API remains unstable.

The app takes JSON Lines, validates the full batch before I/O, performs one batched
request per ticket, and emits confidence/probabilities/call metadata. Uncertainty
routes to review. Execution errors stop further calls, preserve earlier output and
exit 1. Input/configuration errors exit 2 before dispatch. No retries/warmup.

No hosted model requests, private-document transfer, paid inference, deployment,
crate publication or credentials were introduced. No service remains running after
verification. Source siblings and frozen reference fixtures are unchanged. Update
status, validate, commit/push and refresh the registry after material work.
