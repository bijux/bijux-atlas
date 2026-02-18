#!/usr/bin/env python3
# Purpose: ensure docs/api/v1-surface.md is generated from docs/contracts/ENDPOINTS.json.
# Inputs: docs/contracts/ENDPOINTS.json.
# Outputs: docs/api/v1-surface.md (with --write) or non-zero drift report.
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ENDPOINTS = ROOT / "docs" / "contracts" / "ENDPOINTS.json"
SURFACE = ROOT / "docs" / "api" / "v1-surface.md"


def render() -> str:
    c = json.loads(ENDPOINTS.read_text())
    lines = [
        "# V1 API Surface",
        "",
        "- Owner: `api`",
        "- Stability: `stable`",
        "",
        f"- Contract version: `{c.get('api_contract_version', 'v1')}`",
        "- Compatibility: additive-only within v1",
        "",
        "## Endpoints",
        "",
        "| Method | Path | Semantics |",
        "| --- | --- | --- |",
    ]
    semantics = {
        "health": "health/readiness signal",
        "metrics": "Prometheus metrics",
        "control": "version/control metadata",
        "catalog": "dataset catalog/metadata",
        "query": "gene query/search",
        "sequence": "sequence retrieval",
        "transcript": "transcript retrieval",
        "diff": "cross-release diff",
        "debug": "debug-only diagnostics",
    }
    for ep in c["endpoints"]:
        lines.append(
            f"| {ep['method']} | `{ep['path']}` | {semantics.get(ep.get('telemetry_class', 'control'), 'api endpoint')} |"
        )
    lines += [
        "",
        "## Source Of Truth",
        "",
        "- `docs/contracts/endpoints.md` is authoritative for paths/params/responses.",
        "- `configs/openapi/v1/openapi.generated.json` is generated from contract-constrained API spec.",
    ]
    return "\n".join(lines) + "\n"


def main() -> int:
    expected = render()
    if "--write" in sys.argv:
        SURFACE.parent.mkdir(parents=True, exist_ok=True)
        SURFACE.write_text(expected)
        return 0
    if not SURFACE.exists() or SURFACE.read_text() != expected:
        print("docs/api/v1-surface.md drift; run scripts/contracts/check_v1_surface.py --write", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
