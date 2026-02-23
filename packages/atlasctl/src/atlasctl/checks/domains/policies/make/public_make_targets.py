from __future__ import annotations

import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[8]
SSOT = ROOT / "configs" / "make" / "public-targets.json"
OWNERSHIP = ROOT / "configs" / "make" / "ownership.json"
ALLOWED_AREAS = {"cargo", "docs", "ops", "scripts", "configs", "policies", "product", "make"}


def _load_json(path: Path) -> dict:
    payload = json.loads(path.read_text(encoding="utf-8"))
    return payload if isinstance(payload, dict) else {}


def public_entries() -> list[dict]:
    payload = _load_json(SSOT)
    rows = payload.get("public_targets", [])
    return [row for row in rows if isinstance(row, dict)]


def public_names() -> list[str]:
    return [str(entry.get("name", "")).strip() for entry in public_entries() if str(entry.get("name", "")).strip()]


def entry_map() -> dict[str, dict]:
    return {name: entry for entry in public_entries() if (name := str(entry.get("name", "")).strip())}


def load_ownership() -> dict[str, dict]:
    payload = _load_json(OWNERSHIP)
    merged: dict[str, dict] = {}
    for key, value in payload.items():
        if key == "targets":
            continue
        if isinstance(value, dict):
            merged[key] = value
    nested = payload.get("targets")
    if isinstance(nested, dict):
        for key, value in nested.items():
            if isinstance(value, dict):
                merged[key] = value
    return merged


__all__ = ["ALLOWED_AREAS", "public_names", "entry_map", "load_ownership"]
