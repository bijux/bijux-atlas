from __future__ import annotations

import json
import socket
from fnmatch import fnmatch
from pathlib import Path

from atlasctl.cli import main as cli_main
from atlasctl.core.scan import iter_python_files

ROOT = Path(__file__).resolve().parents[3]


def test_scripts_python_files_are_tracked_by_migration_exception() -> None:
    cfg = json.loads((ROOT / "configs/layout/python-migration-exceptions.json").read_text(encoding="utf-8"))
    exceptions = cfg.get("exceptions", [])
    globs = [e.get("path_glob", "") for e in exceptions if e.get("kind") in {"scripts_dir", "executable_python"}]
    assert globs, "missing scripts migration exceptions"

    scripts_root = ROOT / "scripts"
    if not scripts_root.exists():
        return
    for path in iter_python_files(scripts_root):
        rel = path.relative_to(ROOT).as_posix()
        assert any(fnmatch(rel, g) or rel == g for g in globs), f"untracked legacy python file: {rel}"


def test_no_hidden_imports_do_not_touch_network(monkeypatch) -> None:
    called = {"connect": 0}

    def _deny(*_args, **_kwargs):
        called["connect"] += 1
        raise AssertionError("network access attempted during import")

    monkeypatch.setattr(socket, "create_connection", _deny)
    # Import surface modules expected to be safe in offline environments.
    from atlasctl import cli, contracts, docs, layout, make, ops, report  # noqa: F401

    assert called["connect"] == 0


def test_offline_mode_flag_is_respected() -> None:
    rc = cli_main(["--network", "forbid", "--quiet", "env", "--json"])
    assert rc == 0
