#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

# Every JSON under ops/ must be covered either by a schema validation pair or a dedicated validator.
SCHEMA_COVERED = {
    "ops/stack/profiles.json",
    "ops/k8s/install-matrix.json",
    "ops/load/suites/suites.json",
    "ops/obs/drills/drills.json",
    "ops/_generated/report.example.json",
    "ops/_generated/report.unified.example.json",
    "ops/stack/version-manifest.json",
    "ops/_meta/ownership.json",
    "ops/load/queries/pinned-v1.lock",
    "ops/datasets/manifest.lock",
    "ops/e2e/scenarios/scenarios.json",
    "ops/e2e/realdata/scenarios.json",
}

VALIDATOR_COVERED_PREFIXES = (
    "ops/_schemas/",
    "ops/fixtures/",
    "ops/load/scenarios/",
    "ops/load/contracts/",
    "ops/k8s/charts/",
    "ops/obs/grafana/",
)

VALIDATOR_COVERED_FILES = {
    "ops/_meta/contracts.json",  # checked by ops/_lint/no-orphan-contract.py
    "ops/_meta/surface.json",  # checked by scripts/layout/check_ops_surface_drift.py
    "ops/_meta/error-registry.json",  # consumed by ops/_lib/errors.sh and contract docs
    "ops/obs/contract/alerts-contract.json",  # checked by check_alerts_contract.py
    "ops/obs/contract/dashboard-panels-contract.json",  # checked by check_dashboards_contract.py
    "ops/obs/contract/logs-fields-contract.json",  # checked by check_logs_contract.py
    "ops/obs/contract/metrics-contract.json",  # checked by check_metrics_coverage.py
    "ops/obs/contract/trace-structure.golden.json",  # checked by check_trace_structure.py
    "ops/obs/drills/result.schema.json",  # consumed by drill report validators
    "ops/report/schema.json",  # used by ops/report/report_contract_check.py
    "ops/registry/pins.json",  # validated by ops-registry pin checks
    "ops/datasets/manifest.json",  # validated by datasets scripts contract checks
    "ops/datasets/real-datasets.json",  # validated by ops/e2e realdata suite loader
    "ops/e2e/realdata/canonical_queries.json",  # validated by realdata suite
    "ops/e2e/realdata/snapshots/release110_snapshot.json",  # validated by snapshot checks
    "ops/e2e/smoke/goldens/status_codes.json",  # validated by smoke query snapshot checks
    "ops/k8s/tests/manifest.json",  # validated by ops/k8s/tests/validate_suites.py
    "ops/k8s/tests/ownership.json",  # validated by k8s test ownership checks
    "ops/k8s/tests/suites.json",  # validated by ops/k8s/tests/validate_suites.py
    "ops/load/baselines/ci-runner.json",  # validated by load baseline checks
    "ops/load/baselines/local.json",  # validated by load baseline checks
    "ops/load/queries/pinned-v1.json",  # validated by load query lock checks
    "ops/obs/pack/compose.schema.json",  # used by pack compose validation
    "ops/obs/tests/suites.json",  # validated by ops/_lint/no-orphan-suite.py
    "ops/stack/versions.json",  # generated from configs/ops/tool-versions.json and checked in contracts
}

EXCLUDE_PREFIXES = (
    "ops/_artifacts/",
    "ops/_generated/gates/",
    "ops/_generated/ops-",
    "ops/_generated/stack/",
    "ops/_generated/k8s/",
    "ops/_generated/load/",
    "ops/_generated/obs/",
)


def is_covered(path: str) -> bool:
    if path in SCHEMA_COVERED or path in VALIDATOR_COVERED_FILES:
        return True
    if any(path.startswith(prefix) for prefix in VALIDATOR_COVERED_PREFIXES):
        return True
    return False


def main() -> int:
    errors: list[str] = []
    for p in sorted((ROOT / "ops").rglob("*.json")):
        rel = p.relative_to(ROOT).as_posix()
        if any(rel.startswith(prefix) for prefix in EXCLUDE_PREFIXES):
            continue
        if not is_covered(rel):
            errors.append(rel)

    if errors:
        print("ops json files without declared schema/validator coverage:", file=sys.stderr)
        for rel in errors:
            print(f"- {rel}", file=sys.stderr)
        return 1

    print("ops json schema coverage passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
