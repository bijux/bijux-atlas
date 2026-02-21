from __future__ import annotations

from tests.helpers import run_atlasctl


def test_root_help_shows_only_control_plane_groups() -> None:
    proc = run_atlasctl("--help")
    assert proc.returncode == 0, proc.stderr
    text = proc.stdout
    assert "control-plane groups:" in text
    for group in ("docs", "configs", "dev", "ops", "policies", "internal"):
        assert group in text
    assert "k8s" not in text
    assert "contracts-snapshot" not in text


def test_internal_flags_hidden_from_public_help() -> None:
    help_proc = run_atlasctl("help", "--help")
    list_proc = run_atlasctl("list", "--help")
    assert help_proc.returncode == 0, help_proc.stderr
    assert list_proc.returncode == 0, list_proc.stderr
    assert "--include-internal" not in help_proc.stdout
    assert "--include-internal" not in list_proc.stdout


def test_group_help_stable_ordering() -> None:
    proc = run_atlasctl("dev", "--help")
    assert proc.returncode == 0, proc.stderr
    lines = [line.strip() for line in proc.stdout.splitlines()]
    expected = [
        "list                forward to `atlasctl list ...`",
        "check               forward to `atlasctl check ...`",
        "suite               forward to `atlasctl suite ...`",
        "test                forward to `atlasctl test ...`",
        "commands            forward to `atlasctl commands ...`",
        "explain             forward to `atlasctl explain ...`",
    ]
    for line in expected:
        assert line in lines
