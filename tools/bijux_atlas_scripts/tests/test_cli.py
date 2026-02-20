from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

from bijux_atlas_scripts.cli import build_parser

ROOT = Path(__file__).resolve().parents[3]


def _run_cli(*args: str) -> subprocess.CompletedProcess[str]:
    env = {"PYTHONPATH": str(ROOT / "tools/bijux_atlas_scripts/src")}
    return subprocess.run(
        [sys.executable, "-m", "bijux_atlas_scripts.cli", *args],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


def test_parser_run_subcommand() -> None:
    parser = build_parser()
    ns = parser.parse_args(["run", "scripts/areas/layout/render_public_help.py", "--mode", "list"])
    assert ns.cmd == "run"
    assert ns.script.endswith("render_public_help.py")


def test_help_for_all_subcommands() -> None:
    for subcommand in ("run", "validate-output", "surface", "doctor"):
        proc = _run_cli(subcommand, "--help")
        assert proc.returncode == 0
        assert "usage:" in proc.stdout.lower()


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


def test_no_network_mode_blocks_connect() -> None:
    code = "import socket; socket.create_connection(('example.com', 80), timeout=0.1)"
    probe = ROOT / "artifacts/scripts/no_network_probe.py"
    probe.parent.mkdir(parents=True, exist_ok=True)
    probe.write_text(code, encoding="utf-8")
    proc = _run_cli("--no-network", "run", str(probe.relative_to(ROOT)))
    assert proc.returncode == 11
    assert "network disabled by --no-network" in proc.stderr
