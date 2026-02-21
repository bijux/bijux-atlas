from __future__ import annotations

import json
import re
from dataclasses import dataclass
from pathlib import Path

from ...errors import ScriptError
from ...exit_codes import ERR_VALIDATION
from .schemas import schemas_root


@dataclass(frozen=True)
class CatalogEntry:
    name: str
    version: int
    file: str


def catalog_path() -> Path:
    return schemas_root() / "catalog.json"


_SCHEMA_ID_RE = re.compile(r"^atlasctl\.[a-z0-9][a-z0-9._-]*\.v[1-9][0-9]*$")


def _raw_catalog() -> dict[str, object]:
    return json.loads(catalog_path().read_text(encoding="utf-8"))


def list_catalog_entries() -> list[CatalogEntry]:
    raw = _raw_catalog()
    rows: list[CatalogEntry] = []
    for row in raw.get("schemas", []):
        name = str(row.get("name", "")).strip()
        file_name = str(row.get("file", "")).strip()
        if not name or not file_name:
            continue
        rows.append(CatalogEntry(name=name, version=int(row["version"]), file=file_name))
    return rows


def load_catalog() -> dict[str, CatalogEntry]:
    entries: dict[str, CatalogEntry] = {}
    for row in list_catalog_entries():
        entries[row.name] = row
    return entries


def lint_catalog() -> list[str]:
    errors: list[str] = []
    entries = list_catalog_entries()
    names = [e.name for e in entries]
    if names != sorted(names):
        errors.append("schema catalog order must be sorted by schema name")
    if len(names) != len(set(names)):
        errors.append("schema catalog contains duplicate schema names")

    catalog_files: set[str] = set()
    for entry in entries:
        catalog_files.add(entry.file)
        if not _SCHEMA_ID_RE.match(entry.name):
            errors.append(f"invalid schema id format: {entry.name}")
        suffix = entry.name.rsplit(".v", 1)
        if len(suffix) != 2 or str(entry.version) != suffix[1]:
            errors.append(f"schema version mismatch for {entry.name}: catalog version={entry.version}")
        rel = Path(entry.file)
        if rel.is_absolute() or ".." in rel.parts:
            errors.append(f"{entry.name}: invalid schema path {entry.file}")
            continue
        if not (schemas_root() / rel).exists():
            errors.append(f"{entry.name}: missing schema file {entry.file}")

    disk_files = {path.name for path in schemas_root().glob("*.schema.json")}
    missing_from_catalog = sorted(disk_files - catalog_files)
    if missing_from_catalog:
        errors.append(f"schema files not in catalog: {missing_from_catalog}")
    unknown_in_catalog = sorted(catalog_files - disk_files)
    if unknown_in_catalog:
        errors.append(f"catalog references unknown schema files: {unknown_in_catalog}")
    return sorted(errors)


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
