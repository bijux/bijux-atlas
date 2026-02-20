"""Canonical JSON serialization helpers."""

from __future__ import annotations

import json
from typing import Any


def dumps_json(payload: Any, pretty: bool = False) -> str:
    if pretty:
        return json.dumps(payload, indent=2, sort_keys=True)
    return json.dumps(payload, sort_keys=True)
