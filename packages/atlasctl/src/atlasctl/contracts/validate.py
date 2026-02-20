from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from ..errors import ScriptError
from ..exit_codes import ERR_VALIDATION
from .schemas import schemas_root


@dataclass(frozen=True)
class CatalogEntry:
    name: str
    version: int
    file: str


def _catalog_path() -> Path:
    return schemas_root() / "catalog.json"


def load_catalog() -> dict[str, CatalogEntry]:
    raw = json.loads(_catalog_path().read_text(encoding="utf-8"))
    entries: dict[str, CatalogEntry] = {}
    for row in raw.get("schemas", []):
        entry = CatalogEntry(name=row["name"], version=int(row["version"]), file=row["file"])
        entries[entry.name] = entry
    return entries


def schema_path(schema_name: str) -> Path:
    catalog = load_catalog()
    entry = catalog.get(schema_name)
    if entry is None:
        raise ScriptError(f"unknown schema: {schema_name}", ERR_VALIDATION)
    return schemas_root() / entry.file


def validate(schema_name: str, payload: Any) -> None:
    import jsonschema

    schema = json.loads(schema_path(schema_name).read_text(encoding="utf-8"))
    try:
        jsonschema.validate(payload, schema)
    except jsonschema.ValidationError as exc:
        pointer = "/".join(str(p) for p in exc.absolute_path)
        loc = pointer or "<root>"
        raise ScriptError(f"schema validation failed for {schema_name} at {loc}: {exc.message}", ERR_VALIDATION) from exc


def validate_file(schema_name: str, file_path: str | Path) -> None:
    payload = json.loads(Path(file_path).read_text(encoding="utf-8"))
    validate(schema_name, payload)
