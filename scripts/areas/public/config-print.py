#!/usr/bin/env python3
# owner: platform
# purpose: print canonical merged config payload for debugging.
# stability: public
# called-by: make config-print
# Purpose: print canonical merged config payload for debugging.
# Inputs: configs/policy/policy.json, configs/ops/env.schema.json, configs/ops/tool-versions.json, configs/ops/observability-pack.json, configs/perf/k6-thresholds.v1.json, configs/slo/slo.json
# Outputs: json to stdout
from __future__ import annotations

import json
import hashlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def read_json(path: str) -> dict:
    return json.loads((ROOT / path).read_text(encoding="utf-8"))


def main() -> int:
    sources = [
        "configs/policy/policy.json",
        "configs/ops/env.schema.json",
        "configs/ops/tool-versions.json",
        "configs/ops/observability-pack.json",
        "configs/perf/k6-thresholds.v1.json",
        "configs/slo/slo.json",
    ]
    provenance = []
    for src in sources:
        raw = (ROOT / src).read_bytes()
        provenance.append(
            {
                "path": src,
                "sha256": hashlib.sha256(raw).hexdigest(),
            }
        )
    merged = {
        "policy": read_json("configs/policy/policy.json"),
        "ops_env_schema": read_json("configs/ops/env.schema.json"),
        "ops_tool_versions": read_json("configs/ops/tool-versions.json"),
        "ops_observability_pack": read_json("configs/ops/observability-pack.json"),
        "perf_thresholds": read_json("configs/perf/k6-thresholds.v1.json"),
        "slo": read_json("configs/slo/slo.json"),
        "_provenance": provenance,
    }
    print(json.dumps(merged, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
