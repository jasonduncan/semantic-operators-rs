#!/usr/bin/env python3
"""Plan or explicitly execute one bounded synthetic triage check. Default: no calls."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import uuid

from score_triage_check import load_dataset, parse_dataset, score_files

ROOT = Path(__file__).resolve().parents[1]
DATASET = ROOT / "qualification" / "triage-v1.json"
MANIFEST = ROOT / "integrations" / "triage-app" / "Cargo.toml"
TARGET = ROOT / "target" / "triage-app"
ENDPOINT = "https://api.typesafe.ai/v1/systemone"
TIMEOUT_MS = 30_000


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def input_bytes(dataset: dict) -> bytes:
    data = "".join(json.dumps({"id": c["id"], "text": c["text"]}) + "\n" for c in dataset["cases"]).encode("utf-8")
    if len(data) > 1024 * 1024:
        raise ValueError("serialized input exceeds consumer 1 MiB limit")
    return data


def plan(dataset_path: Path, model: str | None, max_calls: int | None) -> dict:
    dataset_bytes = dataset_path.read_bytes()
    dataset = parse_dataset(dataset_bytes)
    serialized = input_bytes(dataset)
    if model is not None and (not model.strip() or any(ord(c) < 32 or ord(c) == 127 for c in model)):
        raise ValueError("model must be nonblank and contain no control characters")
    if max_calls is not None and (max_calls < len(dataset["cases"]) or max_calls > 100):
        raise ValueError("call limit must cover the complete dataset and be at most 100")
    return {
        "schema": "semops.triage-run/1", "mode": "plan", "dataset_sha256": hashlib.sha256(dataset_bytes).hexdigest(),
        "case_count": len(dataset["cases"]), "planned_requests": len(dataset["cases"]),
        "input_bytes": len(serialized),
        "questions_per_request": 2, "requested_model": model, "max_calls": max_calls,
        "endpoint": ENDPOINT, "timeout_ms_per_request": TIMEOUT_MS,
        "retries": 0, "warmup": 0, "credential_reference": "environment:TYPESAFE_API_KEY",
        "ready_to_invoke": model is not None and max_calls is not None,
        "label_authority": dataset["label_authority"],
        "note": "Plan only; ready_to_invoke is configuration readiness, not execution or authorization.",
    }


def write_json(path: Path, value: dict) -> None:
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n", encoding="utf-8")


def source_hashes() -> dict:
    paths = [ROOT / "Cargo.toml", MANIFEST, MANIFEST.parent / "Cargo.lock",
             MANIFEST.parent / "src/main.rs", ROOT / "tools/run_triage_check.py",
             ROOT / "tools/score_triage_check.py"]
    paths.extend(sorted((ROOT / "src").rglob("*.rs")))
    return {str(path.relative_to(ROOT)): sha(path) for path in paths}


def run_consumer(command: list[str], source, output, errors, timeout: int) -> dict:
    try:
        process = subprocess.Popen(command, stdin=source, stdout=output, stderr=errors, cwd=ROOT)
    except OSError:
        return {"process_started": False, "execution_error": "could_not_start_process"}
    result = {"process_started": True}
    try:
        result["exit_code"] = process.wait(timeout=timeout)
    except (subprocess.TimeoutExpired, KeyboardInterrupt) as error:
        result["execution_error"] = "interrupted" if isinstance(error, KeyboardInterrupt) else "outer_process_timeout"
        # Stop and reap the child before sealing evidence. Never retry an uncertain call.
        try:
            process.kill()
        except ProcessLookupError:
            pass  # Child exited between wait and kill; still reap it below.
        result["exit_code"] = process.wait()
    return result


def execute(dataset_path: Path, model: str, max_calls: int) -> Path:
    receipt = plan(dataset_path, model, max_calls)
    if not os.environ.get("TYPESAFE_API_KEY"):
        raise ValueError("TYPESAFE_API_KEY is not set")
    recorded_sources = source_hashes()
    # Rebuild from the recorded checkout; fail if inputs changed during the build.
    subprocess.run(["cargo", "build", "--locked", "--offline", "--manifest-path", str(MANIFEST),
                    "--target-dir", str(TARGET)], cwd=ROOT, check=True)
    if source_hashes() != recorded_sources:
        raise ValueError("source changed during build; no request made")
    binary = TARGET / "debug" / ("semops-triage.exe" if os.name == "nt" else "semops-triage")
    # Keep artifacts private by default. The run directory is unique and never reused.
    previous_umask = os.umask(0o077)
    try:
        directory = ROOT / "var" / ("triage-check-" + datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ-") + uuid.uuid4().hex[:8])
        directory.mkdir(parents=True, mode=0o700, exist_ok=False)
        frozen = directory / "dataset.json"
        frozen.write_bytes(dataset_path.read_bytes())
        dataset = load_dataset(frozen)
        if sha(frozen) != receipt["dataset_sha256"]:
            raise ValueError("dataset changed after planning; no request made")
        inputs = directory / "input.jsonl"
        inputs.write_bytes(input_bytes(dataset))
        receipt.update({
            "mode": "execution", "started_at": now(), "binary_sha256": sha(binary),
            "source_sha256": recorded_sources,
            "input_sha256": sha(inputs), "exit_code": None, "finished_at": None,
            "process_started": False, "complete": False,
        })
        write_json(directory / "receipt.json", receipt)
        command = [str(binary), "--live", "--endpoint", ENDPOINT, "--model", model,
                   "--max-calls", str(max_calls), "--timeout-ms", str(TIMEOUT_MS)]
        receipt["command"] = command
        # Null means launch outcome unknown if this runner is abruptly terminated.
        receipt["process_started"] = None
        write_json(directory / "receipt.json", receipt)
        with inputs.open("rb") as source, (directory / "output.jsonl").open("wb") as output, (directory / "stderr.txt").open("wb") as errors:
            receipt.update(run_consumer(command, source, output, errors,
                                        timeout=len(dataset["cases"]) * 30 + 15))
        receipt["finished_at"] = now()
        receipt["output_sha256"] = sha(directory / "output.jsonl")
        receipt["stderr_sha256"] = sha(directory / "stderr.txt")
        try:
            report = score_files(frozen, directory / "output.jsonl")
            write_json(directory / "report.json", report)
            # Zero exit and all rows are required; label agreement is a separate metric.
            receipt["complete"] = receipt["exit_code"] == 0 and report["counts"]["missing"] == 0
        except (ValueError, UnicodeError, OSError):
            receipt["scoring_error"] = "output_not_scoreable; preserve original artifacts"
        write_json(directory / "receipt.json", receipt)
        return directory
    finally:
        os.umask(previous_umask)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dataset", type=Path, default=DATASET)
    parser.add_argument("--model")
    parser.add_argument("--max-calls", type=int)
    parser.add_argument("--execute", action="store_true", help="invoke the real provider once; requires model and call limit")
    args = parser.parse_args()
    try:
        proposed = plan(args.dataset, args.model, args.max_calls)
        if not args.execute:
            print(json.dumps(proposed, indent=2))
            return 0
        if not proposed["ready_to_invoke"]:
            raise ValueError("execution requires an exact model and call limit")
        directory = execute(args.dataset, args.model, args.max_calls)
        receipt = json.loads((directory / "receipt.json").read_text())
        print(json.dumps({"run_directory": str(directory), "exit_code": receipt["exit_code"],
                          "complete": receipt["complete"]}, indent=2))
        return 0 if receipt["complete"] else 1
    except (ValueError, OSError, subprocess.CalledProcessError):
        print("check configuration, dataset or local build; no automatic retry", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
