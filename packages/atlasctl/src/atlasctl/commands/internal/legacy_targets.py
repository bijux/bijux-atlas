from __future__ import annotations

import json
from datetime import date

from ...core.context import RunContext


_LEGACY_TARGETS: tuple[dict[str, str], ...] = (
    {
        "target": "internal/dev/fmt",
        "replacement": "make fmt",
        "expires_on": "2026-06-01",
    },
    {
        "target": "internal/dev/lint",
        "replacement": "make lint",
        "expires_on": "2026-06-01",
    },
    {
        "target": "internal/dev/test",
        "replacement": "make test",
        "expires_on": "2026-06-01",
    },
    {
        "target": "internal/dev/audit",
        "replacement": "make audit",
        "expires_on": "2026-06-01",
    },
)


def run_legacy_targets(ctx: RunContext, report: str) -> int:
    today = date.today()
    rows: list[dict[str, object]] = []
    for row in _LEGACY_TARGETS:
        expires_on = date.fromisoformat(row["expires_on"])
        rows.append(
            {
                **row,
                "expired": expires_on < today,
                "days_to_expiry": (expires_on - today).days,
            }
        )
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok",
        "kind": "legacy-targets",
        "run_id": ctx.run_id,
        "targets": rows,
    }
    if report == "json" or ctx.output_format == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        for row in rows:
            mark = "EXPIRED" if row["expired"] else f"{row['days_to_expiry']}d"
            print(f"{row['target']} -> {row['replacement']} (expires {row['expires_on']} | {mark})")
    return 0

