from __future__ import annotations

import json
from pathlib import Path


def read_tool_versions(repo_root: Path) -> dict[str, object]:
    path = repo_root / "configs/ops/tool-versions.json"
    return json.loads(path.read_text(encoding="utf-8"))


def read_pins(repo_root: Path) -> dict[str, object]:
    aggregate_path = repo_root / "configs/ops/pins.json"
    if aggregate_path.exists():
        return json.loads(aggregate_path.read_text(encoding="utf-8"))

    pins_dir = repo_root / "configs/ops/pins"
    payload: dict[str, object] = {"schema_version": 1, "sources": {}}
    if not pins_dir.exists():
        return payload
    for p in sorted(pins_dir.glob("*.json")):
        payload["sources"][p.name] = json.loads(p.read_text(encoding="utf-8"))
    return payload
