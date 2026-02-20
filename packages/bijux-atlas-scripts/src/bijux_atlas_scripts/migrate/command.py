from __future__ import annotations

import argparse
import subprocess

from ..core.context import RunContext


def _run(repo_root, cmd: list[str]) -> int:
    proc = subprocess.run(cmd, cwd=repo_root, text=True, check=False)
    return proc.returncode


def run_migrate_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    if ns.migrate_cmd != "layout":
        return 2

    sequence = [
        ["bash", "scripts/areas/internal/migrate_paths.sh", "--apply"],
        ["bash", "scripts/areas/layout/check_root_shape.sh"],
        ["bash", "scripts/areas/layout/check_forbidden_root_names.sh"],
        ["bash", "scripts/areas/layout/check_repo_hygiene.sh"],
    ]
    for legacy in ("charts", "e2e", "load", "observability", "datasets", "fixtures"):
        path = ctx.repo_root / legacy
        if path.is_symlink() or path.is_file():
            path.unlink(missing_ok=True)
        elif path.is_dir():
            try:
                path.rmdir()
            except OSError:
                pass
    for cmd in sequence:
        code = _run(ctx.repo_root, cmd)
        if code != 0:
            return code
    print("layout migration completed")
    return 0


def configure_migrate_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("migrate", help="repository migration command group")
    msub = p.add_subparsers(dest="migrate_cmd", required=True)
    msub.add_parser("layout", help="apply deterministic layout path migrations")
