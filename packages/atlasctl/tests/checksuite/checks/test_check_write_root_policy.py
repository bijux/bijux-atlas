from __future__ import annotations

import argparse
from pathlib import Path
from types import SimpleNamespace

from atlasctl.checks.core.base import CheckDef
from atlasctl.checks.effects import CheckEffect
from atlasctl.commands.check import command as check_command
from atlasctl.core.exit_codes import ERR_USER


def _ns(**kwargs: object) -> argparse.Namespace:
    base: dict[str, object] = {
        "select": "",
        "id": "",
        "k": "",
        "check_target": "",
        "domain_filter": "",
        "suite": "",
        "category": "",
        "group": "",
        "exclude_group": [],
        "only_slow": False,
        "only_fast": False,
        "exclude_slow": False,
        "include_all": True,
        "marker": [],
        "require_markers": [],
        "exclude_marker": [],
        "match": "",
        "list_selected": False,
        "jsonl": False,
        "json": True,
        "timeout_ms": 500,
        "run_quiet": False,
        "run_verbose": False,
        "jobs": 1,
        "max_failures": 0,
        "maxfail": 0,
        "failfast": False,
        "keep_going": True,
        "slow_threshold_ms": 800,
        "slow_ratchet_config": "configs/policy/slow-checks-ratchet.json",
        "ignore_speed_regressions": True,
        "durations": 0,
        "json_report": None,
        "slow_report": None,
        "profile": False,
        "profile_out": None,
        "junitxml": None,
        "write_root": "",
    }
    base.update(kwargs)
    return argparse.Namespace(**base)


def test_write_enabled_checks_require_write_root(monkeypatch, tmp_path: Path) -> None:
    check = CheckDef(
        "checks_repo_writer_guard",
        "repo",
        "writer guard",
        1000,
        lambda _root: (0, []),
        effects=(CheckEffect.FS_READ.value, CheckEffect.FS_WRITE.value),
    )
    monkeypatch.setattr(check_command, "list_checks", lambda: (check,))
    ctx = SimpleNamespace(repo_root=tmp_path, output_format="json", run_id="run-1", quiet=False, profile="local")
    rc = check_command._run_check_registry(ctx, _ns())
    assert rc == ERR_USER


def test_write_enabled_checks_use_explicit_write_root(monkeypatch, tmp_path: Path) -> None:
    check = CheckDef(
        "checks_repo_writer_guard",
        "repo",
        "writer guard",
        1000,
        lambda _root: (0, []),
        effects=(CheckEffect.FS_READ.value, CheckEffect.FS_WRITE.value),
    )
    monkeypatch.setattr(check_command, "list_checks", lambda: (check,))
    captured: dict[str, Path] = {}

    def _fake_runner(repo_root: Path, **kwargs):  # type: ignore[no-untyped-def]
        captured["run_root"] = kwargs["options"].run_root
        return 0, {"rows": [], "events": [], "attachments": []}

    monkeypatch.setattr(check_command, "run_checks_payload", _fake_runner)
    ctx = SimpleNamespace(repo_root=tmp_path, output_format="json", run_id="run-1", quiet=False, profile="local")
    rc = check_command._run_check_registry(ctx, _ns(write_root="artifacts/runs/run-1/checks"))
    assert rc == 0
    assert captured["run_root"] == (tmp_path / "artifacts/runs/run-1/checks").resolve()
