from __future__ import annotations

import json
from pathlib import Path

from atlasctl.core.context import RunContext
from atlasctl.core.runtime.paths import write_text_file

from .artifacts import ops_evidence_dir


def write_ops_json_report(ctx: RunContext, area: str, filename: str, payload: dict[str, object]) -> Path:
    out_dir = ops_evidence_dir(ctx, area)
    out = out_dir / filename
    write_text_file(out, json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return out
