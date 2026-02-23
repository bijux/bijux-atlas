from __future__ import annotations

import argparse
from pathlib import Path
from types import SimpleNamespace

from atlasctl.checks.model import CheckDef
from atlasctl.checks.effects import CheckEffect
from atlasctl.checks.report import resolve_last_run_report
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
    ctx = SimpleNamespace(repo_root=tmp_path, evidence_root=tmp_path / "artifacts" / "evidence", output_format="json", run_id="run-1", quiet=False, profile="local")
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
    ctx = SimpleNamespace(repo_root=tmp_path, evidence_root=tmp_path / "artifacts" / "evidence", output_format="json", run_id="run-1", quiet=False, profile="local")
    rc = check_command._run_check_registry(ctx, _ns(write_root="artifacts/runs/run-1/checks"))
    assert rc == 0
    assert captured["run_root"] == (tmp_path / "artifacts/runs/run-1/checks").resolve()


def test_check_run_writes_default_unified_report(monkeypatch, tmp_path: Path) -> None:
    check = CheckDef(
        "checks_repo_writer_guard",
        "repo",
        "writer guard",
        1000,
        lambda _root: (0, []),
        effects=(CheckEffect.FS_READ.value,),
    )
    monkeypatch.setattr(check_command, "list_checks", lambda: (check,))
    monkeypatch.setattr(check_command, "ensure_evidence_path", lambda _ctx, path: (tmp_path / path).resolve())
    writes: list[Path] = []

    def _fake_write(path: Path, _content: str, encoding: str = "utf-8") -> None:  # noqa: ARG001
        writes.append(path)

    def _fake_runner(repo_root: Path, **kwargs):  # type: ignore[no-untyped-def]
        return 0, {"rows": [{"id": "checks_repo_writer_guard", "status": "PASS", "duration_ms": 1, "domain": "repo", "reason": "", "hints": [], "owners": [], "artifacts": [], "findings": [], "category": "check", "attachments": []}], "events": [], "attachments": []}

    monkeypatch.setattr(check_command, "write_text_file", _fake_write)
    monkeypatch.setattr(check_command, "run_checks_payload", _fake_runner)
    monkeypatch.setattr(check_command, "emit_telemetry", lambda *_a, **_k: None)
    ctx = SimpleNamespace(repo_root=tmp_path, evidence_root=tmp_path / "artifacts" / "evidence", output_format="json", run_id="run-1", quiet=False, profile="local")
    rc = check_command._run_check_registry(ctx, _ns())
    assert rc == 0
    assert (tmp_path / "artifacts/evidence/run-1/checks/report.unified.json").resolve() in writes


def test_last_run_resolution_uses_mtime_then_path_order(tmp_path: Path) -> None:
    run_dir = tmp_path / "runs"
    run_dir.mkdir()
    a = run_dir / "a.json"
    b = run_dir / "b.json"
    payload = '{"kind":"check-run","rows":[]}'
    a.write_text(payload, encoding="utf-8")
    b.write_text(payload, encoding="utf-8")
    # Keep equal mtime to assert path-name tie break determinism.
    stat = a.stat()
    b.touch()
    import os

    os.utime(a, (stat.st_atime, stat.st_mtime))
    os.utime(b, (stat.st_atime, stat.st_mtime))
    resolved = resolve_last_run_report(str(run_dir))
    assert resolved == a
