#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[8]
MANIFEST = ROOT / "ops/k8s/tests/manifest.json"
SUITES = ROOT / "ops/k8s/tests/suites.json"


def _tests_by_script(manifest: dict) -> dict[str, dict]:
    rows = {}
    for row in manifest.get("tests", []):
        if isinstance(row, dict) and isinstance(row.get("script"), str):
            rows[str(row["script"])] = row
    return rows


def _require_scripts(rows: dict[str, dict], scripts: list[str], label: str, errs: list[str]) -> None:
    missing = [s for s in scripts if s not in rows]
    if missing:
        errs.append(f"{label}: missing tests in ops/k8s/tests/manifest.json: {', '.join(missing)}")


def main() -> int:
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    suites = json.loads(SUITES.read_text(encoding="utf-8"))
    rows = _tests_by_script(manifest)
    errs: list[str] = []

    # 141 values schema strictness / unknown keys
    _require_scripts(rows, ["checks/obs/contracts/test_values_schema_strict.sh"], "values schema strictness", errs)
    _require_scripts(rows, ["checks/config/test_configmap_unknown_keys_rejected.sh"], "unknown config keys rejected", errs)

    # 142 required labels/annotations/observability+rollout metadata coverage
    _require_scripts(
        rows,
        [
            "checks/obs/runtime/test_observability_objects_contract.sh",
            "checks/config/test_deployment_envFrom_configmap.sh",
        ],
        "required labels/annotations and rollout wiring",
        errs,
    )

    # 143 perf probes and resources in perf profiles
    _require_scripts(
        rows,
        [
            "checks/perf/pressure/test_resource_limits.sh",
            "checks/perf/degradation/test_liveness_under_load.sh",
        ],
        "perf profile probes/resources",
        errs,
    )

    # 144 networkpolicy egress matches network checks
    _require_scripts(
        rows,
        ["checks/network/test_networkpolicy.sh", "checks/network/test_networkpolicy_metadata_egress.sh"],
        "networkpolicy egress coverage",
        errs,
    )

    # 145 PDB check in smoke suite
    smoke_groups = []
    for suite in suites.get("suites", []):
        if isinstance(suite, dict) and suite.get("id") == "smoke":
            smoke_groups = [str(g) for g in suite.get("groups", []) if isinstance(g, str)]
            break
    if "pdb" not in smoke_groups:
        errs.append("ops/k8s/tests/suites.json smoke suite must include `pdb` group")
    _require_scripts(rows, ["checks/datasets/test_pdb_required_when_replicas_gt1.sh"], "PDB contract test", errs)

    # 146 HPA requires metrics stack
    _require_scripts(rows, ["checks/autoscaling/contracts/test_hpa_enabled_requires_metrics_stack.sh"], "HPA metrics stack contract", errs)

    # 147 configmap version stamp/checksum rollout behavior
    _require_scripts(
        rows,
        ["checks/config/test_configmap_version_stamp.sh", "checks/obs/contracts/test_no_checksum_rollout.sh"],
        "configmap version stamp / checksum rollout policy",
        errs,
    )

    # 150-154 explicit conformance slices via existing tests
    _require_scripts(rows, ["checks/obs/contracts/test_chart_drift.sh"], "chart drift", errs)
    _require_scripts(rows, ["checks/security/test_rbac_minimalism.sh"], "RBAC minimalism", errs)
    _require_scripts(rows, ["checks/security/test_secrets_rotation.sh"], "secret rotation drill", errs)
    _require_scripts(rows, ["checks/datasets/test_offline_profile.sh"], "offline profile install behavior", errs)
    _require_scripts(rows, ["checks/datasets/test_warmup_job.sh"], "dataset warmup job contract", errs)

    if errs:
        print("\n".join(errs), file=sys.stderr)
        return 1
    print("k8s conformance contract coverage checks passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

