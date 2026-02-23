from __future__ import annotations

import json
import socket
from pathlib import Path
from typing import Any

from atlasctl.core.context import RunContext
from atlasctl.core.fs import ensure_evidence_path
from atlasctl.core.runtime.paths import write_text_file
from atlasctl.core.schema.schema_utils import validate_json


def _artifact_base(ctx: RunContext, area: str) -> Path:
    return ensure_evidence_path(ctx, ctx.evidence_root / area / ctx.run_id)


def _emit(payload: dict[str, Any], report_format: str) -> None:
    if report_format == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        print(
            f"{payload['area']}:{payload['action']} status={payload['status']} run_id={payload['run_id']}"
        )


def ports_show(ctx: RunContext, report_format: str) -> int:
    ports_cfg = json.loads((ctx.repo_root / "configs/ops/ports.json").read_text(encoding="utf-8"))
    payload: dict[str, Any] = {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "status": "pass",
        "run_id": ctx.run_id,
        "area": "ports",
        "action": "show",
        "details": ports_cfg,
    }
    _emit(payload, report_format)
    return 0


def _reserve_ephemeral_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("", 0))
        s.listen(1)
        return int(s.getsockname()[1])


def ports_reserve(ctx: RunContext, report_format: str, name: str, port: int | None) -> int:
    chosen = int(port) if port is not None else _reserve_ephemeral_port()
    out_dir = _artifact_base(ctx, "ports")
    reservation = {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "status": "pass",
        "run_id": ctx.run_id,
        "area": "ports",
        "action": "reserve",
        "details": {"name": name, "port": chosen},
    }
    validate_json(reservation, ctx.repo_root / "configs/contracts/scripts-tool-output.schema.json")
    write_text_file(out_dir / "report.json", json.dumps(reservation, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    write_text_file(out_dir / "run.log", f"reserved {name}={chosen}\n", encoding="utf-8")
    _emit(reservation, report_format)
    return 0
