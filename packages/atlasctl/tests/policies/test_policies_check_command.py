from __future__ import annotations

import argparse
from pathlib import Path

from atlasctl.core.context import RunContext
from atlasctl.policies import budget_handlers
from atlasctl.policies import command as policies_command


def _ctx(root: Path) -> RunContext:
    return RunContext(
        run_id="r1",
        profile="local",
        repo_root=root,
        evidence_root=root / "artifacts/evidence",
        scripts_artifact_root=root / "artifacts/atlasctl/run/r1",
        run_dir=root / "artifacts/evidence/r1",
        output_format="text",
        network_mode="allow",
        verbose=False,
        quiet=False,
        require_clean_git=False,
        no_network=False,
        git_sha="unknown",
        git_dirty=False,
    )


def test_policies_check_does_not_invoke_recursive_make_policy_targets(tmp_path: Path, monkeypatch) -> None:
    ns = argparse.Namespace(
        policies_cmd="check",
        report="json",
        emit_artifacts=False,
        fail_fast=False,
    )
    commands: list[list[str]] = []

    def _fake_run(cmd: list[str], repo_root: Path) -> tuple[int, str]:
        assert repo_root == tmp_path
        commands.append(cmd)
        return 0, ""

    monkeypatch.setattr(policies_command, "_run", _fake_run)
    assert policies_command.run_policies_command(_ctx(tmp_path), ns) == 0
    assert all(not (cmd[:2] == ["make", "-s"]) for cmd in commands)


def test_policies_repo_stats_writes_artifact(tmp_path: Path, monkeypatch) -> None:
    ns = argparse.Namespace(
        policies_cmd="repo-stats",
        report="json",
        out_file="artifacts/reports/atlasctl/repo-stats/test.json",
        diff_previous=False,
    )

    monkeypatch.setattr(
        policies_command,
        "_repo_stats_payload",
        lambda _repo_root: {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok",
            "total_dirs": 1,
            "top_dirs": [{"dir": "packages/atlasctl/src/atlasctl", "total_loc": 10}],
        },
    )
    code = policies_command.run_policies_command(_ctx(tmp_path), ns)
    assert code == 0
    out = tmp_path / ns.out_file
    assert out.exists()


def test_policies_explain_budgets_json(tmp_path: Path) -> None:
    pyproject = tmp_path / "packages/atlasctl/pyproject.toml"
    pyproject.parent.mkdir(parents=True, exist_ok=True)
    pyproject.write_text("[tool.atlasctl.budgets]\nmax_py_files_per_dir = 10\nmax_modules_per_dir = 10\n", encoding="utf-8")
    ns = argparse.Namespace(
        policies_cmd="explain",
        subject="budgets",
        report="json",
    )
    code = policies_command.run_policies_command(_ctx(tmp_path), ns)
    assert code == 0


def test_culprits_files_per_dir_alias(monkeypatch, tmp_path: Path, capsys) -> None:
    ns = argparse.Namespace(
        policies_cmd="culprits-files-per-dir",
        report="json",
        out_file="",
    )

    monkeypatch.setattr(
        budget_handlers,
        "evaluate_metric",
        lambda _repo, metric: {"schema_version": 1, "tool": "atlasctl", "status": "ok", "metric": metric, "items": []},
    )
    code = policies_command.run_policies_command(_ctx(tmp_path), ns)
    assert code == 0
    assert '"metric": "files-per-dir"' in capsys.readouterr().out


def test_check_dir_entry_budgets_json(monkeypatch, tmp_path: Path, capsys) -> None:
    ns = argparse.Namespace(
        policies_cmd="check-dir-entry-budgets",
        json=True,
        print_culprits=False,
        top=5,
        fail_on_warn=False,
        out_file="",
    )
    monkeypatch.setattr(
        budget_handlers,
        "evaluate_budget",
        lambda _repo, _metric, fail_on_warn=False, top_n=10: {
            "schema_version": 1,
            "tool": "atlasctl",
            "metric": "entries-per-dir",
            "status": "ok",
            "fail_count": 0,
            "warn_count": 0,
            "items": [],
            "culprits": [],
        },
    )
    code = policies_command.run_policies_command(_ctx(tmp_path), ns)
    assert code == 0
    assert '"metric": "entries-per-dir"' in capsys.readouterr().out
