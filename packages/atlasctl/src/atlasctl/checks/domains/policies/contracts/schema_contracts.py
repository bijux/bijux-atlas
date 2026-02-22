from __future__ import annotations

import json
import re
from pathlib import Path

from .....contracts.catalog import (
    lint_catalog,
    load_catalog,
    check_schema_readme_sync,
    check_schema_change_release_policy,
)
from .....contracts.validate import validate


def check_schema_catalog_integrity(repo_root: Path) -> tuple[int, list[str]]:
    errors = lint_catalog()
    return (0 if not errors else 1), sorted(errors)


def check_schema_catalog_files_exist(repo_root: Path) -> tuple[int, list[str]]:
    errors = [err for err in lint_catalog() if "missing schema file" in err]
    return (0 if not errors else 1), sorted(errors)


def check_schema_disk_files_listed(repo_root: Path) -> tuple[int, list[str]]:
    errors = [err for err in lint_catalog() if "schema files not in catalog" in err]
    return (0 if not errors else 1), sorted(errors)


def check_schema_catalog_sorted(repo_root: Path) -> tuple[int, list[str]]:
    errors = [err for err in lint_catalog() if "order must be sorted" in err]
    return (0 if not errors else 1), sorted(errors)


def check_schema_id_naming(repo_root: Path) -> tuple[int, list[str]]:
    errors = [err for err in lint_catalog() if "invalid schema id format" in err or "version mismatch" in err]
    return (0 if not errors else 1), sorted(errors)


def check_schema_samples_validate(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    samples = sorted((repo_root / "packages/atlasctl/tests/goldens/samples").glob("*.json"))
    if not samples:
        return 1, ["no sample payloads found under packages/atlasctl/tests/goldens/samples"]
    catalog = load_catalog()
    covered: set[str] = set()
    for sample in samples:
        payload = json.loads(sample.read_text(encoding="utf-8"))
        schema_name = payload.get("schema_name")
        if not isinstance(schema_name, str):
            errors.append(f"{sample.name}: missing schema_name")
            continue
        if schema_name not in catalog:
            errors.append(f"{sample.name}: unknown schema_name {schema_name}")
            continue
        covered.add(schema_name)
        expected_version = int(catalog[schema_name].version)
        if int(payload.get("schema_version", -1)) != expected_version:
            errors.append(f"{sample.name}: schema_version mismatch for {schema_name} (expected {expected_version})")
            continue
        try:
            validate(schema_name, payload)
        except Exception as exc:
            errors.append(f"{sample.name}: {exc}")
    missing = sorted(set(catalog.keys()) - covered)
    for schema_name in missing:
        errors.append(f"missing sample payload for schema {schema_name}")
    return (0 if not errors else 1), sorted(errors)


def check_schema_catalog_referenced(repo_root: Path) -> tuple[int, list[str]]:
    catalog = load_catalog()
    referenced: set[str] = set()
    pattern = re.compile(r"atlasctl\.[a-z0-9.-]+\.v\d+")
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


def check_schema_catalog_ssot(repo_root: Path) -> tuple[int, list[str]]:
    src_root = repo_root / "packages/atlasctl/src/atlasctl"
    if not src_root.exists():
        return 1, ["missing atlasctl src root"]
    allow_rel = {
        "contracts/catalog.py",
        "contracts/schemas/README.md",
    }
    errors: list[str] = []
    for path in sorted(src_root.rglob("*")):
        if not path.is_file() or path.suffix not in {".py", ".md"}:
            continue
        rel = path.relative_to(src_root).as_posix()
        if rel in allow_rel:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if "catalog.json" in text:
            errors.append(
                f"schema catalog SSOT violation: {rel} references catalog.json directly; use contracts.catalog APIs"
            )
    return (0 if not errors else 1), sorted(errors)


def check_schema_readme_updated(repo_root: Path) -> tuple[int, list[str]]:
    errors = check_schema_readme_sync()
    return (0 if not errors else 1), sorted(errors)


def check_schema_change_release_notes(repo_root: Path) -> tuple[int, list[str]]:
    errors = check_schema_change_release_policy(repo_root)
    return (0 if not errors else 1), sorted(errors)
