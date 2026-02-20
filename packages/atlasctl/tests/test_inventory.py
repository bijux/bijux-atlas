from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import jsonschema

ROOT = Path(__file__).resolve().parents[3]


def _run_inventory(category: str, *extra: str) -> subprocess.CompletedProcess[str]:
    env = {"PYTHONPATH": str(ROOT / "packages/atlasctl/src")}
    return subprocess.run(
        [
            sys.executable,
            "-m",
            "atlasctl.cli",
            "inventory",
            category,
            "--format",
            "json",
            "--dry-run",
            *extra,
        ],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


def _validate(category: str, payload: dict[str, object]) -> None:
    schema_path = ROOT / f"configs/contracts/inventory-{category}.schema.json"
    schema = json.loads(schema_path.read_text(encoding="utf-8"))
    jsonschema.validate(payload, schema)


def test_inventory_json_categories_schema_valid() -> None:
    for category in (
        "make",
        "ops",
        "configs",
        "schemas",
        "owners",
        "contracts",
        "budgets",
        "scripts-migration",
        "legacy-scripts",
        "commands",
    ):
        proc = _run_inventory(category)
        assert proc.returncode == 0, proc.stderr
        payload = json.loads(proc.stdout)
        _validate(category, payload)


def test_inventory_touched_paths_is_deterministic() -> None:
    env = {"PYTHONPATH": str(ROOT / "packages/atlasctl/src")}
    cmd = [
        sys.executable,
        "-m",
        "atlasctl.cli",
        "inventory",
        "touched-paths",
        "--command",
        "check",
        "--format",
        "json",
        "--dry-run",
    ]
    first = subprocess.run(cmd, cwd=ROOT, env=env, text=True, capture_output=True, check=False)
    second = subprocess.run(cmd, cwd=ROOT, env=env, text=True, capture_output=True, check=False)
    assert first.returncode == 0, first.stderr
    assert second.returncode == 0, second.stderr
    assert first.stdout == second.stdout


def test_inventory_budgets_check_passes() -> None:
    proc = _run_inventory("budgets", "--check")
    assert proc.returncode == 0, proc.stderr
