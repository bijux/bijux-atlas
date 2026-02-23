from __future__ import annotations

import json
import sys
from typing import Any


def emit_ops_payload(payload: dict[str, Any], report_format: str, *, compact_json: bool = True) -> None:
    if report_format == "json":
        if compact_json:
            sys.stdout.write(json.dumps(payload, sort_keys=True) + "\n")
        else:
            sys.stdout.write(json.dumps(payload, indent=2, sort_keys=True) + "\n")
        return
    area = str(payload.get("area", "ops"))
    action = str(payload.get("action", ""))
    status = str(payload.get("status", "unknown"))
    run_id = str(payload.get("run_id", ""))
    sys.stdout.write(f"{area}:{action} status={status} run_id={run_id}\n")
