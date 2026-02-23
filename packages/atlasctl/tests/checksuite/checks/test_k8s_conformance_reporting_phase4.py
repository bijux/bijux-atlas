from __future__ import annotations

import json
import subprocess
from pathlib import Path

from jsonschema import Draft202012Validator


ROOT = Path(__file__).resolve().parents[5]


def _run(rel: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["python3", rel], cwd=ROOT, text=True, capture_output=True, check=False)


def test_k8s_render_kind_golden_contract_check_passes() -> None:
    proc = _run("packages/atlasctl/src/atlasctl/checks/layout/ops/validation/check_k8s_render_kind_golden_contract.py")
    assert proc.returncode == 0, proc.stderr


def test_k8s_conformance_report_sample_matches_schema() -> None:
    schema = json.loads((ROOT / "ops/_schemas/k8s/conformance-report.schema.json").read_text(encoding="utf-8"))
    sample = json.loads((ROOT / "ops/k8s/tests/goldens/k8s-conformance-report.sample.json").read_text(encoding="utf-8"))
    Draft202012Validator(schema).validate(sample)


def test_k8s_conformance_report_command_generates_json_and_md() -> None:
    tmp_dir = ROOT / "artifacts" / "tmp" / "pytest-k8s-conformance"
    tmp_dir.mkdir(parents=True, exist_ok=True)
    suite_json = tmp_dir / "suite.json"
    suite_json.write_text(
        json.dumps(
            {
                "run_id": "r1",
                "suite_id": "smoke",
                "results": [
                    {"script": "checks/autoscaling/runtime/test_hpa.sh", "status": "passed"},
                    {"script": "checks/autoscaling/runtime/test_hpa_under_latency_metric.sh", "status": "passed"},
                    {"script": "checks/autoscaling/runtime/test_hpa_under_inflight_metric.sh", "status": "passed"},
                    {"script": "checks/autoscaling/contracts/test_hpa_max_replicas_cap.sh", "status": "passed"},
                    {"script": "checks/config/test_configmap.sh", "status": "passed"},
                    {"script": "checks/config/test_configmap_must_exist.sh", "status": "passed"},
                    {"script": "checks/config/test_configmap_version_stamp.sh", "status": "passed"},
                    {"script": "checks/perf/degradation/test_liveness_under_load.sh", "status": "passed"},
                    {"script": "checks/datasets/test_readiness_semantics.sh", "status": "passed"},
                    {"script": "checks/datasets/test_pdb.sh", "status": "passed"},
                    {"script": "checks/datasets/test_pdb_required_when_replicas_gt1.sh", "status": "passed"},
                    {"script": "checks/network/test_networkpolicy.sh", "status": "passed"},
                    {"script": "checks/network/test_networkpolicy_metadata_egress.sh", "status": "passed"},
                    {"script": "checks/obs/runtime/test_service_monitor.sh", "status": "passed"},
                    {"script": "checks/obs/runtime/test_prometheus_rule.sh", "status": "passed"},
                    {"script": "checks/obs/runtime/test_observability_objects_contract.sh", "status": "passed"}
                ],
            },
            sort_keys=True,
        ),
        encoding="utf-8",
    )
    out_json = tmp_dir / "conformance.json"
    out_md = tmp_dir / "conformance.md"
    proc = subprocess.run(
        [
            "./bin/atlasctl",
            "ops",
            "k8s",
            "--report",
            "text",
            "conformance-report",
            "--suite-json",
            suite_json.relative_to(ROOT).as_posix(),
            "--out-json",
            out_json.relative_to(ROOT).as_posix(),
            "--out-md",
            out_md.relative_to(ROOT).as_posix(),
        ],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    assert proc.returncode == 0, proc.stderr
    assert out_json.exists()
    assert out_md.exists()
