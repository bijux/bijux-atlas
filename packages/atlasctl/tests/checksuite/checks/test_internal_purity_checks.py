from __future__ import annotations

from pathlib import Path

from atlasctl.checks.domains.internal import (
    check_checks_no_direct_env_reads,
    check_checks_no_print_calls,
    check_checks_no_sys_exit_calls,
    check_checks_root_allowed_entries_only,
)


def _write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def test_no_print_calls_detects_print_usage(tmp_path: Path) -> None:
    _write(
        tmp_path / "packages/atlasctl/src/atlasctl/checks/tools/sample.py",
        "def run():\n    print('x')\n",
    )
    code, errors = check_checks_no_print_calls(tmp_path)
    assert code == 1
    assert any("print" in line for line in errors)


def test_no_sys_exit_calls_detects_usage(tmp_path: Path) -> None:
    _write(
        tmp_path / "packages/atlasctl/src/atlasctl/checks/tools/sample.py",
        "import sys\n\ndef run():\n    sys.exit(2)\n",
    )
    code, errors = check_checks_no_sys_exit_calls(tmp_path)
    assert code == 1
    assert any("sys.exit" in line for line in errors)


def test_no_direct_env_reads_detects_usage(tmp_path: Path) -> None:
    _write(
        tmp_path / "packages/atlasctl/src/atlasctl/checks/tools/sample.py",
        "import os\n\nX = os.environ.get('CI')\n",
    )
    code, errors = check_checks_no_direct_env_reads(tmp_path)
    assert code == 1
    assert any("environment" in line for line in errors)


def test_root_allowed_entries_only_flags_unknown_entries(tmp_path: Path) -> None:
    root = tmp_path / "packages/atlasctl/src/atlasctl/checks"
    root.mkdir(parents=True, exist_ok=True)
    _write(root / "__init__.py", "")
    _write(root / "model.py", "")
    _write(root / "registry.py", "")
    _write(root / "selectors.py", "")
    _write(root / "policy.py", "")
    _write(root / "runner.py", "")
    _write(root / "report.py", "")
    _write(root / "gen_registry.py", "")
    _write(root / "README.md", "")
    (root / "tools").mkdir(exist_ok=True)
    (root / "domains").mkdir(exist_ok=True)
    _write(root / "legacy_extra.py", "")
    code, errors = check_checks_root_allowed_entries_only(tmp_path)
    assert code == 1
    assert any("legacy_extra.py" in line for line in errors)
