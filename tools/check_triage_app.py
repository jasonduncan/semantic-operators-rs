#!/usr/bin/env python3
"""Exercise the standalone Rust consumer against a synthetic loopback HTTP server.

No provider credentials or hosted inference are used. The entire JSONL batch is
validated before dispatch. HTTP failures test preserved earlier output and bounded
call counts. Run with --skip-build when CI has already built the consumer.
"""
from __future__ import annotations

import argparse
from contextlib import contextmanager
import json
import os
from pathlib import Path
import subprocess
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import Iterator

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "integrations" / "triage-app" / "Cargo.toml"
TARGET = ROOT / "target" / "triage-app"
SECRET = "synthetic-triage-key-NOT-A-REAL-CREDENTIAL"
MODEL = "synthetic-selected-model"
PRIVATE_TEXT = "synthetic-private-content-DO-NOT-ECHO"


def response(department: str = "technical", department_p: float = 0.9,
             score: float = 1.8, probabilities: tuple[float, float, float] = (0.0, 0.2, 0.8)) -> dict:
    other = "billing" if department == "technical" else "technical"
    return {
        "model": "synthetic-reported-model",
        "usage": {"input_tokens": 17, "output_tokens": 5},
        "answers": {
            "department": {"type": "choice", "choice": department,
                           "confidence": 0.1,
                           "probabilities": {department: department_p, other: round(1 - department_p, 10)}},
            "urgency": {"type": "score", "score": score, "confidence": 0.1,
                        "legend": {"0": "routine", "1": "soon", "2": "urgent"},
                        "probabilities": {str(i): p for i, p in enumerate(probabilities)}},
        },
    }


class FixtureServer(ThreadingHTTPServer):
    daemon_threads = True

    def __init__(self, fixtures: list[tuple[int, dict | bytes]]) -> None:
        super().__init__(("127.0.0.1", 0), Handler)
        self.fixtures = fixtures
        self.requests: list[dict] = []
        self.failures: list[str] = []
        self.lock = threading.Lock()

    @property
    def endpoint(self) -> str:
        return f"http://127.0.0.1:{self.server_port}/v1/systemone"


class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def setup(self) -> None:
        super().setup()
        self.connection.settimeout(3)

    def log_message(self, *_args: object) -> None:
        pass

    def do_POST(self) -> None:
        server: FixtureServer = self.server  # type: ignore[assignment]
        length = int(self.headers.get("Content-Length", "0"))
        data = self.rfile.read(length)
        with server.lock:
            index = len(server.requests)
            try:
                body = json.loads(data)
            except (ValueError, UnicodeError):
                body = None
                server.failures.append("consumer sent invalid JSON")
            server.requests.append({"path": self.path, "headers": dict(self.headers), "body": body})
            if index >= len(server.fixtures):
                server.failures.append("unexpected additional HTTP call")
                status, value = 500, {"error": "unexpected call"}
            else:
                status, value = server.fixtures[index]
        payload = value if isinstance(value, bytes) else json.dumps(value).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(payload)))
        self.send_header("Connection", "close")
        self.end_headers()
        try:
            self.wfile.write(payload)
        except (BrokenPipeError, ConnectionResetError):
            pass


@contextmanager
def server(fixtures: list[tuple[int, dict | bytes]]) -> Iterator[FixtureServer]:
    fixture = FixtureServer(fixtures)
    thread = threading.Thread(target=fixture.serve_forever, kwargs={"poll_interval": 0.01}, daemon=True)
    thread.start()
    try:
        yield fixture
    finally:
        fixture.shutdown()
        fixture.server_close()
        thread.join(timeout=2)
        assert not thread.is_alive(), "fixture server did not stop"


def lines(records: list[dict]) -> str:
    return "".join(json.dumps(record, ensure_ascii=False) + "\n" for record in records)


def run(binary: Path, fixture: FixtureServer, text: str | bytes, *, max_calls: int = 10,
        extra: list[str] | None = None, timeout_ms: int = 2000) -> subprocess.CompletedProcess[str]:
    environment = os.environ.copy()
    environment["TYPESAFE_API_KEY"] = SECRET
    for key in list(environment):
        if key.lower().endswith("_proxy"):
            del environment[key]
    # Keep all fixture traffic local even on development hosts with a proxy.
    environment["NO_PROXY"] = "127.0.0.1,localhost"
    environment["no_proxy"] = "127.0.0.1,localhost"
    command = [str(binary), "--endpoint", fixture.endpoint, "--model", MODEL,
               "--max-calls", str(max_calls), "--timeout-ms", str(timeout_ms)]
    if extra:
        command.extend(extra)
    raw = subprocess.run(command, input=text.encode("utf-8") if isinstance(text, str) else text,
                         capture_output=True, env=environment, timeout=15, check=False)
    completed = subprocess.CompletedProcess(raw.args, raw.returncode,
                                            raw.stdout.decode("utf-8"), raw.stderr.decode("utf-8"))
    # Assertions deliberately never print process input/output or request bodies.
    assert SECRET not in completed.stdout + completed.stderr, "credential appeared in process output"
    assert PRIVATE_TEXT not in completed.stdout + completed.stderr, "private input appeared in process output"
    assert not fixture.failures, "; ".join(fixture.failures)
    return completed


def outputs(completed: subprocess.CompletedProcess[str]) -> list[dict]:
    try:
        return [json.loads(line) for line in completed.stdout.splitlines()]
    except ValueError as error:
        raise AssertionError("consumer output is not JSONL") from error


def check_request(request: dict, expected_text: str) -> None:
    assert request["path"] == "/v1/systemone", "wrong endpoint path"
    headers = {key.lower(): value for key, value in request["headers"].items()}
    assert headers.get("authorization") == f"Bearer {SECRET}", "wrong authorization header"
    assert headers.get("content-type", "").startswith("application/json"), "wrong content type"
    body = request["body"]
    assert body["model"] == MODEL, "configured model was not sent"
    assert body["state"] == expected_text, "state must contain only record text"
    assert set(body["questions"]) == {"department", "urgency"}, "wrong question batch"
    department = body["questions"]["department"]
    assert department["type"] == "choice", "department is not a choice"
    assert list(department["criteria"]) == ["technical", "billing"], "department order changed"
    urgency = body["questions"]["urgency"]
    assert urgency["type"] == "score", "urgency is not a score"
    assert urgency["criteria"] == ["routine", "soon", "urgent"], "urgency levels changed"


def success_cases(binary: Path) -> int:
    fixtures = [
        response(),
        response("billing", 0.8, 0.2, (0.8, 0.2, 0.0)),
        response("technical", 0.5),
        response("technical", 0.79),
        response("technical", 0.9, 1.5, (0.0, 0.5, 0.5)),
        response("technical", 0.9, 1.2, (0.0, 0.8, 0.2)),
    ]
    records = [{"id": f"row-{i}", "text": PRIVATE_TEXT + f" {i}"} for i in range(len(fixtures))]
    with server([(200, value) for value in fixtures]) as fixture:
        completed = run(binary, fixture, lines(records), max_calls=len(records))
        assert completed.returncode == 0, "success batch failed"
        assert not completed.stderr, "successful consumer wrote diagnostics"
        rows = outputs(completed)
        assert len(rows) == len(records), "success row count differs"
        assert len(fixture.requests) == len(records), "success call count differs"
        for request, record in zip(fixture.requests, records, strict=True):
            check_request(request, record["text"])
        expected = [("routed", "technical", "urgent"), ("routed", "billing", "routine"),
                    ("review", None, None), ("review", None, None), ("review", None, None),
                    ("routed", "technical", "routine")]
        for row, record, expected_fields in zip(rows, records, expected, strict=True):
            assert row["id"] == record["id"], "record order or identity changed"
            assert (row["status"], row["route"], row["priority"]) == expected_fields, "incorrect routing or abstention"
            assert set(row["answers"]) == {"department", "urgency"}, "answer evidence missing"
            for answer in row["answers"].values():
                assert set(answer) >= {"value", "confidence", "probabilities", "call"}, "incomplete answer evidence"
                assert answer["call"]["provider"] == "TypeSafe", "provider identity lost"
                assert answer["call"]["model"] == "synthetic-reported-model", "reported model identity lost"
                assert answer["call"]["input_tokens"] == 17, "reported usage lost"
                assert answer["call"]["output_tokens"] == 5, "reported usage lost"
        assert rows[1]["answers"]["department"]["confidence"] == 0.8, "threshold equality must pass"
        assert rows[1]["answers"]["urgency"]["confidence"] == 0.8, "score threshold equality must pass"
        assert rows[2]["answers"]["department"]["value"] is None, "tie must abstain"
        assert rows[3]["answers"]["department"]["value"] is None, "below-threshold value must be null"
    return 6


def missing_metadata(binary: Path) -> int:
    value = response()
    del value["model"]
    del value["usage"]
    with server([(200, value)]) as fixture:
        completed = run(binary, fixture, lines([{"id": "unknown-metadata", "text": PRIVATE_TEXT}]), max_calls=1)
        assert completed.returncode == 0, "absent metadata must not invent a provider failure"
        rows = outputs(completed)
        assert len(rows) == 1 and rows[0]["status"] == "routed", "metadata absence changed the decision"
        for answer in rows[0]["answers"].values():
            call = answer["call"]
            assert call["provider"] == "TypeSafe", "adapter identity must remain available"
            assert call["model"] is None, "selected model must not replace absent reported model"
            assert call["input_tokens"] is None and call["output_tokens"] is None, "absent usage must remain null"
        assert len(fixture.requests) == 1, "absent metadata changed call accounting"
    return 1


def failure_cases(binary: Path) -> int:
    malformed = response()
    malformed["answers"]["department"]["probabilities"] = {"technical": 0.9, "billing": 0.9}
    count = 0
    for failure in [(429, {"error": PRIVATE_TEXT}), (200, b"not JSON synthetic-private-content-DO-NOT-ECHO"), (200, malformed)]:
        with server([(200, response()), failure, (200, response())]) as fixture:
            records = [{"id": f"row-{i}", "text": PRIVATE_TEXT} for i in range(3)]
            completed = run(binary, fixture, lines(records))
            assert completed.returncode == 1, "provider failure must exit 1"
            rows = outputs(completed)
            assert len(rows) == 1 and rows[0]["id"] == "row-0", "earlier output must remain, failing row must not emit output"
            assert len(fixture.requests) == 2, "provider failure retried or continued later rows"
            assert completed.stderr.strip(), "failure category diagnostic missing"
        count += 1
    return count


def invalid_inputs(binary: Path) -> int:
    valid = {"id": "first", "text": PRIVATE_TEXT}
    cases = [
        (lines([valid, valid]), 10),
        (lines([valid, {"id": "second", "text": "x", "unexpected": True}]), 10),
        (lines([valid]) + "{broken\n", 10),
        (lines([valid, {"id": "second", "text": "x"}]), 1),
        (lines([{"id": " ", "text": "x"}]), 10),
        (lines([{"id": "first", "text": " \t"}]), 10),
        (lines([{"id": "first"}]), 10),
        (lines([{"id": "first", "text": 42}]), 10),
        ("[]\n", 10),
        (lines([{"id": "first", "text": "x" * (16 * 1024 + 1)}]), 10),
        (lines([{"id": str(i), "text": "x"} for i in range(101)]), 100),
        (lines([{"id": "first", "text": "é" * 8193}]), 10),
        (lines([{"id": "x" * 1025, "text": "x"}]), 10),
        (lines([valid]).encode("utf-8") + b"\xff\n", 10),
        (" " * (1024 * 1024 + 1), 10),
    ]
    for text, budget in cases:
        with server([]) as fixture:
            completed = run(binary, fixture, text, max_calls=budget)
            assert completed.returncode == 2, "invalid input must exit 2"
            assert completed.stdout == "", "invalid input produced partial output"
            assert fixture.requests == [], "invalid batch made an HTTP call"
    return len(cases)


def configuration_and_empty(binary: Path) -> int:
    with server([]) as fixture:
        completed = run(binary, fixture, "", max_calls=1)
        assert completed.returncode == 0 and completed.stdout == "", "empty input must succeed without output"
        assert not fixture.requests, "empty input made a call"
    for extra, budget in [([], 0), (["--unknown"], 1), (["--timeout-ms", "0"], 1)]:
        with server([]) as fixture:
            completed = run(binary, fixture, lines([{"id": "one", "text": PRIVATE_TEXT}]), max_calls=budget, extra=extra)
            assert completed.returncode == 2, "invalid config must exit 2"
            assert completed.stdout == "" and not fixture.requests, "invalid config caused work"
    with server([]) as fixture:
        completed = run(binary, fixture, "", timeout_ms=0)
        assert completed.returncode == 2 and not fixture.requests, "zero timeout must fail before dispatch"
    # Empty input ensures this gate test cannot send a request even if the gate regresses.
    environment = os.environ.copy()
    environment["TYPESAFE_API_KEY"] = SECRET
    completed = subprocess.run([str(binary), "--model", MODEL, "--max-calls", "1"],
                               input="", text=True, capture_output=True, env=environment, timeout=5)
    assert completed.returncode == 2 and completed.stdout == "", "default remote endpoint requires --live"
    count = 6
    if os.name == "posix":
        completed = subprocess.run([os.fsencode(binary), b"--model", b"\xff", b"--max-calls", b"1"],
                                   input=b"", capture_output=True, timeout=5, check=False)
        assert completed.returncode == 2 and completed.stdout == b"", "non-UTF8 argv must return configuration error"
        assert b"panicked" not in completed.stderr and b"\xff" not in completed.stderr, "invalid argv leaked or panicked"
        count += 1
    return count


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--skip-build", action="store_true")
    parser.add_argument("--binary", type=Path)
    arguments = parser.parse_args()
    if not arguments.skip_build:
        subprocess.run(["cargo", "build", "--locked", "--manifest-path", str(MANIFEST),
                        "--target-dir", str(TARGET)], cwd=ROOT, check=True)
    binary = arguments.binary or TARGET / "debug" / ("semops-triage.exe" if os.name == "nt" else "semops-triage")
    assert binary.is_file(), "consumer binary is missing; build it or pass --binary"
    count = success_cases(binary) + missing_metadata(binary) + failure_cases(binary) + invalid_inputs(binary) + configuration_and_empty(binary)
    print(json.dumps({"status": "passed", "scenarios": count, "provider": "synthetic-loopback-only", "live_calls": 0}))


if __name__ == "__main__":
    main()
