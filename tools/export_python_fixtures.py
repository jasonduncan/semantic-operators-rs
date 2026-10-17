#!/usr/bin/env python3
"""Generate synthetic golden cases from the pinned, trusted local Python source."""

import argparse
import dataclasses
import hashlib
import json
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    baseline = json.loads((ROOT / "docs/source-baseline.json").read_text())["sources"][0]
    source = args.source.resolve()
    revision = subprocess.check_output(
        ["git", "-C", str(source), "rev-parse", "HEAD"], text=True
    ).strip()
    if revision != baseline["revision"]:
        parser.error(f"reference revision differs from pinned {baseline['revision']}")
    dirty = subprocess.check_output(
        ["git", "-C", str(source), "status", "--porcelain", "--untracked-files=no"], text=True
    )
    if dirty:
        parser.error("tracked reference files have local changes")
    for item in baseline["files"]:
        if hashlib.sha256((source / item["path"]).read_bytes()).hexdigest() != item["sha256"]:
            parser.error(f"source hash mismatch: {item['path']}")
    sys.path.insert(0, str(source / "src"))
    from semantic_operators.types import Boolean, Box, Choice, Region, Score, complement, make_answer
    from semantic_operators.interfaces.wire import WireError, decode

    definitions = {
        "bool": {"type": "boolean", "instructions": "Is it relevant?", "min_confidence": 0.0},
        "bool93": {"type": "boolean", "instructions": "Is it relevant?", "min_confidence": 0.93},
        "choice": {"type": "choice", "instructions": "Which group?", "options": [["a", None], ["b", "Other"]]},
        "score": {"type": "score", "instructions": "How urgent?", "levels": ["low", "medium", "high"]},
        "score8": {"type": "score", "instructions": "How urgent?", "levels": ["low", "medium", "high"], "min_confidence": 0.8},
        "box": {"type": "box", "instructions": "Locate the mug"},
        "box1": {"type": "box", "instructions": "Locate the mug", "min_confidence": 0.1},
    }
    questions = {}
    for key, spec in definitions.items():
        kwargs = {k: v for k, v in spec.items() if k != "type"}
        if "options" in kwargs:
            kwargs["options"] = dict(kwargs["options"])
        questions[key] = {"boolean": Boolean, "choice": Choice, "score": Score, "box": Box}[spec["type"]](**kwargs)
    cases = [
        ("boolean_true", "bool", True, {"true": 0.9, "false": 0.1}, None),
        ("boolean_false_is_decided", "bool", False, {"true": 0.1, "false": 0.9}, None),
        ("boolean_tie", "bool", False, {"true": 0.5, "false": 0.5}, None),
        ("threshold_equal", "bool93", False, {"true": 0.07, "false": complement(0.07)}, None),
        ("threshold_below", "bool93", False, {"true": 0.08, "false": 0.92}, None),
        ("boolean_mismatch", "bool", True, {"true": 0.1, "false": 0.9}, None),
        ("boolean_number_is_invalid", "bool", 1, {"true": 0.9, "false": 0.1}, None),
        ("missing_outcome", "bool", True, {"true": 0.9}, None),
        ("bad_total", "bool", True, {"true": 0.9, "false": 0.2}, None),
        ("rounded_total_allowed", "bool", True, {"true": 0.6, "false": 0.395}, None),
        ("choice_decided", "choice", "a", {"a": 0.7, "b": 0.3}, None),
        ("choice_tie", "choice", "a", {"a": 0.5, "b": 0.5}, None),
        ("choice_not_max", "choice", "b", {"a": 0.7, "b": 0.3}, None),
        ("score_expected", "score", 1.3, {"low": 0.2, "medium": 0.3, "high": 0.5}, None),
        ("score_half_rounds_up", "score", 1.5, {"low": 0.0, "medium": 0.5, "high": 0.5}, None),
        ("score_low_nearest_confidence", "score8", 1.3, {"low": 0.2, "medium": 0.3, "high": 0.5}, None),
        ("score_mismatch", "score", 0.0, {"low": 0.2, "medium": 0.3, "high": 0.5}, None),
        ("score_zero", "score", 0.0, {"low": 1.0, "medium": 0.0, "high": 0.0}, None),
        ("box_no_confidence", "box", Region(0.1, 0.8, 0.2, 0.7), {}, None),
        ("box_missing_confidence_abstains", "box1", Region(0.1, 0.8, 0.2, 0.7), {}, None),
        ("box_explicit_confidence", "box1", Region(0.1, 0.8, 0.2, 0.7), {}, 0.7),
        ("box_zero_area", "box", Region(0.2, 0.2, 0.4, 0.4), {}, None),
        ("box_distribution_invalid", "box", Region(0.1, 0.8, 0.2, 0.7), {"yes": 1.0}, None),
    ]
    def value_json(value):
        return dataclasses.asdict(value) if dataclasses.is_dataclass(value) else value

    answers = []
    for name, question, value, probabilities, confidence in cases:
        record = {"id": name, "question": definitions[question], "value": value_json(value),
                  "probabilities": probabilities, "confidence": confidence}
        try:
            answer = make_answer(questions[question], value, probabilities, confidence=confidence)
            record["expected"] = {"value": value_json(answer.value), "decided": answer.decided,
                                  "confidence": answer.confidence, "probabilities": answer.probabilities}
        except ValueError as error:
            record["expected_error"] = {"kind": "invalid_response", "reference_message": str(error)}
        answers.append(record)
    request = {"state": "Synthetic support message", "questions": [
        {"name": "ok", "type": "boolean", "instructions": "Is it relevant?"}]}
    wire_inputs = [
        ("valid", request), ("request_id", {**request, "request_id": "case-1"}),
        ("null_id", {**request, "request_id": None}),
        ("bad_version", {**request, "schema_version": "2"}),
        ("unknown_field", {**request, "typo": True}),
        ("invalid_state", {**request, "state": 42}),
        ("empty_questions", {**request, "questions": []}),
        ("duplicate_question", {**request, "questions": request["questions"] * 2}),
        ("box_not_wire_v1", {**request, "questions": [
            {"name": "box", "type": "box", "instructions": "Locate a mug"}]}),
        ("not_object", []),
    ]
    wire_cases = []
    for name, data in wire_inputs:
        record = {"id": name, "request": data}
        try:
            state, decoded, request_id = decode(data)
            record["expected"] = {"state": state, "question_names": list(decoded), "request_id": request_id}
        except WireError as error:
            record["expected_error"] = {"path": error.path, "message": str(error)}
        wire_cases.append(record)
    document = {"fixture_version": 1, "reference_revision": revision,
                "generator_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                "answer_cases": answers,
                "complement_cases": [{"input": p, "expected": complement(p)} for p in [0.0, 0.07, 0.5, 0.93, 1.0]],
                "wire_decode_cases": wire_cases}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(document, indent=2, allow_nan=False) + "\n")
    print(f"Wrote {len(answers)} answer, 5 complement, {len(wire_cases)} wire cases to {args.output}")


if __name__ == "__main__":
    main()

