from __future__ import annotations

import json
from pathlib import Path

from ...contracts.validate import load_catalog, validate


def check_schema_catalog_integrity(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    catalog_path = repo_root / "packages/atlasctl/src/atlasctl/contracts/schemas/catalog.json"
    raw = json.loads(catalog_path.read_text(encoding="utf-8"))
    names: set[str] = set()
    for row in raw.get("schemas", []):
        name = str(row.get("name", ""))
        if not name:
            errors.append("schema catalog entry missing name")
            continue
        if name in names:
            errors.append(f"duplicate schema name in catalog: {name}")
        names.add(name)
        file_name = str(row.get("file", ""))
        if not file_name:
            errors.append(f"{name}: missing file")
            continue
        schema_file = catalog_path.parent / file_name
        if not schema_file.exists():
            errors.append(f"{name}: schema file missing: {schema_file.relative_to(repo_root)}")
    return (0 if not errors else 1), sorted(errors)


def check_schema_samples_validate(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    samples = sorted((repo_root / "packages/atlasctl/tests/goldens/samples").glob("*.json"))
    if not samples:
        return 1, ["no sample payloads found under packages/atlasctl/tests/goldens/samples"]
    catalog = load_catalog()
    for sample in samples:
        payload = json.loads(sample.read_text(encoding="utf-8"))
        schema_name = payload.get("schema_name")
        if not isinstance(schema_name, str):
            errors.append(f"{sample.name}: missing schema_name")
            continue
        if schema_name not in catalog:
            errors.append(f"{sample.name}: unknown schema_name {schema_name}")
            continue
        expected_version = int(catalog[schema_name].version)
        if int(payload.get("schema_version", -1)) != expected_version:
            errors.append(f"{sample.name}: schema_version mismatch for {schema_name} (expected {expected_version})")
            continue
        try:
            validate(schema_name, payload)
        except Exception as exc:
            errors.append(f"{sample.name}: {exc}")
    return (0 if not errors else 1), sorted(errors)
