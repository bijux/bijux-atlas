from __future__ import annotations

from tests.helpers import golden_path, run_atlasctl


def test_check_help_snapshot() -> None:
    proc = run_atlasctl("check", "--help")
    assert proc.returncode == 0, proc.stderr
    golden = golden_path("check_help_snapshot.txt").read_text(encoding="utf-8")
    assert proc.stdout == golden
