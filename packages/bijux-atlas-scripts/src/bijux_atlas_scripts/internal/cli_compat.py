from __future__ import annotations

import argparse
import os
import sys

from .system import dump_env, repo_root, run_timed


def _build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="python -m bijux_atlas_scripts.internal.cli_compat")
    sub = p.add_subparsers(dest="cmd", required=True)

    sub.add_parser("repo-root")

    env = sub.add_parser("env-dump")
    env.add_argument("--script-name", default=os.environ.get("SCRIPT_NAME", "env-dump"))
    env.add_argument("--run-id", default=os.environ.get("RUN_ID"))

    ex = sub.add_parser("exec")
    ex.add_argument("--script-name", default=os.environ.get("SCRIPT_NAME", "exec"))
    ex.add_argument("--run-id", default=os.environ.get("RUN_ID"))
    ex.add_argument("argv", nargs=argparse.REMAINDER)
    return p


def main(argv: list[str] | None = None) -> int:
    ns = _build_parser().parse_args(argv)
    if ns.cmd == "repo-root":
        print(repo_root())
        return 0
    if ns.cmd == "env-dump":
        print(dump_env(script_name=ns.script_name, run_id=ns.run_id))
        return 0
    if ns.cmd == "exec":
        cmd = ns.argv
        if cmd and cmd[0] == "--":
            cmd = cmd[1:]
        if not cmd:
            return 2
        code, _timing_path = run_timed(cmd, script_name=ns.script_name, run_id=ns.run_id)
        return code
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

