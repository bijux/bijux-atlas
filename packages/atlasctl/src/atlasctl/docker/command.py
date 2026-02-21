from __future__ import annotations

import argparse

from ..core.context import RunContext
from ..core.exec_shell import run_shell_script


def run_docker_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    image = ns.image
    if ns.docker_cmd == "scan":
        payload = run_shell_script(ctx.repo_root / "docker/scripts/docker-scan.sh", args=[image], cwd=ctx.repo_root)
        return int(payload["exit_code"])
    if ns.docker_cmd == "smoke":
        payload = run_shell_script(ctx.repo_root / "docker/scripts/docker-runtime-smoke.sh", args=[image], cwd=ctx.repo_root)
        return int(payload["exit_code"])
    return 2


def configure_docker_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("docker", help="docker verification command group")
    docker_sub = p.add_subparsers(dest="docker_cmd", required=True)
    scan = docker_sub.add_parser("scan", help="run docker image vulnerability scan")
    scan.add_argument("--image", default="bijux-atlas:local")
    smoke = docker_sub.add_parser("smoke", help="run docker runtime smoke checks")
    smoke.add_argument("--image", default="bijux-atlas:local")
