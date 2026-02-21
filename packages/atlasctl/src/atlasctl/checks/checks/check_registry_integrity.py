from __future__ import annotations

from pathlib import Path

def check_registry_integrity(repo_root: Path) -> tuple[int, list[str]]:
    from ..registry.ssot import generate_registry_json

    try:
        _out, changed = generate_registry_json(repo_root, check_only=True)
    except Exception as exc:
        return 1, [f"checks registry integrity failed: {exc}"]
    if changed:
        return 1, ["checks registry generated json drift detected; run `./bin/atlasctl gen checks-registry`"]
    return 0, []
