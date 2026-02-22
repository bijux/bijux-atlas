from __future__ import annotations


def split_marker_values(raw_values: list[str] | None) -> set[str]:
    values: set[str] = set()
    for raw in raw_values or []:
        for part in str(raw).split(","):
            marker = part.strip()
            if marker:
                values.add(marker)
    return values


def split_group_values(raw: list[str] | None) -> set[str]:
    values: set[str] = set()
    for item in raw or []:
        for part in str(item).split(","):
            value = part.strip()
            if value:
                values.add(value)
    return values
