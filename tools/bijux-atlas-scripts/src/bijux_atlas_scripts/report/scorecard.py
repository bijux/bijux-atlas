from __future__ import annotations


def summarize_status(items: list[dict[str, object]]) -> dict[str, int]:
    counts = {"pass": 0, "fail": 0, "unknown": 0}
    for item in items:
        status = str(item.get("status", "unknown"))
        if status not in counts:
            status = "unknown"
        counts[status] += 1
    return counts
