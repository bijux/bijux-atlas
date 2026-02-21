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


def test_env_info_json() -> None:
    proc = _run_cli("--quiet", "env", "info", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["status"] == "pass"
    assert "artifact_root" in payload


def test_env_create_and_clean(tmp_path: Path) -> None:
    venv_path = tmp_path / "venv" / ".venv"
    create = _run_cli("--quiet", "env", "create", "--path", str(venv_path), "--json")
    assert create.returncode == 0, create.stderr
    created = json.loads(create.stdout)
    assert created["status"] == "pass"
    assert venv_path.exists()

    clean = _run_cli("--quiet", "env", "clean", "--json")
    assert clean.returncode == 0, clean.stderr
    payload = json.loads(clean.stdout)
    assert payload["status"] == "pass"


def test_runtime_contract_files_emitted() -> None:
    run_id = "env-contract-test"
    proc = _run_cli("--quiet", "--run-id", run_id, "doctor", "--json")
    assert proc.returncode == 0, proc.stderr
    root = ROOT / "artifacts/atlasctl/run" / run_id / "reports"
    assert (root / "write-roots-contract.json").exists()
    assert (root / "run-manifest.json").exists()
