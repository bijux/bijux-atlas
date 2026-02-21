from __future__ import annotations

from dataclasses import dataclass
from fnmatch import fnmatch
from pathlib import Path
from typing import Any

try:
    import tomllib  # py311+
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib  # type: ignore


@dataclass(frozen=True)
class BudgetRule:
    name: str
    path_glob: str
    enforce: bool
    max_py_files_per_dir: int
    max_modules_per_dir: int
    max_total_loc_per_dir: int
    max_total_bytes_per_dir: int


@dataclass(frozen=True)
class BudgetException:
    path: str
    reason: str


@dataclass(frozen=True)
class DirStat:
    dir: str
    py_files: int
    modules: int
    total_loc: int
    total_bytes: int
    top_offenders: list[dict[str, Any]]
    rule: str
    enforce: bool
    budget: dict[str, int]


def _load_pyproject(repo_root: Path) -> dict[str, Any]:
    pyproject = repo_root / "packages/atlasctl/pyproject.toml"
    return tomllib.loads(pyproject.read_text(encoding="utf-8"))


def load_budgets(repo_root: Path) -> tuple[dict[str, int], list[BudgetRule], list[BudgetException]]:
    data = _load_pyproject(repo_root)
    budgets = data.get("tool", {}).get("atlasctl", {}).get("budgets", {})
    defaults = {
        "max_py_files_per_dir": int(budgets.get("max_py_files_per_dir", 10)),
        "max_modules_per_dir": int(budgets.get("max_modules_per_dir", budgets.get("max_py_files_per_dir", 10))),
        "max_total_loc_per_dir": int(budgets.get("max_total_loc_per_dir", 3000)),
        "max_total_bytes_per_dir": int(budgets.get("max_total_bytes_per_dir", 300000)),
    }
    rules: list[BudgetRule] = []
    for row in budgets.get("rules", []):
        if not isinstance(row, dict):
            continue
        rules.append(
            BudgetRule(
                name=str(row.get("name", "rule")),
                path_glob=str(row.get("path_glob", "packages/**")),
                enforce=bool(row.get("enforce", True)),
                max_py_files_per_dir=int(row.get("max_py_files_per_dir", defaults["max_py_files_per_dir"])),
                max_modules_per_dir=int(row.get("max_modules_per_dir", defaults["max_modules_per_dir"])),
                max_total_loc_per_dir=int(row.get("max_total_loc_per_dir", defaults["max_total_loc_per_dir"])),
                max_total_bytes_per_dir=int(row.get("max_total_bytes_per_dir", defaults["max_total_bytes_per_dir"])),
            )
        )
    exceptions: list[BudgetException] = []
    for row in budgets.get("exceptions", []):
        if not isinstance(row, dict):
            continue
        exceptions.append(BudgetException(path=str(row.get("path", "")).strip(), reason=str(row.get("reason", "")).strip()))
    return defaults, rules, exceptions


def _rule_for_dir(rel_dir: str, defaults: dict[str, int], rules: list[BudgetRule], exceptions: list[BudgetException]) -> tuple[str, bool, dict[str, int]]:
    for exc in exceptions:
        if exc.path == rel_dir:
            return ("exception", False, defaults)
    matches = [rule for rule in rules if fnmatch(rel_dir, rule.path_glob)]
    if not matches:
        return ("default", True, defaults)
    rule = sorted(matches, key=lambda item: len(item.path_glob), reverse=True)[0]
    return (
        rule.name,
        rule.enforce,
        {
            "max_py_files_per_dir": rule.max_py_files_per_dir,
            "max_modules_per_dir": rule.max_modules_per_dir,
            "max_total_loc_per_dir": rule.max_total_loc_per_dir,
            "max_total_bytes_per_dir": rule.max_total_bytes_per_dir,
        },
    )


def collect_dir_stats(repo_root: Path) -> list[DirStat]:
    defaults, rules, exceptions = load_budgets(repo_root)
    files = sorted((repo_root / "packages").rglob("*.py"))
    by_dir: dict[str, list[Path]] = {}
    for path in files:
        rel_dir = path.parent.relative_to(repo_root).as_posix()
        by_dir.setdefault(rel_dir, []).append(path)

    rows: list[DirStat] = []
    for rel_dir, paths in sorted(by_dir.items()):
        offenders = []
        total_loc = 0
        total_bytes = 0
        modules = 0
        for path in sorted(paths):
            text = path.read_text(encoding="utf-8", errors="ignore")
            loc = len(text.splitlines())
            size = path.stat().st_size
            total_loc += loc
            total_bytes += size
            if path.name != "__init__.py":
                modules += 1
            offenders.append({"file": path.relative_to(repo_root).as_posix(), "loc": loc, "bytes": size})
        rule_name, enforce, budget = _rule_for_dir(rel_dir, defaults, rules, exceptions)
        top = sorted(offenders, key=lambda item: (int(item["loc"]), int(item["bytes"])), reverse=True)[:5]
        rows.append(
            DirStat(
                dir=rel_dir,
                py_files=len(paths),
                modules=modules,
                total_loc=total_loc,
                total_bytes=total_bytes,
                top_offenders=top,
                rule=rule_name,
                enforce=enforce,
                budget=budget,
            )
        )
    return rows


def _status(value: int, budget: int) -> str:
    if value > budget:
        return "fail"
    near = int(budget * 0.9)
    if value >= near:
        return "warn"
    return "ok"


def evaluate_metric(repo_root: Path, metric: str) -> dict[str, Any]:
    stats = collect_dir_stats(repo_root)
    budget_key = {
        "modules-per-dir": "max_modules_per_dir",
        "py-files-per-dir": "max_py_files_per_dir",
        "dir-loc": "max_total_loc_per_dir",
    }[metric]
    value_key = {
        "modules-per-dir": "modules",
        "py-files-per-dir": "py_files",
        "dir-loc": "total_loc",
    }[metric]

    rows: list[dict[str, Any]] = []
    any_fail = False
    for row in stats:
        budget = int(row.budget[budget_key])
        value = int(getattr(row, value_key))
        status = _status(value, budget) if row.enforce else "ok"
        if status == "fail":
            any_fail = True
        rows.append(
            {
                "dir": row.dir,
                "count": value,
                "budget": budget,
                "status": status,
                "rule": row.rule,
                "enforce": row.enforce,
                "top_offenders": row.top_offenders,
            }
        )

    rows = sorted(rows, key=lambda item: ({"fail": 0, "warn": 1, "ok": 2}[str(item["status"])], str(item["dir"])))
    return {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "fail" if any_fail else "ok",
        "metric": metric,
        "items": rows,
        "failed_count": sum(1 for item in rows if item["status"] == "fail"),
        "warn_count": sum(1 for item in rows if item["status"] == "warn"),
    }


def render_text(payload: dict[str, Any]) -> str:
    metric = str(payload["metric"])
    lines = [f"culprits {metric}: {payload['status']}"]
    for item in payload["items"]:
        status = str(item["status"]).upper()
        if status == "OK":
            continue
        lines.append(f"- {status} {item['dir']}: count={item['count']} budget={item['budget']} rule={item['rule']}")
        if item["top_offenders"]:
            top = ", ".join(f"{o['file']} ({o['loc']} loc)" for o in item["top_offenders"][:3])
            lines.append(f"  top offenders: {top}")
    if len(lines) == 1:
        lines.append("- no offenders")
    return "\n".join(lines)


def suggested_splits(dir_path: str) -> str:
    if "checks/layout" in dir_path:
        return "split into checks/layout/{makefiles,ops,contracts,docs,public_surface,scripts}"
    if dir_path.endswith("/core"):
        return "split by concern into subpackages (runtime/contracts/models/adapters)"
    if "/legacy/" in dir_path:
        return "avoid new code in legacy; migrate to canonical module"
    return "split by domain subpackages and move unrelated modules out"


def check_budget_metric(repo_root: Path, metric: str) -> tuple[int, list[str]]:
    payload = evaluate_metric(repo_root, metric)
    errors: list[str] = []
    for item in payload["items"]:
        status = str(item["status"])
        if status == "fail":
            errors.append(
                f"{item['dir']}: {item['count']} > {item['budget']} ({metric}); suggested split: {suggested_splits(str(item['dir']))}"
            )
        elif status == "warn":
            errors.append(f"WARN: {item['dir']}: {item['count']} is within 10% of budget {item['budget']} ({metric})")
    return (1 if payload["failed_count"] else 0), errors


def check_budget_exceptions_documented(repo_root: Path) -> tuple[int, list[str]]:
    _, _, exceptions = load_budgets(repo_root)
    doc_path = repo_root / "docs/architecture-budgets.md"
    text = doc_path.read_text(encoding="utf-8") if doc_path.exists() else ""
    errors: list[str] = []
    for exc in exceptions:
        if not exc.path:
            errors.append("budget exception with empty path")
            continue
        if f"`{exc.path}`" not in text:
            errors.append(f"budget exception not documented in docs/architecture-budgets.md: {exc.path}")
    return (0 if not errors else 1), errors
