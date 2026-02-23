from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[5]


def test_obs_drill_dry_run_emits_contract_valid_report() -> None:
    proc = subprocess.run(
        [
            "python3",
            "packages/atlasctl/src/atlasctl/commands/ops/observability/drills/run_drill.py",
            "--id",
            "store-outage-under-load",
            "--dry-run",
        ],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    assert proc.returncode == 0, proc.stderr
    result = ROOT / "artifacts/observability/drills/store-outage-under-load.result.json"
    payload = json.loads(result.read_text(encoding="utf-8"))
    schema = json.loads((ROOT / "ops/obs/drills/result.schema.json").read_text(encoding="utf-8"))
    for key in schema["required"]:
        assert key in payload
    assert payload["drill"] == "store-outage-under-load"


def test_atlasctl_obs_drill_accepts_id_flag() -> None:
    proc = subprocess.run(
        [str(ROOT / "bin/atlasctl"), "ops", "obs", "drill", "--id", "store-outage-under-load", "--report", "text"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    # Command may fail live depending on local tools/cluster, but parser/runtime should not reject --id.
    assert "missing --drill/--id" not in (proc.stdout + proc.stderr)
