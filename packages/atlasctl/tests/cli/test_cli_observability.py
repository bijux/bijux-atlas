from __future__ import annotations

import json
from pathlib import Path

from helpers import run_atlasctl


ROOT = Path(__file__).resolve().parents[4]


def test_log_json_emits_structured_log_lines() -> None:
    proc = run_atlasctl("--log-json", "--run-id", "log-json-test", "help", "--json")
    assert proc.returncode == 0, proc.stderr
    lines = [line.strip() for line in proc.stderr.splitlines() if line.strip().startswith("{")]
    assert lines
    payload = json.loads(lines[0])
    assert payload["run_id"] == "log-json-test"
    assert payload["component"] == "cli"


def test_cli_writes_local_telemetry_events() -> None:
    run_id = "telemetry-local-test"
    proc = run_atlasctl("--run-id", run_id, "help")
    assert proc.returncode == 0, proc.stderr
    events = ROOT / "artifacts/isolate" / run_id / "atlasctl-telemetry" / "events.jsonl"
    assert events.exists()
    text = events.read_text(encoding="utf-8")
    assert "cli.start" in text
