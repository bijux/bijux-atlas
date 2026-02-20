from __future__ import annotations

import argparse
import json

from ..core.context import RunContext


def run_paths_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok",
        "action": "paths-print",
        "paths": {
            "repo_root": str(ctx.repo_root),
            "configs_root": str((ctx.repo_root / "configs").resolve()),
            "ops_root": str((ctx.repo_root / "ops").resolve()),
            "docs_root": str((ctx.repo_root / "docs").resolve()),
            "artifacts_root": str((ctx.repo_root / "artifacts").resolve()),
            "evidence_root": str(ctx.evidence_root),
            "run_dir": str(ctx.run_dir),
            "atlasctl_artifact_root": str(ctx.scripts_artifact_root),
        },
    }
    print(json.dumps(payload, sort_keys=True) if ns.json else json.dumps(payload, indent=2, sort_keys=True))
    return 0


def configure_paths_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    paths = sub.add_parser("paths", help="print key repo and runtime paths")
    paths.add_argument("--json", action="store_true", help="emit JSON output")
    paths_sub = paths.add_subparsers(dest="paths_cmd", required=False)
    prn = paths_sub.add_parser("print", help="print key repo and runtime paths")
    prn.add_argument("--json", action="store_true", help="emit JSON output")
