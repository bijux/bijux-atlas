from __future__ import annotations

from pathlib import Path
from typing import Any

from ..culprits import _rule_for_dir, load_budgets

_SCOPE_ROOTS = (
    "packages/atlasctl/src/atlasctl",
    "packages/atlasctl/tests",
)
_SPLIT_DOC = "packages/atlasctl/docs/architecture/how-to-split-a-module.md"


def _iter_scoped_dirs(repo_root: Path) -> list[Path]:
    dirs: list[Path] = []
    for rel_root in _SCOPE_ROOTS:
        root = repo_root / rel_root
        if not root.exists():
            continue
        dirs.append(root)
        for path in sorted(root.rglob("*")):
            if path.is_dir() and path.name != "__pycache__":
                dirs.append(path)
    return sorted({p for p in dirs})


def _entry_count(directory: Path) -> int:
    return sum(1 for p in directory.iterdir() if p.name != "__pycache__")


def _py_file_count(directory: Path) -> int:
    count = 0
    for path in directory.iterdir():
        if path.name == "__pycache__":
            continue
        if path.is_file() and path.suffix == ".py" and path.name != "__init__.py":
            count += 1
    return count


def _status_for(value: int, budget: int) -> str:
    if value > budget:
        return "fail"
    near = int(budget * 0.9)
    if value >= near:
        return "warn"
    return "ok"


def _budget_profile_errors(repo_root: Path) -> list[str]:
    defaults, rules, _exceptions = load_budgets(repo_root)
    globs = [rule.path_glob for rule in rules]
    src_root = repo_root / "packages/atlasctl/src/atlasctl"
    if not src_root.exists():
        return []
    errors: list[str] = []
    for domain_dir in sorted(path for path in src_root.iterdir() if path.is_dir() and path.name != "__pycache__"):
        rel = domain_dir.relative_to(repo_root).as_posix()
        domain_glob = f"{rel}*"
        covered = domain_glob in globs or "packages/atlasctl/src/atlasctl/*" in globs
        if not covered:
            errors.append(f"top-level domain missing budget profile rule: {rel}")
    if "packages/atlasctl/tests/*" not in globs and "packages/atlasctl/tests/**" not in globs:
        errors.append("tests budget profile missing: add rule for packages/atlasctl/tests/*")
    if int(defaults.get("max_entries_per_dir", 0)) < 1:
        errors.append("invalid max_entries_per_dir default in [tool.atlasctl.budgets]")
    return errors


def _metric_keys(metric: str) -> tuple[str, str]:
    if metric == "entries-per-dir":
        return "max_entries_per_dir", "entries"
    return "max_py_files_per_dir", "py_files"


def _collect_budget_rows(repo_root: Path, metric: str) -> list[dict[str, Any]]:
    defaults, rules, exceptions = load_budgets(repo_root)
    errors = _budget_profile_errors(repo_root)
    budget_key, _value_key = _metric_keys(metric)
    rows: list[dict[str, Any]] = []
    for directory in _iter_scoped_dirs(repo_root):
        rel = directory.relative_to(repo_root).as_posix()
        _rule_name, enforce, budget = _rule_for_dir(rel, defaults, rules, exceptions)
        count = _entry_count(directory) if metric == "entries-per-dir" else _py_file_count(directory)
        limit = int(budget[budget_key])
        status = _status_for(count, limit) if enforce else "ok"
        rows.append(
            {
                "path": rel,
                "count": count,
                "budget": limit,
                "status": status,
                "severity": "warning" if status == "warn" else "error" if status == "fail" else "ok",
                "enforced": enforce,
            }
        )
    rows = sorted(rows, key=lambda r: ({"fail": 0, "warn": 1, "ok": 2}[str(r["status"])], -int(r["count"]), str(r["path"])))
    if errors:
        rows.insert(
            0,
            {
                "path": "_budget_profile_",
                "count": len(errors),
                "budget": 0,
                "status": "fail",
                "severity": "error",
                "enforced": True,
                "errors": errors,
            },
        )
    return rows


def evaluate_budget(repo_root: Path, metric: str, *, fail_on_warn: bool = False, top_n: int = 10) -> dict[str, Any]:
    rows = _collect_budget_rows(repo_root, metric)
    fails = [r for r in rows if r["status"] == "fail"]
    warns = [r for r in rows if r["status"] == "warn"]
    status = "fail" if fails or (fail_on_warn and warns) else "ok"
    return {
        "schema_version": 1,
        "tool": "atlasctl",
        "metric": metric,
        "status": status,
        "fail_count": len(fails),
        "warn_count": len(warns),
        "items": rows,
        "culprits": rows[: max(0, int(top_n))],
    }


def render_budget_text(payload: dict[str, Any], *, print_culprits: bool = False) -> str:
    lines = [f"{payload['metric']}: {payload['status']} (fail={payload['fail_count']} warn={payload['warn_count']})"]
    items = payload["culprits"] if print_culprits else payload["items"]
    for row in items:
        if row["status"] == "ok":
            continue
        path = row["path"]
        lines.append(
            f"- {str(row['status']).upper()} {path}: count={row['count']} budget={row['budget']} "
            f"(suggested split: {_SPLIT_DOC})"
        )
        if "errors" in row:
            for err in row["errors"]:
                lines.append(f"  - {err}")
    if len(lines) == 1:
        lines.append("- no offenders")
    return "\n".join(lines)


def _domain_from_path(path: str) -> str:
    src_prefix = "packages/atlasctl/src/atlasctl/"
    tests_prefix = "packages/atlasctl/tests/"
    if path.startswith(src_prefix):
        parts = path[len(src_prefix) :].split("/")
        return parts[0] if parts and parts[0] else "src"
    if path.startswith(tests_prefix):
        parts = path[len(tests_prefix) :].split("/")
        return f"tests:{parts[0]}" if parts and parts[0] else "tests"
    return "other"


def report_budgets(repo_root: Path, *, by_domain: bool = False, top_n: int = 25) -> dict[str, Any]:
    entry = evaluate_budget(repo_root, "entries-per-dir", top_n=top_n)
    py = evaluate_budget(repo_root, "py-files-per-dir", top_n=top_n)
    if not by_domain:
        return {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "fail" if entry["status"] == "fail" or py["status"] == "fail" else "ok",
            "reports": [entry, py],
        }
    buckets: dict[str, dict[str, int]] = {}
    for payload in (entry, py):
        metric = str(payload["metric"])
        for row in payload["items"]:
            domain = _domain_from_path(str(row["path"]))
            b = buckets.setdefault(domain, {"entry_fail": 0, "entry_warn": 0, "py_fail": 0, "py_warn": 0})
            if metric == "entries-per-dir":
                if row["status"] == "fail":
                    b["entry_fail"] += 1
                elif row["status"] == "warn":
                    b["entry_warn"] += 1
            else:
                if row["status"] == "fail":
                    b["py_fail"] += 1
                elif row["status"] == "warn":
                    b["py_warn"] += 1
    return {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "fail" if entry["status"] == "fail" or py["status"] == "fail" else "ok",
        "by_domain": [
            {"domain": domain, **vals}
            for domain, vals in sorted(
                buckets.items(),
                key=lambda kv: (-(kv[1]["entry_fail"] + kv[1]["py_fail"]), kv[0]),
            )
        ],
    }


def check_dir_entry_budgets(repo_root: Path, *, fail_on_warn: bool = False) -> tuple[int, list[str]]:
    payload = evaluate_budget(repo_root, "entries-per-dir", fail_on_warn=fail_on_warn)
    if payload["status"] == "ok":
        return 0, []
    return 1, [render_budget_text(payload, print_culprits=True)]


def check_py_files_per_dir_budget(repo_root: Path, *, fail_on_warn: bool = False) -> tuple[int, list[str]]:
    payload = evaluate_budget(repo_root, "py-files-per-dir", fail_on_warn=fail_on_warn)
    if payload["status"] == "ok":
        return 0, []
    return 1, [render_budget_text(payload, print_culprits=True)]
