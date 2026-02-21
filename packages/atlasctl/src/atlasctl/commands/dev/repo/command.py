from __future__ import annotations

import argparse
import json
from pathlib import Path

from ....core.context import RunContext
from ....core.fs import ensure_evidence_path
from ....commands.policies.runtime.culprits import collect_dir_stats


def _density_rows(repo_root: Path) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    for row in collect_dir_stats(repo_root):
        rows.append(
            {
                "dir": row.dir,
                "py_files": row.py_files,
                "modules": row.modules,
                "shell_files": row.shell_files,
                "loc": row.total_loc,
                "rule": row.rule,
                "enforce": row.enforce,
                "budget": row.budget,
            }
        )
    return rows


def _top(rows: list[dict[str, object]], key: str, limit: int) -> list[dict[str, object]]:
    return sorted(rows, key=lambda item: (int(item[key]), str(item["dir"])), reverse=True)[:limit]


def run_repo_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    if ns.repo_cmd != "stats":
        return 2

    rows = _density_rows(ctx.repo_root)
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok",
        "command": "repo stats",
        "top": ns.top,
        "summary": {
            "total_dirs": len(rows),
            "densest_by_py_files": _top(rows, "py_files", ns.top),
            "densest_by_modules": _top(rows, "modules", ns.top),
            "largest_by_loc": _top(rows, "loc", ns.top),
        },
    }

    rendered = json.dumps(payload, sort_keys=True) if (ctx.output_format == "json" or ns.json) else json.dumps(payload, indent=2, sort_keys=True)
    if ns.out_file:
        ensure_evidence_path(ctx, Path(ns.out_file)).write_text(rendered + "\n", encoding="utf-8")
    print(rendered)
    return 0


def configure_repo_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("repo", help="repository stats and density reports")
    repo_sub = p.add_subparsers(dest="repo_cmd", required=True)

    stats = repo_sub.add_parser("stats", help="report file/module density by directory")
    stats.add_argument("--top", type=int, default=10)
    stats.add_argument("--json", action="store_true", help="emit JSON output")
    stats.add_argument("--out-file", default="", help="write output under evidence root")
