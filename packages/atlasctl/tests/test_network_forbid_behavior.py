from __future__ import annotations

from pathlib import Path

from helpers import run_atlasctl


def test_no_network_mode_blocks_network_probe(tmp_path: Path) -> None:
    probe = tmp_path / "probe.py"
    probe.write_text("import socket\nsocket.create_connection(('example.com', 80), timeout=0.1)\n", encoding="utf-8")
    proc = run_atlasctl("--network", "forbid", "run", str(probe), cwd=Path.cwd())
    assert proc.returncode != 0
