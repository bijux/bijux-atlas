#!/usr/bin/env python3
# owner: platform
# purpose: print canonical merged config payload for debugging.
# stability: public
# called-by: make config-print
# Purpose: print canonical merged config payload for debugging.
# Inputs: configs/policy/policy.json, configs/ops/env.schema.json, configs/ops/tool-versions.json, configs/perf/thresholds.json, configs/slo/slo.json
# Outputs: json to stdout
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read_json(path: str) -> dict:
    return json.loads((ROOT / path).read_text(encoding="utf-8"))


def main() -> int:
    merged = {
        "policy": read_json("configs/policy/policy.json"),
        "ops_env_schema": read_json("configs/ops/env.schema.json"),
        "ops_tool_versions": read_json("configs/ops/tool-versions.json"),
        "perf_thresholds": read_json("configs/perf/thresholds.json"),
        "slo": read_json("configs/slo/slo.json"),
    }
    print(json.dumps(merged, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
