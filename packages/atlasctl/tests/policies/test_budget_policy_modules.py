from __future__ import annotations

from pathlib import Path

from atlasctl.policies.module_budgets import check_modules_per_domain
from atlasctl.policies.tree_depth import check_tree_depth


def _write_pyproject(repo: Path, body: str) -> None:
    pyproject = repo / "packages/atlasctl/pyproject.toml"
    pyproject.parent.mkdir(parents=True, exist_ok=True)
    pyproject.write_text(body, encoding="utf-8")


def test_modules_per_domain_budget_fails_when_exceeded(tmp_path: Path) -> None:
    _write_pyproject(
        tmp_path,
        "\n".join(
            (
                "[tool.atlasctl.budgets]",
                "max_modules_per_domain = 1",
            )
        )
        + "\n",
    )
    root = tmp_path / "packages/atlasctl/src/atlasctl/docs"
    root.mkdir(parents=True, exist_ok=True)
    (root / "a.py").write_text("x = 1\n", encoding="utf-8")
    (root / "b.py").write_text("y = 2\n", encoding="utf-8")

    code, errors = check_modules_per_domain(tmp_path)
    assert code == 1
    assert any("modules-per-domain" in error for error in errors)


def test_tree_depth_uses_exception_allowlist_with_reason(tmp_path: Path) -> None:
    _write_pyproject(
        tmp_path,
        "\n".join(
            (
                "[tool.atlasctl.budgets]",
                "max_tree_depth = 2",
                "",
                "[[tool.atlasctl.budgets.exceptions]]",
                'path = "packages/atlasctl/src/atlasctl/docs/deep"',
                'reason = "temporary transition"',
            )
        )
        + "\n",
    )
    deep_dir = tmp_path / "packages/atlasctl/src/atlasctl/docs/deep/a"
    deep_dir.mkdir(parents=True, exist_ok=True)

    code, errors = check_tree_depth(tmp_path)
    assert code == 0
    assert errors == []
