from __future__ import annotations

import json
import re
import hashlib
from dataclasses import dataclass
from pathlib import Path
import subprocess

from ...core.errors import ScriptError
from ...core.exit_codes import ERR_VALIDATION
from .schemas import schemas_root


@dataclass(frozen=True)
class CatalogEntry:
    name: str
    version: int
    file: str


def catalog_path() -> Path:
    return schemas_root() / "catalog.json"


_SCHEMA_ID_RE = re.compile(r"^atlasctl\.[a-z0-9][a-z0-9._-]*\.v[1-9][0-9]*$")
_SCHEMA_FILE_RE = re.compile(r"^(atlasctl\.[a-z0-9][a-z0-9._-]*\.v([1-9][0-9]*))\.schema\.json$")


def _raw_catalog() -> dict[str, object]:
    return json.loads(catalog_path().read_text(encoding="utf-8"))


def _schema_files_on_disk() -> list[Path]:
    return sorted(path for path in schemas_root().glob("*.schema.json") if path.is_file())


def deterministic_catalog_payload() -> dict[str, object]:
    rows: list[dict[str, object]] = []
    for path in _schema_files_on_disk():
        match = _SCHEMA_FILE_RE.match(path.name)
        if not match:
            continue
        schema_name = match.group(1)
        version = int(match.group(2))
        rows.append({"name": schema_name, "version": version, "file": path.name})
    rows.sort(key=lambda row: (str(row["name"]), int(row["version"]), str(row["file"])))
    return {"schemas": rows}


def write_catalog_deterministic() -> Path:
    payload = deterministic_catalog_payload()
    out = catalog_path()
    out.write_text(json.dumps(payload, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    return out


def deterministic_schema_readme() -> str:
    lines = [
        "# Schema Catalog",
        "",
        "Generated from `packages/atlasctl/src/atlasctl/contracts/schema/schemas/*.schema.json`.",
        "Do not edit rows manually; regenerate via `atlasctl contracts generate --generators catalog`.",
        "",
        "| schema_name | version | file | sha256_16 |",
        "|---|---:|---|---|",
    ]
    for row in deterministic_catalog_payload()["schemas"]:  # type: ignore[index]
        file_name = str(row["file"])
        digest = hashlib.sha256((schemas_root() / file_name).read_bytes()).hexdigest()[:16]
        lines.append(f"| {row['name']} | {row['version']} | {file_name} | `{digest}` |")
    lines.append("")
    return "\n".join(lines)


def write_schema_readme_deterministic() -> Path:
    out = schemas_root() / "README.md"
    out.write_text(deterministic_schema_readme(), encoding="utf-8")
    return out


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

    canonical = deterministic_catalog_payload()
    current = _raw_catalog()
    if current != canonical:
        errors.append("schema catalog must match deterministic generated payload; run `atlasctl contracts generate --generators catalog`")
    return sorted(errors)


def check_schema_readme_sync() -> list[str]:
    readme = schemas_root() / "README.md"
    expected = deterministic_schema_readme().strip()
    if not readme.exists():
        return [f"missing schemas README: {readme.as_posix()}"]
    actual = readme.read_text(encoding="utf-8").strip()
    if actual != expected:
        return [
            "schema README drift: update packages/atlasctl/src/atlasctl/contracts/schema/schemas/README.md",
            "hint: run `atlasctl contracts generate --generators catalog`",
        ]
    return []


def _git_changed_files(repo_root: Path) -> list[str]:
    proc = subprocess.run(
        ["git", "diff", "--name-only", "HEAD~1", "HEAD"],
        cwd=repo_root,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        return []
    return [line.strip() for line in proc.stdout.splitlines() if line.strip()]


def check_schema_change_release_policy(repo_root: Path) -> list[str]:
    changed = _git_changed_files(repo_root)
    schema_prefix = "packages/atlasctl/src/atlasctl/contracts/schema/schemas/"
    changed_schemas = sorted(
        path for path in changed if path.startswith(schema_prefix) and path.endswith(".schema.json")
    )
    if not changed_schemas:
        return []

    errors: list[str] = []
    release_notes = "packages/atlasctl/docs/release-notes.md"
    if release_notes not in changed:
        errors.append("schema change requires release notes update in packages/atlasctl/docs/release-notes.md")

    for rel in changed_schemas:
        existed = subprocess.run(
            ["git", "cat-file", "-e", f"HEAD~1:{rel}"],
            cwd=repo_root,
            text=True,
            capture_output=True,
            check=False,
        ).returncode == 0
        if existed:
            errors.append(
                f"schema bump policy violation: modified existing schema file `{rel}`; add new versioned schema instead"
            )
    return sorted(set(errors))


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
