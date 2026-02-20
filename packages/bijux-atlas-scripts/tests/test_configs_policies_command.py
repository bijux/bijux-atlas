from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def _run_cli(*args: str) -> subprocess.CompletedProcess[str]:
    env = {"PYTHONPATH": str(ROOT / "packages/bijux-atlas-scripts/src")}
    extra: list[str] = []
    if os.environ.get("BIJUX_SCRIPTS_TEST_NO_NETWORK") == "1":
        extra.append("--no-network")
    return subprocess.run(
        [sys.executable, "-m", "bijux_atlas_scripts.cli", *extra, *args],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


def test_configs_inventory_json() -> None:
    proc = _run_cli("configs", "inventory", "--format", "json", "--report", "json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["kind"] == "configs-inventory"
    assert payload["schema_version"] == 1
    assert any(row["path"] == "configs/README.md" for row in payload["files"])


def test_configs_schema_check_json() -> None:
    proc = _run_cli("configs", "schema-check", "--report", "json")
    assert proc.returncode in {0, 1}, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["tool"] == "bijux-atlas"
    assert payload["status"] in {"pass", "fail"}


def test_configs_print_json() -> None:
    proc = _run_cli("configs", "print", "--report", "json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["tool"] == "bijux-atlas"
    assert payload["status"] == "pass"
    assert "ops_tool_versions" in payload["output"]


def test_configs_drift_json() -> None:
    proc = _run_cli("configs", "drift", "--report", "json")
    assert proc.returncode in {0, 1}, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["tool"] == "bijux-atlas"
    assert payload["command"] == "drift"


def test_configs_validate_json() -> None:
    proc = _run_cli("configs", "validate", "--report", "json")
    assert proc.returncode in {0, 1}, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["tool"] == "bijux-atlas"
    assert payload["command"] == "validate"


def test_policies_relaxations_check_json() -> None:
    proc = _run_cli("policies", "relaxations-check", "--report", "json")
    assert proc.returncode in {0, 1}, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["schema_version"] == 1
    assert "active_relaxations" in payload


def test_policies_bypass_scan_fixture_detects_missing_relaxation_id(tmp_path: Path) -> None:
    (tmp_path / "makefiles").mkdir(parents=True)
    (tmp_path / "makefiles/root.mk").write_text("target:\n\t@echo BYPASS this check\n", encoding="utf-8")

    from bijux_atlas_scripts.policies.command import _bypass_scan

    code, payload = _bypass_scan(tmp_path)
    assert code == 1
    assert payload["offenders"]


def test_policies_relaxations_fixture_expiry_enforced(tmp_path: Path) -> None:
    (tmp_path / "configs/policy").mkdir(parents=True)
    (tmp_path / "configs/_schemas").mkdir(parents=True)

    (tmp_path / "configs/policy/ops-lint-relaxations.json").write_text(
        json.dumps(
            {
                "schema_version": 1,
                "relaxations": [
                    {
                        "check_id": "x",
                        "owner": "team",
                        "issue": "ISSUE-1",
                        "expires_on": "2000-01-01",
                    }
                ],
            }
        ),
        encoding="utf-8",
    )
    (tmp_path / "configs/_schemas/ops-lint-relaxations.schema.json").write_text(
        json.dumps(
            {
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "type": "object",
                "required": ["schema_version", "relaxations"],
                "properties": {
                    "schema_version": {"type": "integer"},
                    "relaxations": {"type": "array"},
                },
            }
        ),
        encoding="utf-8",
    )
    for rel in (
        "pin-relaxations.json",
        "budget-relaxations.json",
        "layer-relaxations.json",
        "ops-smoke-budget-relaxations.json",
    ):
        (tmp_path / "configs/policy" / rel).write_text(
            json.dumps({"schema_version": 1, "exceptions": []}),
            encoding="utf-8",
        )

    from bijux_atlas_scripts.policies.command import _check_relaxations

    code, payload = _check_relaxations(tmp_path, require_docs_ref=False)
    assert code == 1
    assert any("expired on" in err for err in payload["errors"])
