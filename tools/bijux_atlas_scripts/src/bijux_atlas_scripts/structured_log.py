from __future__ import annotations

import json
import sys
from datetime import datetime, timezone

from .run_context import RunContext


def log_event(ctx: RunContext, level: str, event: str, **fields: object) -> None:
    payload = {
        "ts": datetime.now(timezone.utc).isoformat(),
        "level": level,
        "event": event,
        "run_id": ctx.run_id,
        "profile": ctx.profile,
        **fields,
    }
    sys.stderr.write(json.dumps(payload, sort_keys=True) + "\n")
