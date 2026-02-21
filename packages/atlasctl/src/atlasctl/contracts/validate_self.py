from __future__ import annotations

from typing import Any

from .validate import validate


def validate_self(schema_name: str, payload: dict[str, Any]) -> dict[str, Any]:
    validate(schema_name, payload)
    return payload
