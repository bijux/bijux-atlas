#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
SSOT = ROOT / "configs" / "make" / "public-targets.json"
OWNERSHIP = ROOT / "configs" / "make" / "ownership.json"
ALLOWED_AREAS = {"cargo", "docs", "ops", "scripts", "configs", "policies"}


def load_ssot() -> dict:
    data = json.loads(SSOT.read_text(encoding="utf-8"))
    targets = data.get("public_targets")
    if not isinstance(targets, list):
        raise SystemExit("configs/make/public-targets.json: public_targets must be a list")
    names = [entry.get("name") for entry in targets if isinstance(entry, dict)]
    if len(names) != len(set(names)):
        raise SystemExit("configs/make/public-targets.json: duplicate target names")
    return data


def public_entries() -> list[dict]:
    return load_ssot()["public_targets"]


def public_names() -> list[str]:
    return [entry["name"] for entry in public_entries()]


def entry_map() -> dict[str, dict]:
    return {entry["name"]: entry for entry in public_entries()}


def load_ownership() -> dict[str, dict]:
    payload = json.loads(OWNERSHIP.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        return {}
    merged: dict[str, dict] = {k: v for k, v in payload.items() if isinstance(v, dict) and k != "targets"}
    nested = payload.get("targets")
    if isinstance(nested, dict):
        for k, v in nested.items():
            if isinstance(v, dict):
                merged[k] = v
    return merged
