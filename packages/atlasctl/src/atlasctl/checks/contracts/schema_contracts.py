from __future__ import annotations

import json
import re
from pathlib import Path

from ...contracts.catalog import catalog_path, load_catalog
from ...contracts.validate import validate


def check_schema_catalog_integrity(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    cat_path = catalog_path()
    raw = json.loads(cat_path.read_text(encoding="utf-8"))
    extra_catalog_files = [
        path.name
        for path in cat_path.parent.glob("*catalog*.json")
        if path.name != "catalog.json"
    ]
    if extra_catalog_files:
        errors.append(f"unexpected extra schema catalog files: {sorted(extra_catalog_files)}")
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
        rel = Path(file_name)
        if rel.is_absolute() or ".." in rel.parts:
            errors.append(f"{name}: schema path must be repo-relative without traversal: {file_name}")
            continue
        schema_file = cat_path.parent / rel
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


def check_schema_catalog_referenced(repo_root: Path) -> tuple[int, list[str]]:
    catalog = load_catalog()
    referenced: set[str] = set()
    pattern = re.compile(r"atlasctl\.[a-z0-9_.]+\.v\d+")
    scan_roots = [
        repo_root / "packages/atlasctl/src/atlasctl",
        repo_root / "packages/atlasctl/tests",
        repo_root / "docs",
    ]
    for root in scan_roots:
        if not root.exists():
            continue
        for path in root.rglob("*"):
            if not path.is_file() or path.suffix not in {".py", ".md", ".json", ".golden"}:
                continue
            text = path.read_text(encoding="utf-8", errors="ignore")
            referenced.update(pattern.findall(text))
    unused = sorted(name for name in catalog if name not in referenced)
    if unused:
        return 1, [f"schema catalog contains unreferenced schema: {name}" for name in unused]
    return 0, []


def check_schema_goldens_validate(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    catalog = load_catalog()
    golden_files = sorted((repo_root / "packages/atlasctl/tests/goldens").glob("*.json.golden"))
    for golden in golden_files:
        text = golden.read_text(encoding="utf-8", errors="ignore").strip()
        if not text.startswith("{"):
            continue
        try:
            payload = json.loads(text)
        except json.JSONDecodeError:
            continue
        schema_name = payload.get("schema_name")
        if not isinstance(schema_name, str):
            continue
        if schema_name not in catalog:
            errors.append(f"{golden.name}: unknown schema_name {schema_name}")
            continue
        try:
            validate(schema_name, payload)
        except Exception as exc:
            errors.append(f"{golden.name}: {exc}")
    return (0 if not errors else 1), sorted(errors)
