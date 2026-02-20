from __future__ import annotations

import argparse
import subprocess

from ..core.context import RunContext


def run_docker_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    image = ns.image
    if ns.docker_cmd == "scan":
        proc = subprocess.run(["docker/scripts/docker-scan.sh", image], cwd=ctx.repo_root, text=True, check=False)
        return proc.returncode
    if ns.docker_cmd == "smoke":
        proc = subprocess.run(
            ["docker/scripts/docker-runtime-smoke.sh", image],
            cwd=ctx.repo_root,
            text=True,
            check=False,
        )
        return proc.returncode
    return 2


def configure_docker_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("docker", help="docker verification command group")
    docker_sub = p.add_subparsers(dest="docker_cmd", required=True)
    scan = docker_sub.add_parser("scan", help="run docker image vulnerability scan")
    scan.add_argument("--image", default="bijux-atlas:local")
    smoke = docker_sub.add_parser("smoke", help="run docker runtime smoke checks")
    smoke.add_argument("--image", default="bijux-atlas:local")

