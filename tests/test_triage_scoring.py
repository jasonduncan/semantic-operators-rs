"""Offline synthetic tests for consumer evidence scoring; no model or HTTP calls."""
from copy import deepcopy
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "tools" / "score_triage_check.py"
spec = importlib.util.spec_from_file_location("triage_scoring", SCRIPT)
scoring = importlib.util.module_from_spec(spec)
spec.loader.exec_module(scoring)


def dataset(*cases):
    return {"schema": "semops.triage-check/1", "label_authority": "assistant-authored-synthetic", "cases": list(cases)}


def clear(identity, route="technical", priority="urgent"):
    return {"id": identity, "text": "Synthetic fixture text.", "expected": {"route": route, "priority": priority}}


def probe(identity):
    return {"id": identity, "text": "Synthetic ambiguous fixture.", "expect_review": True}


def call(model="reported-model", input_tokens=17, output_tokens=5):
    return {"provider": "TypeSafe", "model": model, "input_tokens": input_tokens, "output_tokens": output_tokens}


def row(identity, route="technical", priority="urgent", review=False, metadata="default"):
    metadata = call() if metadata == "default" else metadata
    department = {"value": route, "confidence": 0.9,
                  "probabilities": {"technical": 0.9 if route == "technical" else 0.1,
                                    "billing": 0.9 if route == "billing" else 0.1}, "call": deepcopy(metadata)}
    urgency = {"value": 1.9 if priority == "urgent" else 0.1, "confidence": 0.9,
               "probabilities": {"routine": 0.0 if priority == "urgent" else 0.9,
                                 "soon": 0.1, "urgent": 0.9 if priority == "urgent" else 0.0}, "call": deepcopy(metadata)}
    if review:
        department.update(value=None, confidence=0.5, probabilities={"technical": 0.5, "billing": 0.5})
    return {"id": identity, "status": "review" if review else "routed", "route": None if review else route,
            "priority": None if review else priority, "answers": {"department": department, "urgency": urgency}}


class TriageScoringTests(unittest.TestCase):
    def test_perfect_abstention_wrong_partial_and_review_probes_have_separate_denominators(self):
        data = dataset(clear("correct"), clear("wrong"), clear("abstain"), clear("missing"), probe("probe-review"), probe("probe-route"), probe("probe-missing"))
        rows = [row("correct"), row("wrong", "billing", "routine"), row("abstain", review=True), row("probe-review", review=True), row("probe-route")]
        report = scoring.score(data, rows)
        self.assertEqual(report["counts"], {"total": 7, "clear": 4, "review_probes": 3, "completed": 5, "missing": 2,
                                            "clear_completed": 3, "clear_routed": 2, "clear_correct": 1, "clear_wrong": 1, "clear_abstained": 1})
        self.assertEqual(report["metrics"], {"coverage": 0.5, "observed_coverage": 2 / 3, "conditional_accuracy": 0.5, "overall_accuracy": 0.25})
        self.assertEqual(report["exploratory_review"]["observed_review_rate"], 0.5)
        self.assertEqual(report["exploratory_review"]["missing"], 1)
        self.assertEqual(report["assessments"][3], {"id": "missing", "assessment": "missing"})
        self.assertNotIn("pass", report)
        self.assertEqual(report["label_authority"], "assistant-authored-synthetic")

    def test_perfect_results_and_never_answered_denominators(self):
        data = dataset(clear("a"), clear("b", "billing", "routine"))
        report = scoring.score(data, [row("a"), row("b", "billing", "routine")])
        self.assertEqual(set(report["metrics"].values()), {1.0})
        report = scoring.score(data, [row("a", review=True)])
        self.assertEqual(report["metrics"]["coverage"], 0.0)
        self.assertEqual(report["metrics"]["observed_coverage"], 0.0)
        self.assertIsNone(report["metrics"]["conditional_accuracy"])
        report = scoring.score(data, [])
        self.assertEqual(report["counts"]["missing"], 2)
        self.assertIsNone(report["metrics"]["observed_coverage"])
        self.assertIsNone(report["metrics"]["conditional_accuracy"])
        self.assertEqual(report["metrics"]["overall_accuracy"], 0.0)
        only_probes = scoring.score(dataset(probe("probe")), [row("probe", review=True)])
        self.assertTrue(all(value is None for value in only_probes["metrics"].values()))

    def test_duplicate_unknown_and_malformed_dataset_identity_are_rejected(self):
        data = dataset(clear("a"))
        invalid = [dataset(clear("a"), clear("a")), dataset(), dataset(clear(" ")),
                   dataset({**clear("a"), "text": " "}), dataset({**clear("a"), "expect_review": True}),
                   dataset({"id": "a", "text": "x", "expect_review": 1}),
                   dataset(clear("a", "unknown")), dataset(clear("a", priority="soon")),
                   dataset(*[clear(str(i)) for i in range(101)])]
        for value in invalid:
            with self.subTest(value=value), self.assertRaises(scoring.ContractError):
                scoring.score(value, [])
        for rows in [[row("a"), row("a")], [row("unknown")]]:
            with self.assertRaises(scoring.ContractError):
                scoring.score(data, rows)

    def test_routing_policy_requires_consistent_decided_answers(self):
        mutations = [
            lambda r: r.update(route="billing"),
            lambda r: r.update(priority="routine"),
            lambda r: r.update(status="failed"),
            lambda r: r.update(status="review", route=None, priority=None),
            lambda r: r["answers"]["department"].update(value=None),
            lambda r: r["answers"]["department"].update(value=True),
            lambda r: r["answers"]["department"].update(value="billing"),
            lambda r: r["answers"]["urgency"].update(value=True),
            lambda r: r["answers"]["urgency"].update(value=0.1),
            lambda r: r["answers"]["urgency"].update(confidence=0.8),
            lambda r: r["answers"]["department"].update(confidence=0.7),
        ]
        for mutate in mutations:
            value = row("a")
            mutate(value)
            with self.subTest(value=value), self.assertRaises(scoring.ContractError):
                scoring.score(dataset(clear("a")), [value])
        value = row("a", review=True)
        value["route"] = "technical"
        with self.assertRaises(scoring.ContractError):
            scoring.score(dataset(clear("a")), [value])

    def test_finite_probability_shape_and_sum_checks(self):
        bad_probabilities = [{"technical": 0.9}, {"technical": 0.9, "billing": 0.9},
                             {"technical": True, "billing": 0.0}, {"technical": -0.1, "billing": 1.1},
                             {"technical": float("nan"), "billing": 0.1},
                             {"technical": float("inf"), "billing": 0.1}]
        for probabilities in bad_probabilities:
            value = row("a")
            value["answers"]["department"]["probabilities"] = probabilities
            with self.assertRaises(scoring.ContractError):
                scoring.score(dataset(clear("a")), [value])
        value = row("a", review=True)
        value["answers"]["urgency"].update(value=None, confidence=0.17)
        with self.assertRaises(scoring.ContractError):
            scoring.score(dataset(clear("a")), [value])

    def test_threshold_equality_and_score_normalization_tolerance(self):
        value = row("a", "billing", "routine")
        value["answers"]["department"].update(confidence=0.8, probabilities={"technical": 0.2, "billing": 0.8})
        value["answers"]["urgency"].update(value=0.2, confidence=0.8, probabilities={"routine": 0.8, "soon": 0.2, "urgent": 0.0})
        self.assertEqual(scoring.score(dataset(clear("a", "billing", "routine")), [value])["counts"]["clear_correct"], 1)
        value["answers"]["urgency"]["value"] = 0.219
        scoring.score(dataset(clear("a", "billing", "routine")), [value])
        value["answers"]["urgency"]["value"] = 0.221
        with self.assertRaises(scoring.ContractError):
            scoring.score(dataset(clear("a", "billing", "routine")), [value])

    def test_shared_usage_counted_once_and_unknown_counters_are_separate(self):
        rows = [row("a"), row("b", metadata=call(None, None, 2)), row("c", metadata=None)]
        report = scoring.score(dataset(clear("a"), clear("b"), clear("c"), clear("missing")), rows)
        usage = report["usage_observations"]
        self.assertEqual(usage["known_input_tokens"], 17)
        self.assertEqual(usage["known_output_tokens"], 7)
        self.assertEqual(usage["rows_with_unknown_input_tokens"], 2)
        self.assertEqual(usage["rows_with_unknown_output_tokens"], 1)
        self.assertEqual(usage["rows_without_call_metadata"], 1)
        self.assertEqual(usage["completed_row_call_observations_lower_bound"], 3)
        self.assertEqual(report["reported_models"], [None, "reported-model"])
        self.assertNotIn("actual_calls", report)
        for bad in [True, -1, 0.5, 2**64, float("nan")]:
            value = row("a", metadata=call(input_tokens=bad))
            with self.subTest(counter=bad), self.assertRaises(scoring.ContractError):
                scoring.score(dataset(clear("a")), [value])
        value = row("a")
        value["answers"]["urgency"]["call"]["model"] = "different-model"
        with self.assertRaises(scoring.ContractError):
            scoring.score(dataset(clear("a")), [value])
        value["answers"]["urgency"]["call"] = None
        with self.assertRaises(scoring.ContractError):
            scoring.score(dataset(clear("a")), [value])

    def test_strict_jsonl_rejects_nonfinite_duplicate_keys_and_truncated_lines(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "results.jsonl"
            for text in ['{"id":"a","id":"b"}\n', '{"value":NaN}\n', '{"value":Infinity}\n',
                         '{"value":1e400}\n', '{"id":"a"\n', '{}\n\n', '[]\n']:
                path.write_text(text)
                with self.subTest(text=text), self.assertRaises(scoring.ContractError):
                    scoring.load_results(path)
            path.write_text("")
            self.assertEqual(scoring.load_results(path), [])
            path.write_text('{"schema":"x","schema":"y"}')
            with self.assertRaises(scoring.ContractError):
                scoring.load_dataset(path)

    def test_valid_unicode_line_separators_inside_records_are_not_record_boundaries(self):
        with tempfile.TemporaryDirectory() as directory:
            identity = "case\u2028one"
            path = Path(directory) / "results.jsonl"
            path.write_text(json.dumps(row(identity), ensure_ascii=False) + "\n", encoding="utf-8")
            rows = scoring.load_results(path)
            self.assertEqual(len(rows), 1)
            self.assertEqual(scoring.score(dataset(clear(identity)), rows)["counts"]["clear_correct"], 1)

    def test_file_hashes_bind_exact_bytes_and_cli_is_offline(self):
        with tempfile.TemporaryDirectory() as directory:
            data = Path(directory) / "dataset.json"
            results = Path(directory) / "results.jsonl"
            dataset_bytes = (json.dumps(dataset(clear("a")), indent=2) + "\n").encode()
            result_bytes = (json.dumps(row("a")) + "\n").encode()
            data.write_bytes(dataset_bytes)
            results.write_bytes(result_bytes)
            report = scoring.score_files(data, results)
            self.assertEqual(report["source_hashes"], {"dataset_sha256": hashlib.sha256(dataset_bytes).hexdigest(), "results_sha256": hashlib.sha256(result_bytes).hexdigest()})
            self.assertEqual(scoring.load_dataset(data)["cases"][0]["id"], "a")
            completed = subprocess.run([sys.executable, str(SCRIPT), "--dataset", str(data), "--results", str(results)], capture_output=True, text=True, check=False)
            self.assertEqual(completed.returncode, 0, completed.stderr)
            self.assertEqual(json.loads(completed.stdout), report)
            results.write_text('{"private":"DO_NOT_ECHO_PRIVATE_TEXT"')
            completed = subprocess.run([sys.executable, str(SCRIPT), "--dataset", str(data), "--results", str(results)], capture_output=True, text=True, check=False)
            self.assertEqual(completed.returncode, 2)
            self.assertEqual(completed.stdout, "")
            self.assertNotIn("DO_NOT_ECHO_PRIVATE_TEXT", completed.stderr)


if __name__ == "__main__":
    unittest.main()
