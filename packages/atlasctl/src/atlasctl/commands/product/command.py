from __future__ import annotations

import argparse
import os

from ...core.context import RunContext
from ...core.exec import run


def _run_cmd(ctx: RunContext, cmd: list[str]) -> int:
    return run(cmd, cwd=ctx.repo_root, text=True).returncode


def configure_product_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    parser = sub.add_parser("product", help="product release/build/chart wrapper commands")
    product_sub = parser.add_subparsers(dest="product_cmd", required=True)

    product_sub.add_parser("bootstrap", help="run product bootstrap lane")

    docker = product_sub.add_parser("docker", help="product docker lanes")
    docker_sub = docker.add_subparsers(dest="product_docker_cmd", required=True)
    docker_sub.add_parser("build", help="run product docker build lane")
    docker_sub.add_parser("push", help="run product docker push lane")
    docker_sub.add_parser("release", help="run CI-only product docker release lane")
    docker_sub.add_parser("check", help="run product docker checks lane")

    chart = product_sub.add_parser("chart", help="product chart lanes")
    chart_sub = chart.add_subparsers(dest="product_chart_cmd", required=True)
    chart_sub.add_parser("package", help="run chart package lane")
    chart_sub.add_parser("verify", help="run chart verify lane")
    chart_sub.add_parser("validate", help="run chart validate lane")

    naming = product_sub.add_parser("naming", help="product naming lint lanes")
    naming_sub = naming.add_subparsers(dest="product_naming_cmd", required=True)
    naming_sub.add_parser("lint", help="run naming lint lane")

    docs = product_sub.add_parser("docs", help="product docs helper lanes")
    docs_sub = docs.add_subparsers(dest="product_docs_cmd", required=True)
    docs_sub.add_parser("naming-lint", help="run docs naming lint lane")


def run_product_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    sub = getattr(ns, "product_cmd", "")
    if sub == "bootstrap":
        return _run_cmd(ctx, ["./bin/atlasctl", "run", "./ops/run/product/product_bootstrap.sh"])

    if sub == "docker":
        dsub = getattr(ns, "product_docker_cmd", "")
        if dsub == "build":
            return _run_cmd(ctx, ["./bin/atlasctl", "run", "./ops/run/product/product_docker_build.sh"])
        if dsub == "push":
            return _run_cmd(ctx, ["./bin/atlasctl", "run", "./ops/run/product/product_docker_push.sh"])
        if dsub == "check":
            return _run_cmd(ctx, ["./bin/atlasctl", "run", "./ops/run/product/product_docker_check.sh"])
        if dsub == "release":
            if not str(os.environ.get("CI", "")).strip():
                print("product docker release is CI-only; set CI=1 to run this lane")
                return 2
            return _run_cmd(ctx, ["./bin/atlasctl", "run", "./ops/run/product/product_docker_release.sh"])
        return 2

    if sub == "chart":
        csub = getattr(ns, "product_chart_cmd", "")
        if csub == "package":
            return _run_cmd(ctx, ["./bin/atlasctl", "run", "./ops/run/product/product_chart_package.sh"])
        if csub == "verify":
            return _run_cmd(ctx, ["./bin/atlasctl", "run", "./ops/run/product/product_chart_verify.sh"])
        if csub == "validate":
            return _run_cmd(ctx, ["./bin/atlasctl", "run", "./ops/run/product/product_chart_validate.sh"])
        return 2

    if sub == "naming" and getattr(ns, "product_naming_cmd", "") == "lint":
        return _run_cmd(ctx, ["./bin/atlasctl", "run", "./ops/run/product/product_rename_lint.sh"])

    if sub == "docs" and getattr(ns, "product_docs_cmd", "") == "naming-lint":
        return _run_cmd(ctx, ["./bin/atlasctl", "run", "./ops/run/product/product_docs_lint_names.sh"])

    return 2

