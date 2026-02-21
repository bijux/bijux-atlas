from __future__ import annotations

from pathlib import Path


def check_suite_inventory_policy(repo_root: Path) -> tuple[int, list[str]]:
    from ....suite.command import load_suites, suite_inventory_violations

    _, suites = load_suites(repo_root)
    errors = suite_inventory_violations(suites)
    return (0 if not errors else 1), errors

