from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from ..core.context import RunContext
from ..core.fs import ensure_evidence_path


def write_json_report(ctx: RunContext, relative_path: str, payload: dict[str, Any]) -> Path:
    target = ensure_evidence_path(ctx, ctx.evidence_root / relative_path)
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return target
