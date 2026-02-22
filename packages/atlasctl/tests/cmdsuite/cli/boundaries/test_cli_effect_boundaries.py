from __future__ import annotations

import json
from pathlib import Path

from tests.helpers import ROOT, run_atlasctl_isolated


def test_allow_network_banner_and_manifest_flag(tmp_path: Path) -> None:
    run_id = "allow-net-manifest-test"
    proc = run_atlasctl_isolated(tmp_path, "--allow-network", "--run-id", run_id, "--json", "help")
    assert proc.returncode == 0, proc.stderr
    assert "NETWORK ENABLED:" in proc.stderr
    report = ROOT / "artifacts" / "atlasctl" / "run" / run_id / "reports" / "run-manifest.json"
    payload = json.loads(report.read_text(encoding="utf-8"))
    assert payload["network_mode"] == "allow"
    assert payload["network_requested"] is True


def test_forbidden_write_is_machine_readable_json_error(tmp_path: Path) -> None:
    proc = run_atlasctl_isolated(
        tmp_path,
        "--json",
        "surface",
        "--json",
        "--out-file",
        "ops/forbidden-write.json",
    )
    assert proc.returncode != 0
    payload = json.loads(proc.stderr.strip().splitlines()[-1])
    assert payload["status"] == "error"
    assert payload["errors"][0]["kind"] == "forbidden_write_path"
