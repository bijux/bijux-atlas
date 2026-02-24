"""Compatibility shim for `atlasctl.contracts.validate`."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from atlasctl.core.runtime.repo_root import find_repo_root

from .schema.validate import validate as validate_catalog
from .schema.validate import validate_file as validate_file_catalog

_LEGACY_SCHEMA_FILES = {
    "scripts-tool-output": Path("configs/contracts/scripts-tool-output.schema.json"),
    "ops.scenarios": Path("configs/ops/scenarios.schema.json"),
}


def _validate_legacy(schema_name: str, payload: Any) -> bool:
    rel = _LEGACY_SCHEMA_FILES.get(schema_name)
    if rel is None:
        return False
    import jsonschema

    schema_path = find_repo_root() / rel
    schema = json.loads(schema_path.read_text(encoding="utf-8"))
    jsonschema.validate(payload, schema)
    return True


def validate(schema_name: str, payload: Any) -> None:
    if _validate_legacy(schema_name, payload):
        return
    validate_catalog(schema_name, payload)


def validate_file(schema_name: str, file_path: str | Path) -> None:
    payload = json.loads(Path(file_path).read_text(encoding="utf-8"))
    validate(schema_name, payload)


__all__ = ["validate", "validate_file"]
