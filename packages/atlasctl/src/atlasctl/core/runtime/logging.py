from __future__ import annotations

import inspect
import json
import sys
from typing import TYPE_CHECKING

from .guards.clock import utc_now_iso

if TYPE_CHECKING:
    from ..context import RunContext


def log_event(ctx: RunContext, level: str, component: str, action: str, **fields: object) -> None:
    caller = inspect.stack()[1]
    payload = {
        "ts": utc_now_iso(),
        "level": level,
        "run_id": ctx.run_id,
        "component": component,
        "action": action,
        "file": caller.filename,
        "line": caller.lineno,
        **fields,
    }
    if ctx.log_json:
        sys.stderr.write(json.dumps(payload, sort_keys=True) + "\n")
        return
    core = f"ts={payload['ts']} level={level} run_id={ctx.run_id} component={component} action={action}"
    extras = " ".join(f"{key}={value}" for key, value in sorted(fields.items()))
    sys.stderr.write((core if not extras else f"{core} {extras}") + "\n")
