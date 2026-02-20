#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

CONTRACTS = {
    "hpa": [
        "test_hpa.sh",
        "test_hpa_under_latency_metric.sh",
        "test_hpa_under_inflight_metric.sh",
        "test_hpa_max_replicas_cap.sh",
    ],
    "configmap": [
        "test_configmap.sh",
        "test_configmap_must_exist.sh",
        "test_configmap_version_stamp.sh",
    ],
    "probes": [
        "test_liveness_under_load.sh",
        "test_readiness_semantics.sh",
    ],
    "pdb": [
        "test_pdb.sh",
        "test_pdb_required_when_replicas_gt1.sh",
    ],
    "networkpolicy": [
        "test_networkpolicy.sh",
        "test_networkpolicy_metadata_egress.sh",
    ],
    "observability_wiring": [
        "test_service_monitor.sh",
        "test_prometheus_rule.sh",
        "test_observability_objects_contract.sh",
    ],
}


def main() -> int:
    import argparse

    p = argparse.ArgumentParser(description="Build k8s conformance report from suite results")
    p.add_argument("--json", required=True)
    p.add_argument("--out", required=True)
    args = p.parse_args()

    payload = json.loads(Path(args.json).read_text(encoding="utf-8"))
    by_name = {Path(r.get("script", "")).name: r for r in payload.get("results", [])}
    section = {}
    failed_sections = []
    for name, tests in CONTRACTS.items():
        missing = [t for t in tests if t not in by_name]
        failed = [t for t in tests if by_name.get(t, {}).get("status") == "failed"]
        status = "pass" if not missing and not failed else "fail"
        if status == "fail":
            failed_sections.append(name)
        section[name] = {"status": status, "missing": missing, "failed": failed}

    out_payload = {
        "schema_version": 1,
        "run_id": payload.get("run_id", "unknown"),
        "suite_id": payload.get("suite_id", "unknown"),
        "status": "pass" if not failed_sections else "fail",
        "failed_sections": failed_sections,
        "sections": section,
    }
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(out_payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"wrote {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
