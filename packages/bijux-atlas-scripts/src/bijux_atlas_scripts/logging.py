from __future__ import annotations

import json
from datetime import datetime, timezone


def log_event(level: str, component: str, action: str, run_id: str, json_output: bool = True, **fields: object) -> None:
    payload: dict[str, object] = {
        "ts": datetime.now(timezone.utc).isoformat(),
        "level": level,
        "component": component,
        "action": action,
        "run_id": run_id,
    }
    payload.update(fields)
    if json_output:
        print(json.dumps(payload, sort_keys=True))
    else:
        extras = " ".join(f"{k}={v}" for k, v in fields.items())
        print(f"[{payload['level']}] {payload['component']}:{payload['action']} run_id={run_id} {extras}".strip())
