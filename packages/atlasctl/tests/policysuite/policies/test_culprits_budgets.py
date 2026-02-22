from __future__ import annotations

# BYPASS_TEST_OK: budget policy tests intentionally reference configs/policy fixtures.
from pathlib import Path

from atlasctl.commands.policies.runtime.culprits import check_budget_drift_approval, check_budget_exceptions_sorted, load_budgets


def _repo_with_pyproject(tmp_path: Path, body: str) -> Path:
    pyproject = tmp_path / "packages/atlasctl/pyproject.toml"
    pyproject.parent.mkdir(parents=True, exist_ok=True)
    pyproject.write_text(body, encoding="utf-8")
    return tmp_path


def test_budget_config_parses_and_rule_overrides_apply(tmp_path: Path) -> None:
    repo = _repo_with_pyproject(
        tmp_path,
        """
[tool.atlasctl.budgets]
max_py_files_per_dir = 10
max_modules_per_dir = 9
max_loc_per_file = 500
max_loc_per_dir = 2000

[[tool.atlasctl.budgets.rules]]
name = "core_strict"
path_glob = "packages/atlasctl/src/atlasctl/core*"
enforce = true
max_modules_per_dir = 7
max_loc_per_file = 420
max_loc_per_dir = 1500
""".strip()
        + "\n",
    )
    defaults, rules, exceptions = load_budgets(repo)
    assert defaults["max_loc_per_file"] == 500
    assert defaults["max_loc_per_dir"] == 2000
    assert len(rules) == 1
    assert rules[0].name == "core_strict"
    assert rules[0].max_modules_per_dir == 7
    assert rules[0].max_loc_per_file == 420
    assert rules[0].max_loc_per_dir == 1500
    assert exceptions == []


def test_budget_exceptions_must_be_sorted(tmp_path: Path) -> None:
    repo = _repo_with_pyproject(
        tmp_path,
        """
[tool.atlasctl.budgets]
max_py_files_per_dir = 10
max_modules_per_dir = 10
max_loc_per_file = 600
max_loc_per_dir = 2600

[[tool.atlasctl.budgets.exceptions]]
path = "z/path"
reason = "z"

[[tool.atlasctl.budgets.exceptions]]
path = "a/path"
reason = "a"
""".strip()
        + "\n",
    )
    code, errors = check_budget_exceptions_sorted(repo)
    assert code == 1
    assert errors == ["budget exceptions in pyproject must be sorted by path"]


def test_budget_drift_requires_approval_marker_when_loosened(tmp_path: Path) -> None:
    repo = _repo_with_pyproject(
        tmp_path,
        """
[tool.atlasctl.budgets]
max_py_files_per_dir = 12
max_modules_per_dir = 10
max_loc_per_file = 600
max_loc_per_dir = 2600
""".strip()
        + "\n",
    )
    baseline = repo / "configs/policy/atlasctl-budgets-baseline.json"
    baseline.parent.mkdir(parents=True, exist_ok=True)
    baseline.write_text(
        '{"max_py_files_per_dir": 10, "max_modules_per_dir": 10, "max_loc_per_file": 600, "max_loc_per_dir": 2600}\n',
        encoding="utf-8",
    )
    code, errors = check_budget_drift_approval(repo)
    assert code == 1
    assert any("budget loosened without approval marker file" in err for err in errors)
