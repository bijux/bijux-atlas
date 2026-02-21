from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def _run_cli(*args: str) -> subprocess.CompletedProcess[str]:
    env = {"PYTHONPATH": str(ROOT / "packages/atlasctl/src")}
    return subprocess.run(
        [sys.executable, "-m", "atlasctl.cli", *args],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


def test_breaking_contract_change_fails(tmp_path: Path) -> None:
    before = tmp_path / "before.json"
    after = tmp_path / "after.json"
    before.write_text(
        json.dumps(
            {"endpoints": [{"method": "GET", "path": "/v1/health"}, {"method": "GET", "path": "/v1/items"}]},
            sort_keys=True,
        ),
        encoding="utf-8",
    )
    after.write_text(
        json.dumps({"endpoints": [{"method": "GET", "path": "/v1/health"}]}, sort_keys=True),
        encoding="utf-8",
    )
    proc = _run_cli(
        "contracts",
        "check",
        "--report",
        "json",
        "--checks",
        "breakage",
        "--before",
        str(before),
        "--after",
        str(after),
    )
    assert proc.returncode == 1
    payload = json.loads(proc.stdout)
    assert payload["status"] == "fail"
    assert any("removed endpoints" in err for err in payload["errors"])


def test_non_breaking_contract_change_passes(tmp_path: Path) -> None:
    before = tmp_path / "before.json"
    after = tmp_path / "after.json"
    before.write_text(
        json.dumps({"endpoints": [{"method": "GET", "path": "/v1/health"}]}, sort_keys=True),
        encoding="utf-8",
    )
    after.write_text(
        json.dumps(
            {"endpoints": [{"method": "GET", "path": "/v1/health"}, {"method": "GET", "path": "/v1/items"}]},
            sort_keys=True,
        ),
        encoding="utf-8",
    )
    proc = _run_cli(
        "contracts",
        "check",
        "--report",
        "json",
        "--checks",
        "breakage",
        "--before",
        str(before),
        "--after",
        str(after),
    )
    assert proc.returncode == 0
    payload = json.loads(proc.stdout)
    assert payload["status"] == "pass"
    assert payload["errors"] == []
