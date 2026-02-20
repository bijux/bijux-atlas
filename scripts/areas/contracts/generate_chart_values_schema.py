#!/usr/bin/env python3
# Purpose: generate Helm values.schema.json from CHART_VALUES.json SSOT top-level keys.
# Inputs: docs/contracts/CHART_VALUES.json
# Outputs: ops/k8s/charts/bijux-atlas/values.schema.json
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
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
    "required": ["configSchemaVersion", "readinessProbePath", "startupProbePath"],
    "properties": {
        "configSchemaVersion": {"type": "string", "minLength": 1},
        "readinessProbePath": {"type": "string", "minLength": 1},
        "startupProbePath": {"type": "string", "minLength": 1},
    },
}
schema["properties"]["hpa"] = {
    "type": "object",
    "additionalProperties": True,
    "required": ["enabled", "minReplicas", "maxReplicas", "cpuUtilization", "behavior"],
    "properties": {
        "enabled": {"type": "boolean"},
        "minReplicas": {"type": "integer", "minimum": 1},
        "maxReplicas": {"type": "integer", "minimum": 1},
        "cpuUtilization": {"type": "integer", "minimum": 1, "maximum": 100},
        "behavior": {
            "type": "object",
            "additionalProperties": True,
            "required": ["scaleUp", "scaleDown"],
            "properties": {
                "scaleUp": {
                    "type": "object",
                    "additionalProperties": True,
                    "required": ["policies"],
                    "properties": {
                        "policies": {
                            "type": "array",
                            "minItems": 1,
                            "items": {"type": "object"},
                        }
                    },
                },
                "scaleDown": {
                    "type": "object",
                    "additionalProperties": True,
                    "required": ["policies"],
                    "properties": {
                        "policies": {
                            "type": "array",
                            "minItems": 1,
                            "items": {"type": "object"},
                        }
                    },
                },
            },
        },
    },
}
schema["properties"]["metrics"] = {
    "type": "object",
    "additionalProperties": True,
    "required": ["customMetrics"],
    "properties": {
        "customMetrics": {
            "type": "object",
            "additionalProperties": True,
            "required": ["enabled", "requiredPodAnnotations"],
            "properties": {
                "enabled": {"type": "boolean"},
                "requiredPodAnnotations": {
                    "type": "object",
                    "minProperties": 1,
                    "additionalProperties": {"type": "string"},
                },
            },
        }
    },
}
schema["allOf"] = [
    {
        "if": {
            "properties": {
                "hpa": {
                    "properties": {
                        "enabled": {"const": True},
                    }
                }
            },
            "required": ["hpa"],
        },
        "then": {
            "required": ["metrics"],
            "properties": {
                "metrics": {
                    "required": ["customMetrics"],
                    "properties": {
                        "customMetrics": {
                            "required": ["enabled"],
                            "properties": {
                                "enabled": {"const": True},
                            },
                        }
                    },
                }
            },
        },
    }
]

out = ROOT / "ops/k8s/charts/bijux-atlas/values.schema.json"
out.write_text(json.dumps(schema, indent=2, sort_keys=True) + "\n")
print(f"wrote {out}")
