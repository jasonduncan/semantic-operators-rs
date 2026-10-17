# Standalone support-triage consumer

A small Rust application using semantic-operators as a path dependency through its
public API. It reads JSON Lines from stdin and writes JSON Lines to stdout. It has
its own Cargo manifest and lockfile; no library test helpers or private modules.

For each ticket, one TypeSafe request asks the department (technical/billing) and
urgency (routine/soon/urgent). Both questions require confidence at least 0.8.
A decided expected urgency score at least 1.5 becomes urgent; other decided scores
become routine. If either answer is undecided, the ticket goes to review with no
route or priority. These application thresholds are illustrative policy choices,
not calibrated model claims. Changing them does not require changing the library.

## Run locally without a hosted provider

From the repository root:

```sh
python3 tools/check_triage_app.py
```

This builds the independent consumer and exercises it against a temporary Python
HTTP server on a loopback IP. Synthetic answers test the adapter and application
pipeline. No credentials or hosted model are used; the test server is shut down
when the checks finish. It is not a content-classifying model.

Build/help separately:

```sh
cargo build --locked --manifest-path integrations/triage-app/Cargo.toml
cargo run --locked --manifest-path integrations/triage-app/Cargo.toml -- --help
```

## Input and output

Input fields are exactly id and text; both must be nonblank strings. IDs are unique
within the batch and remain local metadata. The provider receives only the text.
Blank lines are ignored. The full batch is validated before any output or HTTP call.
Limits: 1 MiB total input, 100 records, 16 KiB text per record, 1 KiB ID per record.
The explicit --max-calls limit must be from 1 to 100 and covers the whole batch.

```json
{"id":"ticket-1","text":"The invoice lists the same charge twice."}
```

Each output row includes id, status (routed/review), route (technical/billing/null),
priority (urgent/routine/null), and answers for department and urgency. Each answer
retains value (including null), confidence, probabilities, and reported call metadata
(provider/model/input_tokens/output_tokens). Missing metadata stays null. No raw
response, source message or API key is copied into output. Output IDs and metadata
can still be sensitive; choose output destinations accordingly.

Outputs flush one row at a time. Provider/timeout/protocol failures stop the batch
with exit 1; earlier rows remain available. There is no retry or warmup. An interrupted
or failed call may have reached the service even if it produced no row. Do not
blindly replay the entire input file when resuming a partial batch.

Exit 0 means all records completed, including review outcomes (and empty input).
Exit 2 means malformed input/configuration, with no requests. Exit 1 means execution
or output I/O failure. Diagnostics omit arbitrary input and provider body text.

## Connect a provider

Select an exact model and bounded dataset for the task. Set TYPESAFE_API_KEY through
your shell/password-manager mechanism; the application never loads .env files.
Then explicitly opt into the hosted endpoint:

```sh
cargo run --locked --manifest-path integrations/triage-app/Cargo.toml -- \
  --live --model YOUR_MODEL_ID --max-calls 3 < tickets.jsonl
```

The default endpoint is the library's official TypeSafe endpoint. An alternate
remote endpoint must use HTTPS and --live; a literal loopback IP may use HTTP
without --live for fixtures. Credentials are sent to the configured endpoint.
The underlying HTTP client can honor the application's proxy environment.
--timeout-ms bounds each call (default 30000, maximum 300000); the library disables
redirects and retries. No hosted invocation was performed to validate this app.

## Verification boundary

The end-to-end harness checks routing, abstention, threshold equality, provenance,
exact request shape/call count, fatal failures, full-batch input validation, privacy,
and caller-selected budgets through a real subprocess and local HTTP connection.
It does not establish semantic accuracy, production routing suitability, performance,
or live-model compatibility. This consumer is excluded from the library crate
archive; copy/adapt it for an application without enlarging the library's API.
