from __future__ import annotations

import ast
from dataclasses import dataclass
from fnmatch import fnmatch
from pathlib import Path
from typing import Any

try:
    import tomllib  # py311+
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib  # type: ignore


_BRANCH_TOKENS = (" if ", " elif ", " for ", " while ", " except ", " case ", " and ", " or ")


@dataclass(frozen=True)
class BudgetRule:
    name: str
    path_glob: str
    enforce: bool
    max_py_files_per_dir: int
    max_modules_per_dir: int
    max_shell_files_per_dir: int
    max_loc_per_file: int
    max_loc_per_dir: int
    max_total_bytes_per_dir: int
    max_imports_per_file: int
    max_public_symbols_per_module: int
    max_branch_keywords_per_file: int


@dataclass(frozen=True)
class BudgetException:
    path: str
    reason: str


@dataclass(frozen=True)
class DirStat:
    dir: str
    py_files: int
    modules: int
    shell_files: int
    total_loc: int
    total_bytes: int
    top_offenders: list[dict[str, Any]]
    rule: str
    enforce: bool
    budget: dict[str, int]


@dataclass(frozen=True)
class FileStat:
    path: str
    loc: int
    import_count: int
    public_symbols: int
    branch_keywords: int
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
        "max_shell_files_per_dir": int(budgets.get("max_shell_files_per_dir", 10)),
        "max_loc_per_file": int(budgets.get("max_loc_per_file", 600)),
        "max_loc_per_dir": int(budgets.get("max_loc_per_dir", budgets.get("max_total_loc_per_dir", 3000))),
        "max_total_bytes_per_dir": int(budgets.get("max_total_bytes_per_dir", 300000)),
        "max_imports_per_file": int(budgets.get("max_imports_per_file", 40)),
        "max_public_symbols_per_module": int(budgets.get("max_public_symbols_per_module", 30)),
        "max_branch_keywords_per_file": int(budgets.get("max_branch_keywords_per_file", 80)),
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
                max_shell_files_per_dir=int(row.get("max_shell_files_per_dir", defaults["max_shell_files_per_dir"])),
                max_loc_per_file=int(row.get("max_loc_per_file", defaults["max_loc_per_file"])),
                max_loc_per_dir=int(row.get("max_loc_per_dir", row.get("max_total_loc_per_dir", defaults["max_loc_per_dir"]))),
                max_total_bytes_per_dir=int(row.get("max_total_bytes_per_dir", defaults["max_total_bytes_per_dir"])),
                max_imports_per_file=int(row.get("max_imports_per_file", defaults["max_imports_per_file"])),
                max_public_symbols_per_module=int(
                    row.get("max_public_symbols_per_module", defaults["max_public_symbols_per_module"])
                ),
                max_branch_keywords_per_file=int(
                    row.get("max_branch_keywords_per_file", defaults["max_branch_keywords_per_file"])
                ),
            )
        )
    exceptions: list[BudgetException] = []
    for row in budgets.get("exceptions", []):
        if not isinstance(row, dict):
            continue
        exceptions.append(BudgetException(path=str(row.get("path", "")).strip(), reason=str(row.get("reason", "")).strip()))
    return defaults, rules, exceptions


def _rule_for_dir(
    rel_dir: str, defaults: dict[str, int], rules: list[BudgetRule], exceptions: list[BudgetException]
) -> tuple[str, bool, dict[str, int]]:
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
            "max_shell_files_per_dir": rule.max_shell_files_per_dir,
            "max_loc_per_file": rule.max_loc_per_file,
            "max_loc_per_dir": rule.max_loc_per_dir,
            "max_total_bytes_per_dir": rule.max_total_bytes_per_dir,
            "max_imports_per_file": rule.max_imports_per_file,
            "max_public_symbols_per_module": rule.max_public_symbols_per_module,
            "max_branch_keywords_per_file": rule.max_branch_keywords_per_file,
        },
    )


def _count_imports(tree: ast.AST) -> int:
    count = 0
    for node in ast.walk(tree):
        if isinstance(node, (ast.Import, ast.ImportFrom)):
            count += 1
    return count


def _count_public_symbols(tree: ast.AST) -> int:
    for node in ast.walk(tree):
        if not isinstance(node, ast.Assign):
            continue
        for target in node.targets:
            if not isinstance(target, ast.Name) or target.id != "__all__":
                continue
            if isinstance(node.value, (ast.List, ast.Tuple)):
                return len(node.value.elts)
    return 0


def _branch_score(text: str) -> int:
    lowered = f" {text.lower()} "
    return sum(lowered.count(token) for token in _BRANCH_TOKENS)


def _iter_repo_shell_files(repo_root: Path) -> list[Path]:
    roots = [
        repo_root / "packages/atlasctl/src",
        repo_root / "ops",
    ]
    files: list[Path] = []
    for root in roots:
        if root.exists():
            files.extend(sorted(root.rglob("*.sh")))
    return [path for path in files if ".git/" not in path.as_posix()]


def collect_dir_stats(repo_root: Path) -> list[DirStat]:
    defaults, rules, exceptions = load_budgets(repo_root)
    py_files = sorted((repo_root / "packages").rglob("*.py"))
    shell_files = _iter_repo_shell_files(repo_root)
    by_dir: dict[str, list[Path]] = {}
    for path in py_files:
        rel_dir = path.parent.relative_to(repo_root).as_posix()
        by_dir.setdefault(rel_dir, []).append(path)
    shell_by_dir: dict[str, int] = {}
    for path in shell_files:
        rel_dir = path.parent.relative_to(repo_root).as_posix()
        shell_by_dir[rel_dir] = shell_by_dir.get(rel_dir, 0) + 1
        by_dir.setdefault(rel_dir, [])

    rows: list[DirStat] = []
    for rel_dir, paths in sorted(by_dir.items()):
        offenders: list[dict[str, Any]] = []
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
                shell_files=shell_by_dir.get(rel_dir, 0),
                total_loc=total_loc,
                total_bytes=total_bytes,
                top_offenders=top,
                rule=rule_name,
                enforce=enforce,
                budget=budget,
            )
        )
    return rows


def collect_file_stats(repo_root: Path) -> list[FileStat]:
    defaults, rules, exceptions = load_budgets(repo_root)
    rows: list[FileStat] = []
    for path in sorted((repo_root / "packages").rglob("*.py")):
        text = path.read_text(encoding="utf-8", errors="ignore")
        rel_path = path.relative_to(repo_root).as_posix()
        rel_dir = path.parent.relative_to(repo_root).as_posix()
        tree = ast.parse(text, filename=rel_path)
        rule_name, enforce, budget = _rule_for_dir(rel_dir, defaults, rules, exceptions)
        rows.append(
            FileStat(
                path=rel_path,
                loc=len(text.splitlines()),
                import_count=_count_imports(tree),
                public_symbols=_count_public_symbols(tree),
                branch_keywords=_branch_score(text),
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


def _evaluate_dir_metric(repo_root: Path, metric: str) -> dict[str, Any]:
    stats = collect_dir_stats(repo_root)
    metric_key = "py-files-per-dir" if metric == "files-per-dir" else metric
    budget_key = {
        "modules-per-dir": "max_modules_per_dir",
        "py-files-per-dir": "max_py_files_per_dir",
        "shell-files-per-dir": "max_shell_files_per_dir",
        "dir-loc": "max_loc_per_dir",
    }[metric_key]
    value_key = {
        "modules-per-dir": "modules",
        "py-files-per-dir": "py_files",
        "shell-files-per-dir": "shell_files",
        "dir-loc": "total_loc",
    }[metric_key]

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
        "metric": metric_key,
        "items": rows,
        "failed_count": sum(1 for item in rows if item["status"] == "fail"),
        "warn_count": sum(1 for item in rows if item["status"] == "warn"),
    }


def _evaluate_file_metric(repo_root: Path, metric: str) -> dict[str, Any]:
    stats = collect_file_stats(repo_root)
    budget_key = {
        "imports-per-file": "max_imports_per_file",
        "public-symbols-per-file": "max_public_symbols_per_module",
        "complexity-heuristic": "max_branch_keywords_per_file",
    }[metric]
    value_key = {
        "imports-per-file": "import_count",
        "public-symbols-per-file": "public_symbols",
        "complexity-heuristic": "branch_keywords",
    }[metric]
    scope_prefixes = ("packages/atlasctl/src/atlasctl/core/", "packages/atlasctl/src/atlasctl/cli/")

    rows: list[dict[str, Any]] = []
    any_fail = False
    for row in stats:
        if metric == "complexity-heuristic" and not row.path.startswith(scope_prefixes):
            continue
        budget = int(row.budget[budget_key])
        value = int(getattr(row, value_key))
        status = _status(value, budget) if row.enforce else "ok"
        if status == "fail":
            any_fail = True
        rows.append(
            {
                "file": row.path,
                "count": value,
                "budget": budget,
                "status": status,
                "rule": row.rule,
                "enforce": row.enforce,
            }
        )
    rows = sorted(
        rows,
        key=lambda item: ({"fail": 0, "warn": 1, "ok": 2}[str(item["status"])], -int(item["count"]), str(item["file"])),
    )
    return {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "fail" if any_fail else "ok",
        "metric": metric,
        "items": rows,
        "failed_count": sum(1 for item in rows if item["status"] == "fail"),
        "warn_count": sum(1 for item in rows if item["status"] == "warn"),
    }


def evaluate_metric(repo_root: Path, metric: str) -> dict[str, Any]:
    if metric in {"modules-per-dir", "py-files-per-dir", "files-per-dir", "shell-files-per-dir", "dir-loc", "loc-per-dir"}:
        if metric == "loc-per-dir":
            metric = "dir-loc"
        return _evaluate_dir_metric(repo_root, metric)
    if metric in {"imports-per-file", "public-symbols-per-file", "complexity-heuristic"}:
        return _evaluate_file_metric(repo_root, metric)
    if metric in {"largest-files", "biggest-files"}:
        return biggest_files(repo_root, limit=20)
    raise ValueError(f"unsupported culprits metric: {metric}")


def biggest_files(repo_root: Path, limit: int = 20) -> dict[str, Any]:
    rows = []
    defaults, rules, exceptions = load_budgets(repo_root)
    any_fail = False
    for path in sorted((repo_root / "packages").rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        rel_dir = path.parent.relative_to(repo_root).as_posix()
        rule_name, enforce, budget = _rule_for_dir(rel_dir, defaults, rules, exceptions)
        loc = len(text.splitlines())
        loc_budget = int(budget["max_loc_per_file"])
        status = _status(loc, loc_budget) if enforce else "ok"
        if status == "fail":
            any_fail = True
        rows.append(
            {
                "file": rel,
                "loc": loc,
                "bytes": path.stat().st_size,
                "budget": loc_budget,
                "status": status,
                "rule": rule_name,
                "enforce": enforce,
            }
        )
    rows = sorted(
        rows,
        key=lambda item: ({"fail": 0, "warn": 1, "ok": 2}[str(item["status"])], -int(item["loc"]), -int(item["bytes"]), str(item["file"])),
    )[:limit]
    return {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "fail" if any_fail else "ok",
        "metric": "largest-files",
        "items": rows,
        "failed_count": sum(1 for item in rows if item["status"] == "fail"),
        "warn_count": sum(1 for item in rows if item["status"] == "warn"),
    }


def biggest_dirs(repo_root: Path, limit: int = 20) -> dict[str, Any]:
    rows = collect_dir_stats(repo_root)
    items = [
        {"dir": row.dir, "loc": row.total_loc, "py_files": row.py_files, "modules": row.modules, "shell_files": row.shell_files}
        for row in rows
    ]
    items = sorted(items, key=lambda item: (int(item["loc"]), int(item["py_files"]), str(item["dir"])), reverse=True)[:limit]
    return {"schema_version": 1, "tool": "atlasctl", "status": "ok", "metric": "biggest-dirs", "items": items}


def budget_suite(repo_root: Path) -> dict[str, Any]:
    metrics = ["modules-per-dir", "py-files-per-dir", "shell-files-per-dir", "dir-loc"]
    reports = [evaluate_metric(repo_root, metric) for metric in metrics]
    status = "ok"
    for report in reports:
        if report["status"] != "ok":
            status = "fail"
            break
    return {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": status,
        "metric": "budget-suite",
        "reports": reports,
    }


def explain_budgets(repo_root: Path) -> dict[str, Any]:
    defaults, rules, exceptions = load_budgets(repo_root)
    return {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok",
        "defaults": defaults,
        "rules": [
            {
                "name": rule.name,
                "path_glob": rule.path_glob,
                "enforce": rule.enforce,
                "max_py_files_per_dir": rule.max_py_files_per_dir,
                "max_modules_per_dir": rule.max_modules_per_dir,
                "max_shell_files_per_dir": rule.max_shell_files_per_dir,
                "max_loc_per_file": rule.max_loc_per_file,
                "max_loc_per_dir": rule.max_loc_per_dir,
            }
            for rule in sorted(rules, key=lambda row: row.name)
        ],
        "exceptions": [
            {"path": exc.path, "reason": exc.reason}
            for exc in sorted(exceptions, key=lambda row: row.path)
        ],
    }


def render_text(payload: dict[str, Any]) -> str:
    metric = str(payload["metric"])
    lines = [f"culprits {metric}: {payload['status']}"]
    item_key = "dir" if metric in {"modules-per-dir", "py-files-per-dir", "shell-files-per-dir", "dir-loc"} else "file"
    for item in payload["items"]:
        status = str(item.get("status", "ok")).upper()
        if status == "OK":
            continue
        count = item.get("count", item.get("loc", 0))
        lines.append(
            f"- {status} {item[item_key]}: count={count} budget={item['budget']} rule={item.get('rule', 'default')}"
        )
        top_offenders = item.get("top_offenders", [])
        if top_offenders:
            top = ", ".join(f"{o['file']} ({o['loc']} loc)" for o in top_offenders[:3])
            lines.append(f"  top offenders: {top}")
    if len(lines) == 1:
        lines.append("- no offenders")
    return "\n".join(lines)


def render_table_text(payload: dict[str, Any], label: str) -> str:
    lines = [f"{label}: {payload['status']}"]
    for item in payload.get("items", []):
        if "file" in item:
            lines.append(f"- {item['file']}: {item.get('loc', item.get('count', 0))}")
        elif "dir" in item:
            lines.append(f"- {item['dir']}: {item.get('loc', item.get('count', 0))}")
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
        label = str(item.get("dir", item.get("file", "")))
        if status == "fail":
            errors.append(
                f"{label}: {item['count']} > {item['budget']} ({metric}); suggested split: {suggested_splits(label)}"
            )
        elif status == "warn":
            errors.append(f"WARN: {label}: {item['count']} is within 10% of budget {item['budget']} ({metric})")
    return (1 if payload["failed_count"] else 0), errors


def check_budget_exceptions_documented(repo_root: Path) -> tuple[int, list[str]]:
    _, _, exceptions = load_budgets(repo_root)
    doc_path = repo_root / "packages/atlasctl/docs/architecture.md"
    text = doc_path.read_text(encoding="utf-8") if doc_path.exists() else ""
    errors: list[str] = []
    for exc in exceptions:
        if not exc.path:
            errors.append("budget exception with empty path")
            continue
        if f"`{exc.path}`" not in text:
            errors.append(f"budget exception not documented in packages/atlasctl/docs/architecture.md: {exc.path}")
    return (0 if not errors else 1), errors


def check_budget_exceptions_sorted(repo_root: Path) -> tuple[int, list[str]]:
    _, _, exceptions = load_budgets(repo_root)
    paths = [exc.path for exc in exceptions if exc.path]
    if paths == sorted(paths):
        return 0, []
    return 1, ["budget exceptions in pyproject must be sorted by path"]


def check_budget_drift_approval(repo_root: Path) -> tuple[int, list[str]]:
    baseline_path = repo_root / "configs/policy/atlasctl-budgets-baseline.json"
    if not baseline_path.exists():
        return 1, [f"missing budget baseline file: {baseline_path.relative_to(repo_root).as_posix()}"]
    import json

    baseline = json.loads(baseline_path.read_text(encoding="utf-8"))
    current_defaults, _, _ = load_budgets(repo_root)
    loosened: list[str] = []
    keys = ("max_py_files_per_dir", "max_modules_per_dir", "max_loc_per_file", "max_loc_per_dir")
    for key in keys:
        prev = int(baseline.get(key, current_defaults.get(key, 0)))
        cur = int(current_defaults.get(key, 0))
        if cur > prev:
            loosened.append(f"{key}: baseline={prev} current={cur}")
    if not loosened:
        return 0, []
    approval_path = repo_root / "configs/policy/budget-loosening-approval.json"
    if not approval_path.exists():
        return 1, [f"budget loosened without approval marker file: {approval_path.relative_to(repo_root).as_posix()}", *loosened]
    approval = json.loads(approval_path.read_text(encoding="utf-8"))
    if approval.get("approved") is not True:
        return 1, [f"budget loosened but approval marker not approved: {approval_path.relative_to(repo_root).as_posix()}", *loosened]
    if not str(approval.get("approval_id", "")).strip():
        return 1, [f"budget loosened but approval_id missing in {approval_path.relative_to(repo_root).as_posix()}", *loosened]
    return 0, []


def check_critical_dir_count_trend(repo_root: Path) -> tuple[int, list[str]]:
    baseline_path = repo_root / "configs/policy/atlasctl-dir-count-baseline.json"
    if not baseline_path.exists():
        return 1, [f"missing dir-count baseline file: {baseline_path.relative_to(repo_root).as_posix()}"]
    import json

    baseline = json.loads(baseline_path.read_text(encoding="utf-8"))
    expected = baseline.get("critical_dirs", {})
    if not isinstance(expected, dict):
        return 1, [f"invalid critical_dirs in baseline: {baseline_path.relative_to(repo_root).as_posix()}"]
    stats = {row.dir: row for row in collect_dir_stats(repo_root)}
    errors: list[str] = []
    for rel_dir, limits in sorted(expected.items()):
        if not isinstance(limits, dict):
            errors.append(f"{rel_dir}: invalid baseline row")
            continue
        row = stats.get(rel_dir)
        if row is None:
            errors.append(f"{rel_dir}: directory missing from current stats")
            continue
        max_py = int(limits.get("max_py_files", row.py_files))
        max_modules = int(limits.get("max_modules", row.modules))
        if row.py_files > max_py:
            errors.append(f"{rel_dir}: py_files drifted up {row.py_files} > {max_py}")
        if row.modules > max_modules:
            errors.append(f"{rel_dir}: modules drifted up {row.modules} > {max_modules}")
    return (0 if not errors else 1), errors
