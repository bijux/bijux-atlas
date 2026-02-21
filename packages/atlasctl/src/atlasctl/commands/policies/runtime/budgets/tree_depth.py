from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

from ..culprits import _load_pyproject, load_budgets


@dataclass(frozen=True)
class TreeDepthStat:
    path: str
    depth: int
    budget: int
    status: str
    allowlisted: bool
    reason: str


def _max_tree_depth(repo_root: Path) -> int:
    data = _load_pyproject(repo_root)
    budgets = data.get("tool", {}).get("atlasctl", {}).get("budgets", {})
    configured = budgets.get("max_tree_depth", budgets.get("max_package_depth", 4))
    return int(configured)


def _allowlisted_depth_paths(repo_root: Path) -> dict[str, str]:
    _defaults, _rules, exceptions = load_budgets(repo_root)
    return {exc.path: exc.reason for exc in exceptions if exc.path}


def _is_allowlisted(rel_path: str, allowlist: dict[str, str]) -> tuple[bool, str]:
    for prefix, reason in allowlist.items():
        if rel_path == prefix or rel_path.startswith(f"{prefix}/"):
            return True, reason
    return False, ""


def _status(value: int, budget: int, allowlisted: bool) -> str:
    if allowlisted:
        return "ok"
    if value > budget:
        return "fail"
    if budget < 10:
        return "ok"
    if value >= int(budget * 0.9):
        return "warn"
    return "ok"


def collect_tree_depth_stats(repo_root: Path) -> list[TreeDepthStat]:
    root = repo_root / "packages/atlasctl/src/atlasctl"
    if not root.exists():
        return []
    budget = _max_tree_depth(repo_root)
    allowlist = _allowlisted_depth_paths(repo_root)
    rows: list[TreeDepthStat] = []
    for path in sorted(root.rglob("*")):
        if not path.is_dir() or "__pycache__" in path.parts:
            continue
        rel = path.relative_to(repo_root).as_posix()
        depth = len(path.relative_to(root).parts)
        allowlisted, reason = _is_allowlisted(rel, allowlist)
        rows.append(
            TreeDepthStat(
                path=rel,
                depth=depth,
                budget=budget,
                status=_status(depth, budget, allowlisted),
                allowlisted=allowlisted,
                reason=reason,
            )
        )
    return sorted(rows, key=lambda row: ({"fail": 0, "warn": 1, "ok": 2}[row.status], -row.depth, row.path))


def check_tree_depth(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    failures = 0
    for row in collect_tree_depth_stats(repo_root):
        if row.status == "fail":
            failures += 1
            errors.append(f"{row.path}: depth {row.depth} > {row.budget} (tree-depth)")
        elif row.status == "warn":
            errors.append(f"WARN: {row.path}: depth {row.depth} within 10% of budget {row.budget} (tree-depth)")
    return (1 if failures else 0), errors


def deepest_paths(repo_root: Path, *, limit: int = 20) -> dict[str, Any]:
    items = [
        {
            "path": row.path,
            "depth": row.depth,
            "budget": row.budget,
            "status": row.status,
            "allowlisted": row.allowlisted,
            "reason": row.reason,
        }
        for row in collect_tree_depth_stats(repo_root)
    ][:limit]
    return {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "fail" if any(item["status"] == "fail" for item in items) else "ok",
        "metric": "tree-depth",
        "items": items,
        "failed_count": sum(1 for item in items if item["status"] == "fail"),
        "warn_count": sum(1 for item in items if item["status"] == "warn"),
    }
