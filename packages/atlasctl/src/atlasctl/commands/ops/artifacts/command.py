from __future__ import annotations

import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from atlasctl.core.context import RunContext
from atlasctl.core.process import run_command
from atlasctl.contracts.validate import validate as validate_contract
from atlasctl.commands.ops._shared.output import emit_ops_payload


def artifacts_open(ctx: RunContext, report_format: str) -> int:
    root = ctx.repo_root / "artifacts" / "ops"
    latest = ""
    if root.exists():
        dirs = sorted([p for p in root.iterdir() if p.is_dir()])
        if dirs:
            latest = str(dirs[-1].relative_to(ctx.repo_root))
    if not latest:
        payload = {
            "schema_version": 1,
            "tool": "bijux-atlas",
            "status": "fail",
            "run_id": ctx.run_id,
            "area": "artifacts",
            "action": "open",
            "details": "no artifacts found under artifacts/ops",
        }
        emit_ops_payload(payload, report_format)
        return 2
    target = ctx.repo_root / latest
    for opener in (["open", str(target)], ["xdg-open", str(target)]):
        try:
            run_command(opener, ctx.repo_root, ctx=ctx)
        except FileNotFoundError:
            continue
    payload = {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "status": "pass",
        "run_id": ctx.run_id,
        "area": "artifacts",
        "action": "open",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "details": {"path": latest},
    }
    emit_ops_payload(payload, report_format)
    return 0


def cleanup_gc(ctx: RunContext, report_format: str, older_than_days: int) -> int:
    now = datetime.now(timezone.utc).timestamp()
    root = ctx.evidence_root
    removed: list[str] = []
    if root.exists():
        for path in sorted(root.rglob("*")):
            if not path.is_dir():
                continue
            age_days = (now - path.stat().st_mtime) / 86400.0
            if age_days >= float(older_than_days):
                removed.append(str(path))
    for item in sorted(removed, key=lambda p: p.count("/"), reverse=True):
        p = Path(item)
        try:
            p.rmdir()
        except OSError:
            continue
    payload = {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "status": "pass",
        "run_id": ctx.run_id,
        "area": "cleanup",
        "action": "gc",
        "details": {"older_than_days": older_than_days, "removed_dirs": sorted(removed)},
    }
    validate_contract("scripts-tool-output", payload)
    emit_ops_payload(payload, report_format)
    return 0


def configure_artifacts_command(*_args: object, **_kwargs: object) -> None:
    """CLI intent marker; parser wiring lives in ops root parser."""
    return None


def run_artifacts_command_cli(ctx: RunContext, report_format: str, older_than_days: int) -> int:
    return cleanup_gc(ctx, report_format, older_than_days)
