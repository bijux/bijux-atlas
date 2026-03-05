from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.cli.observability import (
    ExecutionTelemetry,
    classify_error,
    emit_audit,
    emit_telemetry,
    emit_trace,
)


class ObservabilityTests(unittest.TestCase):
    def test_classification(self) -> None:
        self.assertEqual(classify_error(2, "invalid config"), "cli.config")
        self.assertEqual(classify_error(1, "permission denied"), "cli.authorization")
        self.assertEqual(classify_error(1, "not found"), "cli.input")
        self.assertEqual(classify_error(1, "unknown"), "cli.runtime")

    def test_emit_payloads(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            telemetry_path = root / "telemetry.json"
            trace_path = root / "trace.json"
            audit_path = root / "audit.json"
            emit_telemetry(
                telemetry_path,
                ExecutionTelemetry(command=["bijux-dev-atlas", "api", "list"], started_at=10.0, finished_at=11.5, exit_code=0),
            )
            emit_trace(trace_path, "cli.command.completed", {"exit_code": 0})
            emit_audit(audit_path, "command_run", "success", {"classification": "none"})
            self.assertEqual(json.loads(telemetry_path.read_text())["exit_code"], 0)
            self.assertEqual(json.loads(trace_path.read_text())["event"], "cli.command.completed")
            self.assertEqual(json.loads(audit_path.read_text())["status"], "success")


if __name__ == "__main__":
    unittest.main()
