from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .culprits import _load_pyproject


@dataclass(frozen=True)
class DomainModuleStat:
    domain: str
    modules: int
    budget: int
    status: str


def _max_modules_per_domain(repo_root: Path) -> int:
    data = _load_pyproject(repo_root)
    budgets = data.get("tool", {}).get("atlasctl", {}).get("budgets", {})
    configured = budgets.get("max_modules_per_domain", budgets.get("max_modules_per_dir", 10))
    return int(configured)


def _status(value: int, budget: int) -> str:
    if value > budget:
        return "fail"
    if budget < 10:
        return "ok"
    if value >= int(budget * 0.9):
        return "warn"
    return "ok"


def collect_domain_module_stats(repo_root: Path) -> list[DomainModuleStat]:
    root = repo_root / "packages/atlasctl/src/atlasctl"
    if not root.exists():
        return []
    budget = _max_modules_per_domain(repo_root)
    rows: list[DomainModuleStat] = []
    for domain in sorted(path for path in root.iterdir() if path.is_dir() and path.name != "__pycache__"):
        modules = 0
        for py in domain.rglob("*.py"):
            if "__pycache__" in py.parts or py.name == "__init__.py":
                continue
            modules += 1
        rows.append(
            DomainModuleStat(
                domain=domain.relative_to(repo_root).as_posix(),
                modules=modules,
                budget=budget,
                status=_status(modules, budget),
            )
        )
    return sorted(rows, key=lambda row: ({"fail": 0, "warn": 1, "ok": 2}[row.status], -row.modules, row.domain))


def check_modules_per_domain(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    failures = 0
    for row in collect_domain_module_stats(repo_root):
        if row.status == "fail":
            failures += 1
            errors.append(f"{row.domain}: modules {row.modules} > budget {row.budget} (modules-per-domain)")
        elif row.status == "warn":
            errors.append(f"WARN: {row.domain}: modules {row.modules} within 10% of budget {row.budget} (modules-per-domain)")
    return (1 if failures else 0), errors


def worst_domains_by_modules(repo_root: Path, *, limit: int = 10) -> dict[str, Any]:
    items = [
        {
            "domain": row.domain,
            "modules": row.modules,
            "budget": row.budget,
            "status": row.status,
        }
        for row in collect_domain_module_stats(repo_root)
    ][:limit]
    return {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "fail" if any(item["status"] == "fail" for item in items) else "ok",
        "metric": "modules-per-domain",
        "items": items,
        "failed_count": sum(1 for item in items if item["status"] == "fail"),
        "warn_count": sum(1 for item in items if item["status"] == "warn"),
    }
