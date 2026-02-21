from __future__ import annotations

import json
from dataclasses import dataclass
from datetime import date
from pathlib import Path
from typing import Any


_SCOPE_ROOTS = (
    "packages/atlasctl/src/atlasctl",
    "packages/atlasctl/tests",
)
_DEFAULT_MAX_DIR_ENTRIES = 10
_DEFAULT_MAX_PY_FILES = 10
_EXCEPTIONS_PATH = Path("configs/policy/BUDGET_EXCEPTIONS.yml")
_SPLIT_DOC = "packages/atlasctl/docs/architecture/how-to-split-a-module.md"


@dataclass(frozen=True)
class BudgetException:
    path: str
    owner: str
    reason: str
    expires_on: str


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
    return sorted({p.resolve() for p in dirs})


def _load_exceptions(repo_root: Path) -> tuple[int, list[BudgetException], list[str]]:
    path = repo_root / _EXCEPTIONS_PATH
    if not path.exists():
        return 3, [], [f"missing exceptions file: {_EXCEPTIONS_PATH.as_posix()}"]
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        return 3, [], [f"invalid exceptions file (must be JSON-compatible YAML): {exc}"]
    max_ex = int(payload.get("max_exceptions", 3))
    rows: list[BudgetException] = []
    errors: list[str] = []
    today = date.today()
    for idx, item in enumerate(payload.get("exceptions", []), start=1):
        if not isinstance(item, dict):
            errors.append(f"exceptions[{idx}] must be an object")
            continue
        row = BudgetException(
            path=str(item.get("path", "")).strip(),
            owner=str(item.get("owner", "")).strip(),
            reason=str(item.get("reason", "")).strip(),
            expires_on=str(item.get("expires_on", "")).strip(),
        )
        if not row.path:
            errors.append(f"exceptions[{idx}] missing path")
            continue
        if not row.owner:
            errors.append(f"{row.path}: exception missing owner")
        if not row.reason:
            errors.append(f"{row.path}: exception missing reason")
        try:
            exp = date.fromisoformat(row.expires_on)
            if exp < today:
                errors.append(f"{row.path}: exception expired on {row.expires_on}")
        except ValueError:
            errors.append(f"{row.path}: invalid expires_on `{row.expires_on}`")
        rows.append(row)
    if len(rows) > max_ex:
        errors.append(f"exception count exceeded: {len(rows)} > {max_ex}")
    return max_ex, rows, errors


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


def _collect_budget_rows(repo_root: Path, metric: str) -> list[dict[str, Any]]:
    max_ex, exceptions, errors = _load_exceptions(repo_root)
    exc_map = {exc.path: exc for exc in exceptions}
    rows: list[dict[str, Any]] = []
    for directory in _iter_scoped_dirs(repo_root):
        rel = directory.relative_to(repo_root).as_posix()
        budget = _DEFAULT_MAX_DIR_ENTRIES if metric == "entries-per-dir" else _DEFAULT_MAX_PY_FILES
        count = _entry_count(directory) if metric == "entries-per-dir" else _py_file_count(directory)
        status = _status_for(count, budget)
        exc = exc_map.get(rel)
        enforced = exc is None
        if not enforced:
            status = "ok"
        rows.append(
            {
                "path": rel,
                "count": count,
                "budget": budget,
                "status": status,
                "severity": "warning" if status == "warn" else "error" if status == "fail" else "ok",
                "enforced": enforced,
                "exception": None
                if exc is None
                else {"owner": exc.owner, "reason": exc.reason, "expires_on": exc.expires_on},
            }
        )
    rows = sorted(rows, key=lambda r: ({"fail": 0, "warn": 1, "ok": 2}[str(r["status"])], -int(r["count"]), str(r["path"])))
    if errors:
        rows.insert(
            0,
            {
                "path": "_exceptions_",
                "count": len(exceptions),
                "budget": max_ex,
                "status": "fail",
                "severity": "error",
                "enforced": True,
                "exception": None,
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

