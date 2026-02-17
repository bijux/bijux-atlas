#!/usr/bin/env python3
# Purpose: shared JSON helpers for script tooling.
# Inputs: filesystem JSON paths.
# Outputs: parsed JSON payloads with deterministic error messages.
from __future__ import annotations

import json
from pathlib import Path
from typing import Any


def read_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text())
    except Exception as exc:  # pragma: no cover
        raise SystemExit(f"failed to read json {path}: {exc}")
