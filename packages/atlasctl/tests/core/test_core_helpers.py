from __future__ import annotations

from pathlib import Path

from atlasctl.core.runtime.env_guard import ensure_within_repo, guard_no_network_mode
from atlasctl.core.git import read_git_context
from atlasctl.core.models.models import ContractsIndexModel, OwnershipModel, SurfaceModel
from atlasctl.core.paths import find_repo_root
from atlasctl.core.process import run_command
from atlasctl.core.runtime.tooling import read_pins, read_tool_versions
from atlasctl.core.schema.yaml_utils import validate_yaml_required_keys

ROOT = Path(__file__).resolve().parents[3]


def test_run_command_captures_output_and_duration() -> None:
    res = run_command(["python3", "-c", "print('ok')"], ROOT)
    assert res.code == 0
    assert res.stdout.strip() == "ok"
    assert res.duration_ms >= 0


def test_git_context_present() -> None:
    ctx = read_git_context(ROOT)
    assert ctx.sha
    assert isinstance(ctx.is_dirty, bool)


def test_tooling_and_pins_load() -> None:
    tool_versions = read_tool_versions(ROOT)
    pins = read_pins(ROOT)
    assert "python3" in tool_versions
    assert isinstance(pins, dict)


def test_find_repo_root_from_nested_path() -> None:
    nested = ROOT / "packages" / "atlasctl" / "src"
    assert find_repo_root(nested) == ROOT


def test_yaml_required_keys_validation(tmp_path: Path) -> None:
    cfg = tmp_path / "sample.yaml"
    cfg.write_text("a: 1\nb: 2\n", encoding="utf-8")
    assert validate_yaml_required_keys(cfg, ["a"]) == []
    errs = validate_yaml_required_keys(cfg, ["missing"])
    assert errs and "missing key `missing`" in errs[0]


def test_models_parse_expected_shapes() -> None:
    ownership = OwnershipModel.from_json({"paths": {"ops": "team-a"}, "commands": {"root": "team-b"}})
    assert ownership.paths["ops"] == "team-a"
    surface = SurfaceModel.from_json({"schema_version": 2, "commands": [{"name": "doctor"}]})
    assert surface.schema_version == 2
    contracts = ContractsIndexModel.from_json({"contracts": ["ops/CONTRACT.md"]})
    assert contracts.contracts == ["ops/CONTRACT.md"]


def test_env_helpers(tmp_path: Path) -> None:
    guard_no_network_mode(True)
    assert "BIJUX_SCRIPTS_NO_NETWORK" in __import__("os").environ
    assert ensure_within_repo(ROOT, ROOT / "configs")
    outside = tmp_path / "outside"
    outside.mkdir(parents=True, exist_ok=True)
    assert not ensure_within_repo(ROOT, outside)


def test_run_command_timeout_and_retry() -> None:
    timed = run_command(["python3", "-c", "import time; time.sleep(0.2)"], ROOT, timeout_seconds=1)
    assert timed.code == 0

    flaky = run_command(
        ["python3", "-c", "import sys; sys.exit(1)"],
        ROOT,
        retries=1,
        retry_delay_seconds=0.01,
    )
    assert flaky.code == 1
