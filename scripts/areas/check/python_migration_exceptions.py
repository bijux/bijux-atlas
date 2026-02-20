#!/usr/bin/env python3
from __future__ import annotations

import json
from dataclasses import dataclass
from datetime import date
from fnmatch import fnmatch
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
EXCEPTIONS_PATH = ROOT / "configs" / "layout" / "python-migration-exceptions.json"


@dataclass(frozen=True)
class ExceptionEntry:
    id: str
    kind: str
    path_glob: str
    contains: str
    owner: str
    issue: str
    expires_on: date


def load_exceptions() -> list[ExceptionEntry]:
    payload = json.loads(EXCEPTIONS_PATH.read_text(encoding="utf-8"))
    items: list[ExceptionEntry] = []
    for row in payload.get("exceptions", []):
        items.append(
            ExceptionEntry(
                id=str(row["id"]),
                kind=str(row["kind"]),
                path_glob=str(row["path_glob"]),
                contains=str(row["contains"]),
                owner=str(row["owner"]),
                issue=str(row["issue"]),
                expires_on=date.fromisoformat(str(row["expires_on"])),
            )
        )
    return items


def find_matching_exception(kind: str, rel_path: str, line: str) -> ExceptionEntry | None:
    for entry in load_exceptions():
        if entry.kind != kind:
            continue
        if not fnmatch(rel_path, entry.path_glob):
            continue
        if entry.contains and entry.contains not in line:
            continue
        return entry
    return None


def expired_exceptions() -> list[ExceptionEntry]:
    today = date.today()
    return [entry for entry in load_exceptions() if entry.expires_on < today]
