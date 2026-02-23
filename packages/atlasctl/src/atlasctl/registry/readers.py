from __future__ import annotations

import json
from pathlib import Path
from typing import Any


def load_json_catalog(repo_root: Path, rel_path: str) -> dict[str, Any]:
    path = repo_root / rel_path
    return json.loads(path.read_text(encoding="utf-8"))


def load_ops_tasks_catalog(repo_root: Path) -> dict[str, dict[str, str]]:
    payload = load_json_catalog(repo_root, "packages/atlasctl/src/atlasctl/registry/ops_tasks_catalog.json")
    rows = payload.get("tasks", []) if isinstance(payload, dict) else []
    out: dict[str, dict[str, str]] = {}
    for row in rows:
        if not isinstance(row, dict):
            continue
        name = str(row.get("name", "")).strip()
        if not name:
            continue
        out[name] = {
            "manifest": str(row.get("manifest", "")).strip(),
            "owner": str(row.get("owner", "")).strip(),
            "docs": str(row.get("docs", "")).strip(),
            "description": str(row.get("description", "")).strip(),
        }
    return out
