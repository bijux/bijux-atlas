from __future__ import annotations

import json
import shutil
from datetime import datetime, timedelta, timezone
from pathlib import Path

from ..core.context import RunContext


def _cmd_artifact_index(ctx: RunContext, limit: int, out: str | None) -> int:
    root = (ctx.repo_root / "artifacts/atlasctl/run").resolve()
    rows: list[dict[str, object]] = []
    if root.exists():
        candidates = sorted([path for path in root.iterdir() if path.is_dir()], key=lambda path: path.stat().st_mtime, reverse=True)
        for run in candidates[:limit]:
            rows.append({
                "run_id": run.name,
                "path": str(run),
                "reports": sorted(str(path.relative_to(ctx.repo_root)) for path in run.glob("reports/*.json")),
                "logs": sorted(str(path.relative_to(ctx.repo_root)) for path in run.glob("logs/*.log")),
            })
    payload = {"schema_version": 1, "tool": "bijux-atlas", "artifact_runs": rows}
    if out:
        out_path = Path(out)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(out_path)
    else:
        print(json.dumps(payload, indent=2, sort_keys=True))
    return 0


def _cmd_artifact_gc(ctx: RunContext, older_than_days: int | None) -> int:
    cfg = ctx.repo_root / "configs/ops/scripts-artifact-retention.json"
    payload = {"scripts_retention_days": 14}
    if cfg.exists():
        payload = json.loads(cfg.read_text(encoding="utf-8"))
    days = int(payload.get("scripts_retention_days", 14)) if older_than_days is None else int(older_than_days)
    root = (ctx.repo_root / "artifacts/atlasctl/run").resolve()
    removed: list[str] = []
    if root.exists():
        cutoff = datetime.now(timezone.utc) - timedelta(days=days)
        for run in sorted([path for path in root.iterdir() if path.is_dir()]):
            modified = datetime.fromtimestamp(run.stat().st_mtime, tz=timezone.utc)
            if modified < cutoff:
                shutil.rmtree(run, ignore_errors=True)
                removed.append(str(run))
    print(json.dumps({"schema_version": 1, "tool": "bijux-atlas", "removed": sorted(removed), "retention_days": days}, sort_keys=True))
    return 0
