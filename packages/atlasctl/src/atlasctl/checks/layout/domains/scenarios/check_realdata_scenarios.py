#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("ops", "packages", "configs", "makefiles")):
            return parent
    raise RuntimeError("unable to resolve repo root")


ROOT = _repo_root()
SCHEMA_PATH = ROOT / "ops/_schemas/e2e-realdata-scenarios.schema.json"
MANIFEST_PATH = ROOT / "ops/e2e/realdata/scenarios.json"
SURFACE_PATH = ROOT / "ops/_meta/surface.json"


def _expect(cond: bool, msg: str, errors: list[str]) -> None:
    if not cond:
        errors.append(msg)


def main() -> int:
    schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    surface = json.loads(SURFACE_PATH.read_text(encoding="utf-8"))
    known_action_ids = {str(row.get("id")) for row in surface.get("actions", []) if isinstance(row, dict)}
    errors: list[str] = []

    for key in schema.get("required", []):
        _expect(key in manifest, f"missing required key in realdata scenarios manifest: {key}", errors)

    ids: set[str] = set()
    scripts_from_manifest: set[str] = set()
    for i, scenario in enumerate(manifest.get("scenarios", [])):
        sid = scenario.get("id")
        script = scenario.get("script")
        action_id = scenario.get("action_id")
        _expect(isinstance(sid, str) and sid, f"scenario[{i}] missing/invalid id", errors)
        _expect(isinstance(script, str) and script.endswith(".py"), f"scenario[{i}] missing/invalid script", errors)
        _expect(isinstance(action_id, str) and action_id in known_action_ids, f"scenario[{i}] action_id not found: {action_id!r}", errors)
        if isinstance(sid, str):
            _expect(sid not in ids, f"duplicate realdata scenario id: {sid}", errors)
            ids.add(sid)
        if isinstance(script, str):
            scripts_from_manifest.add(script)
            _expect((ROOT / script).exists(), f"realdata scenario script not found: {script}", errors)

    required = {
        "packages/atlasctl/src/atlasctl/commands/ops/e2e/realdata/run_single_release.py",
        "packages/atlasctl/src/atlasctl/commands/ops/e2e/realdata/run_two_release_diff.py",
        "packages/atlasctl/src/atlasctl/commands/ops/e2e/realdata/schema_evolution.py",
        "packages/atlasctl/src/atlasctl/commands/ops/e2e/realdata/upgrade_drill.py",
        "packages/atlasctl/src/atlasctl/commands/ops/e2e/realdata/rollback_drill.py",
    }
    for script in sorted(required):
        _expect(script in scripts_from_manifest, f"required realdata scenario missing from manifest: {script}", errors)

    if errors:
        print("realdata scenario contract failed:", file=sys.stderr)
        for e in errors:
            print(f"- {e}", file=sys.stderr)
        return 1

    print("realdata scenario contract passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
