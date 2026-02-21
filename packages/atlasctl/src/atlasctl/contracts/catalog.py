from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path

from ..errors import ScriptError
from ..exit_codes import ERR_VALIDATION
from .schemas import schemas_root


@dataclass(frozen=True)
class CatalogEntry:
    name: str
    version: int
    file: str


def catalog_path() -> Path:
    return schemas_root() / "catalog.json"


def load_catalog() -> dict[str, CatalogEntry]:
    raw = json.loads(catalog_path().read_text(encoding="utf-8"))
    entries: dict[str, CatalogEntry] = {}
    for row in raw.get("schemas", []):
        name = str(row.get("name", "")).strip()
        file_name = str(row.get("file", "")).strip()
        if not name or not file_name:
            continue
        entries[name] = CatalogEntry(name=name, version=int(row["version"]), file=file_name)
    return entries


def schema_path_for(schema_name: str) -> Path:
    entry = load_catalog().get(schema_name)
    if entry is None:
        raise ScriptError(f"unknown schema: {schema_name}", ERR_VALIDATION)
    rel = Path(entry.file)
    if rel.is_absolute() or ".." in rel.parts:
        raise ScriptError(f"invalid schema path for {schema_name}: {entry.file}", ERR_VALIDATION)
    path = schemas_root() / rel
    if not path.exists():
        raise ScriptError(f"missing schema file for {schema_name}: {entry.file}", ERR_VALIDATION)
    return path
