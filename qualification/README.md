# Bounded triage provider check

This packet evaluates the standalone Rust triage consumer against 12 synthetic
support requests. It is a transport/application smoke check with descriptive
label comparisons. The 10 clear-case labels were authored with the synthetic text
by the assistant; they have not been independently reviewed. The 2 desired-review
cases are exploratory ambiguity probes. This is not a production acceptance set.

## Frozen scope

- Dataset: [triage-v1.json](triage-v1.json), schema semops.triage-check/1.
- Ten clear cases: technical/billing route and urgent/routine priority.
- Two exploratory cases: measure whether the consumer routes them to human review.
- Existing application policy: both questions require confidence 0.8; urgency
  expected score at least 1.5 maps to urgent. Do not tune these on this set.
- One HTTP request per ticket, two questions per request, 12 requests maximum for
  one complete run. No retries, warm-up, replacement cases or repeated sampling.
- Credentials: inherited TYPESAFE_API_KEY, never printed or persisted.
- Provider: official TypeSafe endpoint; an exact model ID and call limit must be
  selected before execution. No implicit model choice or fallback.
- Deadline: 30 seconds per request, plus an outer process limit bounded by the
  selected count. On failure, stop and retain completed output; do not replay.

The repo AGENTS.md requires a separately scoped live request with dataset, model,
call/cost limit and retained evidence. This packet makes the data and evaluation
concrete. Preparing it does not itself authorize paid/hosted execution. The call
ceiling bounds requests, not money or generated tokens; no pricing/cost claim is made.

## Artifacts and interpretation

A run should retain the dataset hash, consumer source/binary identity, requested
model, maximum calls, start/finish timestamps, exit status, output and diagnostics
under ignored var/. No keys belong in those files. Observed model identities and
known reported usage come from the actual responses. Usage is counted once per
completed ticket, even though its two answers share a Call record. Missing usage
and calls with no output remain unknown; totals do not establish full billing.

The offline scorer reports:

- execution completion and missing ticket IDs;
- routed coverage over all planned clear cases and over observed clear cases;
- joint route/priority accuracy among routed clear cases (null if none routed);
- correct joint decisions divided by all planned clear cases;
- clear-case abstentions and incorrect routes/priorities separately;
- desired-review probe outcomes separately from labeled accuracy;
- reported model identities and known/unknown token usage.

Malformed/duplicate/unknown outputs fail validation. A partial valid JSONL prefix
can be scored and is explicitly incomplete. A truncated or malformed JSONL row is
not silently ignored; preserve the original artifact for inspection. Do not equate
abstention, no output, transport failure or wrong decisions. No automatic pass grade
or confidence calibration claim is assigned from this small author-labeled set.

## Local verification

The scorer and runner have offline checks and do not make hosted requests during
normal tests. Live outcomes, if authorized, must be recorded separately from these
local correctness checks. The implementation's previous Python parity fixtures
remain independent of this new semantic dataset.

```sh
python3 -m unittest discover -s tests -p 'test_triage_*.py'
python3 tools/run_triage_check.py
```

The second command prints a plan without building, reading credentials or writing
run artifacts. A scoped model/cap can be checked with `--model MODEL --max-calls 12`;
it still makes no requests. After those selections are explicitly authorized:

```sh
python3 tools/run_triage_check.py --model MODEL --max-calls 12 --execute
```

Use the exact authorized ID in place of MODEL. The runner builds the consumer
with locked, cached dependencies (`cargo build --locked --offline`), checks that
source inputs did not change during the build, and records source/tool/binary
hashes. A missing dependency stops locally; it never triggers an inference retry.
It freezes the dataset and checks its hash again before invoking the consumer.
The generated JSONL includes only ticket IDs and text, never expected labels.

Each invocation creates a unique ignored `var/triage-check-*` directory containing
dataset.json, input.jsonl, output.jsonl, stderr.txt, receipt.json and, when the output
is valid, report.json. POSIX permissions default to owner only. Receipts record
the exact command without credentials, hashes, timestamps and process outcome.
If abruptly terminated, a null process_started/exit_code means unknown; do not
infer no requests or automatically rerun. Timeout/interruption kills and reaps the
consumer before writing its final receipt. Already submitted server work and its
usage can remain unknown even after local cancellation.

Recompute the descriptive report offline from retained files:

```sh
python3 tools/score_triage_check.py --dataset var/RUN/dataset.json --results var/RUN/output.jsonl
```

The runner exits 0 only for zero consumer exit and every expected row retained;
this is execution completion, not label agreement or a quality acceptance grade.
It exits 1 for incomplete execution, or 2 for configuration/dataset/build errors.
The scorer exits 2 for malformed evidence. Preserve artifacts before investigating.
