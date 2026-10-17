#!/usr/bin/env python3
"""Score a bounded synthetic triage check from retained JSON/JSONL files, offline.

This checks the consumer's output contract and reports descriptive metrics. It
is not a substitute for Rust normalization or independently reviewed evaluation.
No network, credential access, model invocation, cost estimate, or pass grade.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path
import sys
from typing import Any

DATASET_SCHEMA = "semops.triage-check/1"
REPORT_SCHEMA = "semops.triage-score/1"
AUTHORITY = "assistant-authored-synthetic"
DEPARTMENTS = {"technical", "billing"}
LEVELS = ("routine", "soon", "urgent")
THRESHOLD = 0.8


class ContractError(ValueError):
    """Malformed dataset or consumer evidence; diagnostic text omits ticket data."""


def _require(condition: bool, message: str) -> None:
    if not condition:
        raise ContractError(message)


def _keys(value: Any, expected: set[str], label: str) -> dict:
    _require(isinstance(value, dict) and set(value) == expected, f"invalid {label} fields")
    return value


def _text(value: Any, label: str, max_bytes: int | None = None) -> str:
    _require(isinstance(value, str) and bool(value.strip()), f"{label} must be a nonblank string")
    try:
        encoded = value.encode("utf-8")
    except UnicodeEncodeError as error:
        raise ContractError(f"{label} must be valid UTF-8") from error
    if max_bytes is not None:
        _require(len(encoded) <= max_bytes, f"{label} exceeds consumer byte limit")
    return value


def _number(value: Any, label: str) -> float:
    _require(type(value) in (int, float), f"{label} must be a finite number")
    try:
        result = float(value)
    except (OverflowError, ValueError) as error:
        raise ContractError(f"{label} must be a finite number") from error
    _require(math.isfinite(result), f"{label} must be a finite number")
    return result


def _pairs(pairs: list[tuple[str, Any]]) -> dict:
    result = {}
    for key, value in pairs:
        _require(key not in result, "duplicate JSON object key")
        result[key] = value
    return result


def _float(text: str) -> float:
    value = float(text)
    _require(math.isfinite(value), "nonfinite JSON number")
    return value


def _constant(_: str) -> None:
    raise ContractError("nonfinite JSON number")


def _json(data: bytes | str) -> Any:
    try:
        return json.loads(data, object_pairs_hook=_pairs, parse_float=_float, parse_constant=_constant)
    except (ValueError, UnicodeDecodeError, RecursionError) as error:
        raise ContractError("invalid or truncated JSON input") from error


def validate_dataset(dataset: dict) -> dict:
    """Validate the exact version-one schema without modifying the dataset."""
    _keys(dataset, {"schema", "label_authority", "cases"}, "dataset")
    _require(dataset["schema"] == DATASET_SCHEMA, "unsupported dataset schema")
    _require(dataset["label_authority"] == AUTHORITY, "unsupported label authority")
    cases = dataset["cases"]
    _require(isinstance(cases, list) and 1 <= len(cases) <= 100, "dataset needs 1 to 100 cases")
    seen = set()
    for case in cases:
        _require(isinstance(case, dict), "dataset case must be an object")
        _require(("expected" in case) != ("expect_review" in case), "case needs either expected labels or a review probe")
        _keys(case, {"id", "text", "expected"} if "expected" in case else {"id", "text", "expect_review"}, "case")
        identity = _text(case["id"], "case id", 1024)
        _text(case["text"], "case text", 16 * 1024)
        _require(identity not in seen, "duplicate dataset id")
        seen.add(identity)
        if "expected" in case:
            labels = _keys(case["expected"], {"route", "priority"}, "expected labels")
            _require(type(labels["route"]) is str and labels["route"] in DEPARTMENTS, "invalid expected route")
            _require(type(labels["priority"]) is str and labels["priority"] in {"urgent", "routine"}, "invalid expected priority")
        else:
            _require(case["expect_review"] is True, "expect_review must be true")
    return dataset


def load_dataset(path: str | Path) -> dict:
    """Load strict JSON and validate its dataset schema."""
    return parse_dataset(Path(path).read_bytes())


def parse_dataset(data: bytes) -> dict:
    """Validate one immutable snapshot of dataset bytes."""
    return validate_dataset(_json(data))


def _results(data: bytes) -> list[dict]:
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise ContractError("results must be UTF-8") from error
    rows = []
    lines = text.split("\n")
    if lines[-1] == "":
        lines.pop()
    for line in lines:
        _require(bool(line.strip()), "blank results line is not a JSON record")
        value = _json(line)
        _require(isinstance(value, dict), "result line must be an object")
        rows.append(value)
        _require(len(rows) <= 100, "results exceed the finite dataset limit")
    return rows


def load_results(path: str | Path) -> list[dict]:
    """Load strict JSONL; an empty file is a valid incomplete observation."""
    return _results(Path(path).read_bytes())


def _call(value: Any) -> dict | None:
    if value is None:
        return None
    _keys(value, {"provider", "model", "input_tokens", "output_tokens"}, "call")
    _text(value["provider"], "call provider")
    _require(value["model"] is None or isinstance(value["model"], str), "reported model must be a string or null")
    if value["model"] is not None:
        try:
            value["model"].encode("utf-8")
        except UnicodeEncodeError as error:
            raise ContractError("reported model must be valid UTF-8") from error
    for name in ("input_tokens", "output_tokens"):
        counter = value[name]
        _require(counter is None or type(counter) is int and 0 <= counter <= 2**64 - 1,
                 "token counters must be unsigned integers or null")
    return value


def _answer(value: Any, kind: str) -> dict:
    _keys(value, {"value", "confidence", "probabilities", "call"}, "answer")
    confidence = _number(value["confidence"], "confidence")
    _require(0 <= confidence <= 1, "confidence must be from zero to one")
    labels = DEPARTMENTS if kind == "department" else set(LEVELS)
    probabilities = _keys(value["probabilities"], labels, "probability")
    for probability in probabilities.values():
        _require(0 <= _number(probability, "probability") <= 1, "probabilities must be from zero to one")
    _require(abs(math.fsum(probabilities.values()) - 1.0) <= 0.005 * len(labels) + 1e-9,
             "probabilities do not satisfy the core sum tolerance")
    observed = value["value"]
    if kind == "department":
        top = max(probabilities.values())
        _require(confidence == top, "department confidence must equal maximum probability")
        tied = sum(p == top for p in probabilities.values()) > 1
        if observed is not None:
            _require(type(observed) is str and observed in labels, "department value must be a known label or null")
            _require(not tied and probabilities[observed] == top and confidence >= THRESHOLD,
                     "decided department conflicts with probabilities or threshold")
    else:
        _require(confidence in probabilities.values(), "urgency confidence must be one rubric probability")
    if kind == "urgency" and observed is not None:
        numeric = _number(observed, "urgency value")
        _require(0 <= numeric <= 2, "urgency value must be between zero and two")
        expected = sum(i * probabilities[label] for i, label in enumerate(LEVELS))
        _require(abs(numeric - expected) <= 0.005 * (1 + sum(range(len(LEVELS)))),
                 "urgency value conflicts with expected rubric score")
        nearest = min(math.floor(numeric + 0.5), len(LEVELS) - 1)
        _require(confidence == probabilities[LEVELS[nearest]] and confidence >= THRESHOLD,
                 "decided urgency conflicts with confidence or threshold")
    # An undecided answer deliberately lacks its original candidate. Do not
    # fabricate that candidate or claim full normalization from this projection.
    _call(value["call"])
    return value


def _row(row: Any) -> dict:
    _keys(row, {"id", "status", "route", "priority", "answers"}, "result")
    _text(row["id"], "result id", 1024)
    _require(row["status"] in ("routed", "review"), "unknown consumer status")
    answers = _keys(row["answers"], {"department", "urgency"}, "answers")
    department = _answer(answers["department"], "department")
    urgency = _answer(answers["urgency"], "urgency")
    _require(department["call"] == urgency["call"], "both answers must carry the same shared call metadata")
    if row["status"] == "routed":
        _require(department["value"] is not None and urgency["value"] is not None,
                 "routed rows require two decided answers")
        _require(row["route"] == department["value"], "route differs from department answer")
        priority = "urgent" if urgency["value"] >= 1.5 else "routine"
        _require(row["priority"] == priority, "priority differs from urgency policy")
    else:
        _require(row["route"] is None and row["priority"] is None,
                 "review rows must have null route and priority")
        _require(department["value"] is None or urgency["value"] is None,
                 "review rows require at least one undecided answer")
    return row


def _ratio(numerator: int, denominator: int) -> float | None:
    return numerator / denominator if denominator else None


def score(dataset: dict, rows: list[dict]) -> dict:
    """Validate consumer evidence and produce descriptive counts without a pass grade.

    Exact file-byte hashes are attached by `score_files`; this pure entry point
    does not invent file hashes for in-memory objects.
    """
    validate_dataset(dataset)
    _require(isinstance(rows, list) and len(rows) <= 100, "results must be a list of at most 100 rows")
    cases = {case["id"]: case for case in dataset["cases"]}
    observed = {}
    models = set()
    providers = set()
    usage = {"known_input_tokens": 0, "known_output_tokens": 0,
             "rows_with_unknown_input_tokens": 0, "rows_with_unknown_output_tokens": 0,
             "rows_without_call_metadata": 0, "rows_with_call_metadata": 0}
    for row in rows:
        _row(row)
        identity = row["id"]
        _require(identity in cases, "result contains an unknown case id")
        _require(identity not in observed, "duplicate result id")
        observed[identity] = row
        call = row["answers"]["department"]["call"]
        models.add(None if call is None else call["model"])
        providers.add(None if call is None else call["provider"])
        usage["rows_without_call_metadata" if call is None else "rows_with_call_metadata"] += 1
        for name in ("input_tokens", "output_tokens"):
            counter = None if call is None else call[name]
            if counter is None:
                usage[f"rows_with_unknown_{name}"] += 1
            else:
                usage[f"known_{name}"] += counter
    counts = {"total": len(cases), "clear": 0, "review_probes": 0, "completed": len(rows),
              "missing": len(cases) - len(rows), "clear_completed": 0, "clear_routed": 0,
              "clear_correct": 0, "clear_wrong": 0, "clear_abstained": 0}
    probes = {"total": 0, "completed": 0, "missing": 0, "reviewed": 0, "routed": 0}
    assessments = []
    for identity, case in cases.items():
        row = observed.get(identity)
        if "expected" in case:
            counts["clear"] += 1
            if row is None:
                assessment = "missing"
            else:
                counts["clear_completed"] += 1
                if row["status"] == "review":
                    counts["clear_abstained"] += 1
                    assessment = "clear_abstained"
                else:
                    counts["clear_routed"] += 1
                    correct = all(row[key] == value for key, value in case["expected"].items())
                    assessment = "clear_correct" if correct else "clear_wrong"
                    counts[assessment] += 1
        else:
            counts["review_probes"] += 1
            probes["total"] += 1
            if row is None:
                probes["missing"] += 1
                assessment = "missing"
            else:
                probes["completed"] += 1
                outcome = "reviewed" if row["status"] == "review" else "routed"
                probes[outcome] += 1
                assessment = f"review_probe_{outcome}"
        assessments.append({"id": identity, "assessment": assessment})
    probes["observed_review_rate"] = _ratio(probes["reviewed"], probes["completed"])
    probes["interpretation"] = "Exploratory desired-review observations; not independently reviewed abstention labels."
    return {"schema": REPORT_SCHEMA, "dataset_schema": DATASET_SCHEMA, "label_authority": AUTHORITY,
            "counts": counts,
            "metrics": {"coverage": _ratio(counts["clear_routed"], counts["clear"]),
                        "observed_coverage": _ratio(counts["clear_routed"], counts["clear_completed"]),
                        "conditional_accuracy": _ratio(counts["clear_correct"], counts["clear_routed"]),
                        "overall_accuracy": _ratio(counts["clear_correct"], counts["clear"])},
            "exploratory_review": probes, "assessments": assessments,
            "reported_models": sorted(models, key=lambda model: (model is not None, model or "")),
            "reported_providers": sorted(providers, key=lambda provider: (provider is not None, provider or "")),
            "usage_observations": {**usage, "completed_row_call_observations_lower_bound": len(rows),
                                   "interpretation": "Shared call metadata counted once per completed row; known tokens are observed lower bounds. Failed, cancelled, or missing-row calls and usage are not inferred."},
            "limitations": ["Labels are assistant-authored synthetic and have not been independently reviewed.",
                            "Input checks validate the consumer projection and routing policy, not the full Rust normalizer; undecided candidates are unavailable.",
                            "Missing rows do not distinguish unattempted, failed, cancelled, or unretained calls.",
                            "No pass grade, production-readiness conclusion, actual-total-call claim, or cost estimate is produced."]}


def score_files(dataset_path: str | Path, results_path: str | Path) -> dict:
    """Score strict files and bind the report to their exact SHA-256 byte hashes."""
    dataset_bytes = Path(dataset_path).read_bytes()
    results_bytes = Path(results_path).read_bytes()
    report = score(_json(dataset_bytes), _results(results_bytes))
    report["source_hashes"] = {"dataset_sha256": hashlib.sha256(dataset_bytes).hexdigest(),
                               "results_sha256": hashlib.sha256(results_bytes).hexdigest()}
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dataset", type=Path, required=True)
    parser.add_argument("--results", type=Path, required=True)
    args = parser.parse_args()
    try:
        report = score_files(args.dataset, args.results)
    except (ContractError, OSError) as error:
        message = str(error) if isinstance(error, ContractError) else "cannot read input files"
        print(f"triage scoring rejected input: {message}", file=sys.stderr)
        return 2
    print(json.dumps(report, indent=2, allow_nan=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
