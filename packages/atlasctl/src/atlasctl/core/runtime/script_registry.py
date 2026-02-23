from __future__ import annotations

import json
import re
from dataclasses import dataclass
from pathlib import Path

from ..context import RunContext
from ..fs import ensure_evidence_path
from ..meta.owners import load_owner_catalog
from .paths import write_text_file

SCRIPT_REGISTRY_PATH = Path("configs/scripts/registry.json")
COMMANDS_ROOT = Path("packages/atlasctl/src/atlasctl/commands")
_SCRIPT_REF_RE = re.compile(
    r"(?:packages/atlasctl/src/atlasctl/commands/[A-Za-z0-9_./-]+\.sh|(?:ops|scripts|docker/scripts)/[A-Za-z0-9_./-]+\.sh)"
)
_NAME_RE = re.compile(r"^[a-z0-9][a-z0-9._/-]*\.sh$")


@dataclass(frozen=True)
class ScriptEntry:
    path: str
    owner: str
    description: str


def _load_payload(repo_root: Path) -> dict[str, object]:
    registry_path = repo_root / SCRIPT_REGISTRY_PATH
    if not registry_path.exists():
        raise ValueError(f"missing script registry: {SCRIPT_REGISTRY_PATH.as_posix()}")
    payload = json.loads(registry_path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError("script registry must be a JSON object")
    return payload


def load_script_registry(repo_root: Path) -> tuple[ScriptEntry, ...]:
    payload = _load_payload(repo_root)
    rows = payload.get("scripts", [])
    if not isinstance(rows, list):
        raise ValueError("script registry must define `scripts` list")
    entries: list[ScriptEntry] = []
    for row in rows:
        if not isinstance(row, dict):
            raise ValueError("script registry entries must be objects")
        path = str(row.get("path", "")).strip()
        owner = str(row.get("owner", "")).strip()
        description = str(row.get("description", "")).strip()
        if not path:
            raise ValueError("script registry entry missing `path`")
        entries.append(ScriptEntry(path=path, owner=owner, description=description))
    return tuple(entries)


def collect_command_script_refs(repo_root: Path) -> tuple[str, ...]:
    refs: set[str] = set()
    root = repo_root / COMMANDS_ROOT
    for py_file in sorted(root.rglob("*.py")):
        rel = py_file.relative_to(repo_root).as_posix()
        text = py_file.read_text(encoding="utf-8", errors="ignore")
        for match in _SCRIPT_REF_RE.findall(text):
            refs.add(match.strip())
        if rel.endswith("runtime.py"):
            # runtime.py uses exec() to stitch runtime modules; exclude it from literal scanning noise.
            continue
    return tuple(sorted(refs))


def lint_script_registry(repo_root: Path) -> tuple[int, dict[str, object]]:
    errors: list[str] = []
    entries = load_script_registry(repo_root)
    valid_owners = set(load_owner_catalog(repo_root).owners)
    seen_paths: set[str] = set()
    for entry in entries:
        if entry.path in seen_paths:
            errors.append(f"duplicate script path in registry: {entry.path}")
        seen_paths.add(entry.path)
        if not _NAME_RE.match(entry.path):
            errors.append(f"script name violates policy (lowercase path required): {entry.path}")
        if entry.owner not in valid_owners:
            errors.append(f"script `{entry.path}` references unknown owner `{entry.owner}`")
        script_path = repo_root / entry.path
        if not script_path.exists():
            errors.append(f"registered script path does not exist: {entry.path}")
        elif not script_path.is_file():
            errors.append(f"registered script path must be a file: {entry.path}")
    refs = collect_command_script_refs(repo_root)
    unregistered = [ref for ref in refs if ref not in seen_paths]
    for ref in unregistered:
        errors.append(f"command invokes non-registered script: {ref}")
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "kind": "script-registry-lint",
        "status": "pass" if not errors else "fail",
        "registry_path": SCRIPT_REGISTRY_PATH.as_posix(),
        "registered_count": len(entries),
        "referenced_count": len(refs),
        "errors": errors,
    }
    return (0 if not errors else 1), payload


def emit_script_registry_evidence(ctx: RunContext, payload: dict[str, object]) -> Path:
    out = ensure_evidence_path(
        ctx,
        ctx.evidence_root / "lint" / "scripts" / ctx.run_id / "report.json",
    )
    write_text_file(out, json.dumps(payload, indent=2, sort_keys=True) + "\n")
    return out
