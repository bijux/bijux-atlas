#!/usr/bin/env python3
# owner: bijux-atlas-operations
# purpose: aggregate drill result artifacts into a single conformance report.
# stability: internal
# called-by: packages/atlasctl/src/atlasctl/commands/ops/observability/tests/test_drills.py
from __future__ import annotations
import json
from pathlib import Path

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


root = _repo_root()
results_dir = root / "artifacts/observability/drills"
out_dir = root / "artifacts/observability"
out_dir.mkdir(parents=True, exist_ok=True)

results = []
for path in sorted(results_dir.glob("*.result.json")):
    results.append(json.loads(path.read_text(encoding="utf-8")))

summary = {
    "schema_version": 1,
    "total": len(results),
    "passed": sum(1 for r in results if r.get("status") == "pass"),
    "failed": sum(1 for r in results if r.get("status") != "pass"),
    "results": [
        {
            "drill": r.get("drill"),
            "status": r.get("status"),
            "started_at": r.get("started_at"),
            "ended_at": r.get("ended_at"),
            "result": f"artifacts/observability/drills/{r.get('drill')}.result.json",
        }
        for r in results
    ],
}

out_path = out_dir / "drill-conformance-report.json"
out_path.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(out_path)
if summary["failed"]:
    raise SystemExit(1)
