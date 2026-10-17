# Development handoff

2026-10-17 · M1–M6 complete; M7 standalone consumer integration being verified.

Read CURRENT.md, VALIDATION.md and the
[consumer README](../integrations/triage-app/README.md). The library source is
unchanged from the verified alpha; this phase adds an independent application,
localhost subprocess harness and CI coverage.

1. Inspect Git state and remote main; preserve unrelated changes.
2. Run `python3 tools/check_triage_app.py` from the repository root to build and
   exercise the consumer. It uses synthetic responses, a fake token and a local
   ephemeral server with proxy settings removed. No provider keys are needed.
3. Complete hosted CI for the consumer integration and record the exact receipt.
   CI exercises the independent app on Linux/macOS/Windows, stable and 1.85.0.
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
