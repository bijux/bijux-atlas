from __future__ import annotations

import json
from datetime import datetime, timezone

from .context import RunContext


def emit_telemetry(ctx: RunContext, event: str, **fields: object) -> None:
    path = ctx.repo_root / "artifacts" / "isolate" / ctx.run_id / "atlasctl-telemetry" / "events.jsonl"
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "ts": datetime.now(timezone.utc).isoformat(),
        "event": event,
        "run_id": ctx.run_id,
        "profile": ctx.profile,
        **fields,
    }
    with path.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(payload, sort_keys=True) + "\n")
