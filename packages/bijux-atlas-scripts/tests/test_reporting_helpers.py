from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

from bijux_atlas_scripts.reporting.make_area_report import main as make_area_report_main


def test_make_area_report_writes_json(tmp_path: Path) -> None:
    out = tmp_path / "report.json"
    rc = make_area_report_main(
        [
            "--path",
            str(out),
            "--lane",
            "docs/check",
            "--status",
            "pass",
            "--start",
            "2026-01-01T00:00:00Z",
            "--end",
            "2026-01-01T00:00:01Z",
        ]
    )
    assert rc == 0
    payload = json.loads(out.read_text(encoding="utf-8"))
    assert payload["lane"] == "docs/check"
    assert payload["status"] == "pass"


def test_run_gate_module_executes_command(tmp_path: Path) -> None:
    (tmp_path / ".git").mkdir()
    proc = subprocess.run(
        [
            sys.executable,
            "-m",
            "bijux_atlas_scripts.reporting.run_gate",
            "sample",
            "sh",
            "-c",
            "true",
        ],
        cwd=tmp_path,
        env={"PYTHONPATH": str(Path(__file__).resolve().parents[1] / "src"), "RUN_ID": "r1"},
        text=True,
        capture_output=True,
        check=False,
    )
    assert proc.returncode == 0
    assert "gate-result:" in proc.stdout

