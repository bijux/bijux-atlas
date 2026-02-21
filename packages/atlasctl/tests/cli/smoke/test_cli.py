from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

from atlasctl.cli import build_parser

ROOT = Path(__file__).resolve().parents[3]


def _run_cli(*args: str) -> subprocess.CompletedProcess[str]:
    env = {"PYTHONPATH": str(ROOT / "packages/atlasctl/src")}
    extra: list[str] = []
    if os.environ.get("BIJUX_SCRIPTS_TEST_NO_NETWORK") == "1":
        extra.append("--no-network")
    return subprocess.run(
        [sys.executable, "-m", "atlasctl.cli", *extra, *args],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


def test_parser_run_subcommand() -> None:
    parser = build_parser()
    ns = parser.parse_args(["run", "packages/atlasctl/src/atlasctl/checks/layout/public_surface/tools/render_public_help.py", "--mode", "list"])
    assert ns.cmd == "run"
    assert ns.script.endswith("render_public_help.py")


def test_help_for_all_commands() -> None:
    commands = (
        "run",
        "validate-output",
        "surface",
        "commands",
        "doctor",
        "ops",
        "docs",
        "configs",
        "policies",
        "make",
        "inventory",
        "contracts",
        "registry",
        "layout",
        "report",
        "internal",
    )
    for command in commands:
        proc = _run_cli(command, "--help")
        assert proc.returncode == 0
        assert "usage:" in proc.stdout.lower()


def test_version_flag() -> None:
    proc = _run_cli("--version")
    assert proc.returncode == 0
    assert "atlasctl 0.1.0+" in proc.stdout.strip()


def test_surface_json_schema_valid() -> None:
    import jsonschema

    proc = _run_cli("surface", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    schema = json.loads((ROOT / "configs/contracts/scripts-surface-output.schema.json").read_text(encoding="utf-8"))
    jsonschema.validate(payload, schema)


def test_doctor_json_schema_valid() -> None:
    import jsonschema

    proc = _run_cli("--run-id", "t1", "--profile", "test", "doctor", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    schema = json.loads((ROOT / "configs/contracts/scripts-doctor-output.schema.json").read_text(encoding="utf-8"))
    jsonschema.validate(payload, schema)


def test_contracts_check_json_contract() -> None:
    proc = _run_cli("--run-id", "t2", "--profile", "test", "contracts", "check", "--report", "json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["schema_version"] == 1
    assert payload["tool"] == "atlasctl"
    assert payload["status"] == "pass"


def test_global_json_flag_applies_to_version() -> None:
    proc = _run_cli("--json", "version")
    assert proc.returncode == 0
    payload = json.loads(proc.stdout)
    assert payload["tool"] == "atlasctl"
    assert "scripts_version" in payload


def test_explain_command_contract() -> None:
    proc = _run_cli("--json", "explain", "check")
    assert proc.returncode == 0
    payload = json.loads(proc.stdout)
    assert payload["tool"] == "atlasctl"
    assert payload["command"] == "check"
    assert isinstance(payload["touches"], list)


def test_python_run_alias_contract() -> None:
    create = _run_cli("--json", "python", "venv", "create")
    assert create.returncode == 0
    proc = _run_cli("--json", "python", "run", "--", "python3", "-c", "import sys")
    assert proc.returncode == 0
    payload = json.loads(proc.stdout)
    assert payload["tool"] == "atlasctl"
    assert payload["status"] == "ok"


def test_out_file_policy_rejects_ops_path() -> None:
    proc = _run_cli("help", "--json", "--out-file", "ops/_evidence/forbidden.json")
    assert proc.returncode == 3
    assert "forbidden write path" in proc.stderr


def test_no_network_mode_blocks_connect() -> None:
    code = "import socket; socket.create_connection(('example.com', 80), timeout=0.1)"
    probe = ROOT / "artifacts/scripts/no_network_probe.py"
    probe.parent.mkdir(parents=True, exist_ok=True)
    probe.write_text(code, encoding="utf-8")
    proc = _run_cli("--no-network", "run", str(probe.relative_to(ROOT)))
    assert proc.returncode == 2
    assert "network disabled by --no-network" in proc.stderr
