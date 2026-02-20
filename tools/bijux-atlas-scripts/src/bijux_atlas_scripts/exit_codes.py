from __future__ import annotations

import json
from pathlib import Path

OPS_ERROR_REGISTRY = Path(__file__).resolve().parents[4] / "ops" / "_meta" / "error-registry.json"


def _load_registry() -> dict[str, int]:
    payload = json.loads(OPS_ERROR_REGISTRY.read_text(encoding="utf-8"))
    mapping: dict[str, int] = {}
    for row in payload.get("codes", []):
        mapping[str(row["name"])] = int(row["code"])
    return mapping


_REG = _load_registry()

OK = 0
ERR_CONFIG = _REG["OPS_ERR_CONFIG"]
ERR_CONTEXT = _REG["OPS_ERR_CONTEXT"]
ERR_VERSION = _REG["OPS_ERR_VERSION"]
ERR_PREREQ = _REG["OPS_ERR_PREREQ"]
ERR_TIMEOUT = _REG["OPS_ERR_TIMEOUT"]
ERR_VALIDATION = _REG["OPS_ERR_VALIDATION"]
ERR_ARTIFACT = _REG["OPS_ERR_ARTIFACT"]
ERR_DOCS = _REG["OPS_ERR_DOCS"]
ERR_INTERNAL = _REG["OPS_ERR_INTERNAL"]
