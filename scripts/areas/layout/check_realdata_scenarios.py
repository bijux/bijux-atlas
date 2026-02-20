#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SCHEMA_PATH = ROOT / "ops/_schemas/e2e-realdata-scenarios.schema.json"
MANIFEST_PATH = ROOT / "ops/e2e/realdata/scenarios.json"
REALDATA_DIR = ROOT / "ops/e2e/realdata"


def _expect(cond: bool, msg: str, errors: list[str]) -> None:
    if not cond:
        errors.append(msg)


def main() -> int:
    schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    errors: list[str] = []

    for key in schema.get("required", []):
        _expect(key in manifest, f"missing required key in realdata scenarios manifest: {key}", errors)

    ids: set[str] = set()
    scripts_from_manifest: set[str] = set()
    for i, scenario in enumerate(manifest.get("scenarios", [])):
        sid = scenario.get("id")
        script = scenario.get("script")
        _expect(isinstance(sid, str) and sid, f"scenario[{i}] missing/invalid id", errors)
        _expect(isinstance(script, str) and script.endswith(".sh"), f"scenario[{i}] missing/invalid script", errors)
        if isinstance(sid, str):
            _expect(sid not in ids, f"duplicate realdata scenario id: {sid}", errors)
            ids.add(sid)
        if isinstance(script, str):
            scripts_from_manifest.add(script)
            _expect((REALDATA_DIR / script).exists(), f"realdata scenario script not found: {script}", errors)

    required = {"run_single_release.sh", "run_two_release_diff.sh", "schema_evolution.sh", "upgrade_drill.sh", "rollback_drill.sh"}
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
