#!/usr/bin/env python3
# Purpose: generate Helm values.schema.json from CHART_VALUES.json SSOT top-level keys.
# Inputs: docs/contracts/CHART_VALUES.json
# Outputs: ops/k8s/charts/bijux-atlas/values.schema.json
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
contract = json.loads((ROOT / "docs/contracts/CHART_VALUES.json").read_text())
keys = contract["top_level_keys"]

schema = {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "title": "bijux-atlas chart values",
    "type": "object",
    "additionalProperties": False,
    "properties": {k: {"description": f"Chart values key `{k}` from SSOT contract."} for k in keys},
}

# Enforce chart runtime safety contract for server resources and probe paths.
schema["properties"]["resources"] = {
    "type": "object",
    "additionalProperties": True,
    "required": ["requests", "limits"],
    "properties": {
        "requests": {
            "type": "object",
            "required": ["cpu", "memory"],
            "additionalProperties": True,
            "properties": {
                "cpu": {"type": "string", "minLength": 1},
                "memory": {"type": "string", "minLength": 1},
                "ephemeral-storage": {"type": "string", "minLength": 1},
            },
        },
        "limits": {
            "type": "object",
            "required": ["cpu", "memory"],
            "additionalProperties": True,
            "properties": {
                "cpu": {"type": "string", "minLength": 1},
                "memory": {"type": "string", "minLength": 1},
                "ephemeral-storage": {"type": "string", "minLength": 1},
            },
        },
    },
}
schema["properties"]["server"] = {
    "type": "object",
    "additionalProperties": True,
    "required": ["readinessProbePath", "startupProbePath"],
    "properties": {
        "readinessProbePath": {"type": "string", "minLength": 1},
        "startupProbePath": {"type": "string", "minLength": 1},
    },
}

out = ROOT / "ops/k8s/charts/bijux-atlas/values.schema.json"
out.write_text(json.dumps(schema, indent=2, sort_keys=True) + "\n")
print(f"wrote {out}")
