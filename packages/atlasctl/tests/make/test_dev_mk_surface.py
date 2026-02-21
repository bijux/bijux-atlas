from __future__ import annotations

import re
import subprocess

from tests.helpers import ROOT

TARGET_RE = re.compile(r"^([A-Za-z0-9_./-]+):(?:\s|$)", re.M)


def _dev_mk_declared_targets() -> set[str]:
    text = (ROOT / "makefiles/dev.mk").read_text(encoding="utf-8")
    return {t for t in TARGET_RE.findall(text) if not t.startswith(".")}


def test_dev_mk_declares_exact_expected_targets() -> None:
    expected = {
        "fmt",
        "fmt-all",
        "fmt-and-slow",
        "lint",
        "lint-all",
        "lint-and-slow",
        "test",
        "test-all",
        "test-and-slow",
        "audit",
        "audit-all",
        "audit-and-slow",
        "coverage",
        "coverage-all",
        "coverage-and-slow",
        "check",
        "check-all",
        "check-and-slow",
        "all",
        "all-all",
        "all-and-slow",
        "internal/dev/check",
    }
    assert _dev_mk_declared_targets() == expected


def test_make_qp_contains_expected_dev_targets() -> None:
    proc = subprocess.run(
        ["make", "-qp", "-f", "makefiles/dev.mk"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    assert proc.returncode in {0, 1}, proc.stderr
    parsed = {t for t in TARGET_RE.findall(proc.stdout) if t in _dev_mk_declared_targets()}
    assert parsed == _dev_mk_declared_targets()
