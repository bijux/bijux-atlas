"""Shared helpers for command modules."""

from __future__ import annotations

import json


def emit_payload(payload: dict[str, object], as_json: bool) -> str:
    """Render payload in deterministic JSON form for text/json command output."""
    if as_json:
        return json.dumps(payload, sort_keys=True)
    return json.dumps(payload, indent=2, sort_keys=True)
