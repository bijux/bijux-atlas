from __future__ import annotations

import subprocess

from tests.helpers import ROOT, golden_path


def test_make_help_snapshot() -> None:
    proc = subprocess.run(
        ["make", "help"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    assert proc.returncode == 0, proc.stderr
    golden = golden_path("help/make_help_snapshot.txt").read_text(encoding="utf-8")
    assert proc.stdout == golden
