"""Runner contract tests with mocked child processes; never contact a provider."""
from contextlib import ExitStack
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import Mock, patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import run_triage_check as runner
from test_triage_scoring import clear, dataset, row


class TriageRunnerTests(unittest.TestCase):
    def setUp(self):
        self.stack = ExitStack()
        self.addCleanup(self.stack.close)
        self.root = Path(self.stack.enter_context(tempfile.TemporaryDirectory()))
        self.dataset = self.root / "dataset.json"
        self.dataset.write_text(json.dumps(dataset(clear("a"), clear("b"))), encoding="utf-8")
        target = self.root / "target"
        binary = target / "debug" / ("semops-triage.exe" if os.name == "nt" else "semops-triage")
        binary.parent.mkdir(parents=True)
        binary.write_bytes(b"synthetic binary; never executed")
        self.stack.enter_context(patch.object(runner, "ROOT", self.root))
        self.stack.enter_context(patch.object(runner, "TARGET", target))
        self.stack.enter_context(patch.dict(os.environ, {"TYPESAFE_API_KEY": "DO_NOT_RETAIN_TEST_KEY"}))
        self.sources = self.stack.enter_context(patch.object(runner, "source_hashes", return_value={"src/lib.rs": "fixture-hash"}))
        self.build = self.stack.enter_context(patch.object(runner.subprocess, "run"))
        self.popen = self.stack.enter_context(patch.object(runner.subprocess, "Popen"))

    def invoke(self, rows=(), exit_code=0, malformed=False):
        def launch(command, **kwargs):
            self.assertNotIn("DO_NOT_RETAIN_TEST_KEY", " ".join(command))
            inputs = [json.loads(line) for line in kwargs["stdin"].read().splitlines()]
            self.assertEqual([set(value) for value in inputs], [{"id", "text"}] * 2)
            output = b'{"truncated":' if malformed else b"".join((json.dumps(value) + "\n").encode() for value in rows)
            kwargs["stdout"].write(output)
            kwargs["stderr"].write(b"synthetic diagnostic\n")
            process = Mock()
            process.wait.return_value = exit_code
            return process
        self.popen.side_effect = launch
        directory = runner.execute(self.dataset, "selected-model", 2)
        return directory, json.loads((directory / "receipt.json").read_text())

    def test_plan_is_offline_and_validates_whole_batch_limit(self):
        planned = runner.plan(self.dataset, None, None)
        self.assertFalse(planned["ready_to_invoke"])
        self.assertEqual(planned["planned_requests"], 2)
        self.assertEqual(planned["dataset_sha256"], runner.sha(self.dataset))
        for model, cap in [("", 2), ("bad\nmodel", 2), ("bad\x7fmodel", 2), ("model", 1), ("model", 101)]:
            with self.assertRaises(ValueError):
                runner.plan(self.dataset, model, cap)
        self.build.assert_not_called()
        self.popen.assert_not_called()
        self.assertFalse((self.root / "var").exists())

    def test_missing_key_or_changed_sources_prevents_launch(self):
        with patch.dict(os.environ, {"TYPESAFE_API_KEY": ""}), self.assertRaises(ValueError):
            runner.execute(self.dataset, "model", 2)
        self.build.assert_not_called()
        self.sources.side_effect = [{"source": "before"}, {"source": "after"}]
        with self.assertRaises(ValueError):
            runner.execute(self.dataset, "model", 2)
        self.popen.assert_not_called()

    def test_serialized_input_size_is_checked_before_build(self):
        data = dataset(*(clear(str(i)) for i in range(100)))
        for case in data["cases"]:
            case["text"] = "x" * (16 * 1024)
        self.dataset.write_text(json.dumps(data))
        with self.assertRaises(ValueError):
            runner.execute(self.dataset, "model", 100)
        self.build.assert_not_called()
        self.popen.assert_not_called()

    def test_dataset_changed_during_build_prevents_launch(self):
        self.build.side_effect = lambda *a, **k: self.dataset.write_text(json.dumps(dataset(clear("replacement"))))
        with self.assertRaises(ValueError):
            runner.execute(self.dataset, "model", 2)
        self.popen.assert_not_called()

    def test_complete_run_seals_hashes_and_deduplicated_usage_without_key(self):
        directory, receipt = self.invoke([row("a"), row("b")])
        self.assertTrue(receipt["complete"])
        self.assertTrue(receipt["process_started"])
        self.assertEqual(receipt["exit_code"], 0)
        self.assertEqual(receipt["retries"], 0)
        self.assertEqual(receipt["output_sha256"], runner.sha(directory / "output.jsonl"))
        self.assertEqual(receipt["dataset_sha256"], runner.sha(directory / "dataset.json"))
        self.assertEqual(receipt["source_sha256"], {"src/lib.rs": "fixture-hash"})
        report = json.loads((directory / "report.json").read_text())
        self.assertEqual(report["usage_observations"]["known_input_tokens"], 34)
        self.build.assert_called_once()
        self.popen.assert_called_once()
        for path in directory.iterdir():
            self.assertNotIn("DO_NOT_RETAIN_TEST_KEY", path.read_text())
        if os.name != "nt":
            self.assertEqual(directory.stat().st_mode & 0o777, 0o700)

    def test_partial_failure_is_scored_but_not_complete(self):
        directory, receipt = self.invoke([row("a")], exit_code=1)
        self.assertFalse(receipt["complete"])
        self.assertEqual(json.loads((directory / "report.json").read_text())["counts"]["missing"], 1)
        self.popen.assert_called_once()

    def test_malformed_output_is_retained_and_not_silently_scored(self):
        directory, receipt = self.invoke(malformed=True)
        self.assertFalse(receipt["complete"])
        self.assertIn("scoring_error", receipt)
        self.assertFalse((directory / "report.json").exists())
        self.assertEqual((directory / "output.jsonl").read_bytes(), b'{"truncated":')

    def test_timeout_and_interrupt_kill_and_reap_child_without_retry(self):
        for error, label in [(subprocess.TimeoutExpired("synthetic", 1), "outer_process_timeout"),
                             (KeyboardInterrupt(), "interrupted")]:
            process = Mock()
            process.wait.side_effect = [error, -9]
            self.popen.return_value = process
            result = runner.run_consumer(["synthetic"], None, None, None, 1)
            self.assertEqual(result["execution_error"], label)
            self.assertEqual(result["exit_code"], -9)
            process.kill.assert_called_once()
            self.assertEqual(process.wait.call_count, 2)
        self.assertEqual(self.popen.call_count, 2)

    def test_launch_failure_is_unknown_usage_and_zero_completed_rows(self):
        self.popen.side_effect = OSError("synthetic failure")
        directory = runner.execute(self.dataset, "model", 2)
        receipt = json.loads((directory / "receipt.json").read_text())
        self.assertFalse(receipt["process_started"])
        self.assertFalse(receipt["complete"])
        self.assertEqual(receipt["execution_error"], "could_not_start_process")
        self.assertEqual(json.loads((directory / "report.json").read_text())["counts"]["missing"], 2)


if __name__ == "__main__":
    unittest.main()
