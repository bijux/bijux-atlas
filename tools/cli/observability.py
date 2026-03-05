#!/usr/bin/env python3
from __future__ import annotations

import json
import time
from dataclasses import dataclass
from pathlib import Path


@dataclass(slots=True)
class ExecutionTelemetry:
    command: list[str]
    started_at: float
    finished_at: float
    exit_code: int

    @property
    def duration_ms(self) -> int:
        return int((self.finished_at - self.started_at) * 1000)


def classify_error(exit_code: int, stderr: str) -> str:
    lowered = stderr.lower()
    if exit_code == 0:
        return "none"
    if "config" in lowered or "invalid" in lowered:
        return "cli.config"
    if "permission" in lowered or "denied" in lowered:
        return "cli.authorization"
    if "not found" in lowered or "no such file" in lowered:
        return "cli.input"
    return "cli.runtime"


def write_json(path: Path, payload: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def emit_telemetry(path: Path, telemetry: ExecutionTelemetry) -> None:
    write_json(
        path,
        {
            "command": telemetry.command,
            "started_at": telemetry.started_at,
            "finished_at": telemetry.finished_at,
            "duration_ms": telemetry.duration_ms,
            "exit_code": telemetry.exit_code,
        },
    )


def emit_trace(path: Path, event: str, fields: dict[str, object]) -> None:
    write_json(path, {"event": event, "fields": fields, "timestamp": time.time()})


def emit_audit(path: Path, action: str, status: str, details: dict[str, object]) -> None:
    write_json(
        path,
        {
            "action": action,
            "status": status,
            "details": details,
            "timestamp": time.time(),
        },
    )
