from __future__ import annotations

import argparse
import json
from fnmatch import fnmatch
from pathlib import Path
from typing import Any

from ..core.context import RunContext

DEFAULT_ALLOWLIST = Path("configs/layout/scripts-references-allowlist.txt")
EXCLUDE_PREFIXES = (
    ".git/",
    "artifacts/",
    "target/",
    "ops/_evidence/",
    "ops/_artifacts/",
    ".mypy_cache/",
    ".pytest_cache/",
    ".ruff_cache/",
    ".hypothesis/",
    ".idea/",
)
EXCLUDE_PARTS = {
    ".venv",
    ".mypy_cache",
    ".pytest_cache",
    ".ruff_cache",
    ".hypothesis",
    ".idea",
    "__pycache__",
    "site-packages",
}


def _iter_files(repo_root: Path) -> list[Path]:
    files: list[Path] = []
    for path in sorted(repo_root.rglob("*")):
        if not path.is_file():
            continue
        rel = path.relative_to(repo_root).as_posix()
        if rel.startswith("scripts/"):
            continue
        if any(rel.startswith(prefix) for prefix in EXCLUDE_PREFIXES):
            continue
        if any(part in EXCLUDE_PARTS for part in path.parts):
            continue
        files.append(path)
    return files


def _scan_refs(repo_root: Path) -> list[str]:
    found: list[str] = []
    for path in _iter_files(repo_root):
        rel = path.relative_to(repo_root).as_posix()
        try:
            text = path.read_text(encoding="utf-8")
        except Exception:
            continue
        if "scripts/" in text:
            found.append(rel)
    return sorted(set(found))


def _load_allowlist(repo_root: Path, allowlist_path: str) -> list[str]:
    path = (repo_root / allowlist_path).resolve()
    if not path.exists():
        return []
    lines = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        lines.append(line)
    return lines


def _uncovered(found: list[str], allowlist: list[str]) -> list[str]:
    missing: list[str] = []
    for item in found:
        if not any(fnmatch(item, pat) or item == pat for pat in allowlist):
            missing.append(item)
    return sorted(missing)


def _emit(payload: dict[str, Any], report_format: str) -> None:
    if report_format == "json":
        print(json.dumps(payload, sort_keys=True))
        return
    print(
        f"legacy {payload['action']}: status={payload['status']} "
        f"count={payload['count']} uncovered={payload.get('uncovered_count', 0)}"
    )
    for item in payload.get("uncovered", []):
        print(f"- {item}")


def run_legacy_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    found = _scan_refs(ctx.repo_root)
    payload: dict[str, Any] = {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "status": "pass",
        "action": ns.legacy_cmd,
        "run_id": ctx.run_id,
        "count": len(found),
        "references": found,
    }
    if ns.legacy_cmd == "audit":
        _emit(payload, ns.report)
        return 0

    allowlist = _load_allowlist(ctx.repo_root, ns.allowlist)
    uncovered = _uncovered(found, allowlist)
    payload["uncovered"] = uncovered
    payload["uncovered_count"] = len(uncovered)
    if uncovered:
        payload["status"] = "fail"
    _emit(payload, ns.report)
    return 1 if uncovered else 0


def configure_legacy_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("legacy", help="legacy migration debt audit helpers")
    legacy_sub = p.add_subparsers(dest="legacy_cmd", required=True)

    audit = legacy_sub.add_parser("audit", help="list current non-scripts files that still reference scripts/")
    audit.add_argument("--report", choices=["text", "json"], default="text")

    check = legacy_sub.add_parser("check", help="fail when scripts/ references are not covered by allowlist")
    check.add_argument("--allowlist", default=str(DEFAULT_ALLOWLIST))
    check.add_argument("--report", choices=["text", "json"], default="text")
