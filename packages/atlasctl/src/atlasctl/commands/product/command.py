from __future__ import annotations

import argparse
import os

from ...core.context import RunContext
from ...core.effects.product import (
    ProductStep,
    product_steps_bootstrap,
    product_steps_chart_package,
    product_steps_chart_validate,
    product_steps_chart_verify,
    product_steps_docker_build,
    product_steps_docker_check,
    product_steps_docker_push,
    product_steps_docker_release,
    product_steps_docs_naming_lint,
    product_steps_naming_lint,
    run_product_lane,
)


def _plan_rows() -> list[tuple[str, list[str]]]:
    return [
        ("bootstrap", ["./bin/atlasctl", "product", "bootstrap"]),
        ("docker build", ["./bin/atlasctl", "product", "docker", "build"]),
        ("docker check", ["./bin/atlasctl", "product", "docker", "check"]),
        ("docker contracts", ["./bin/atlasctl", "check", "domain", "docker"]),
        ("docker push", ["./bin/atlasctl", "product", "docker", "push"]),
        ("docker release", ["./bin/atlasctl", "product", "docker", "release"]),
        ("chart package", ["./bin/atlasctl", "product", "chart", "package"]),
        ("chart verify", ["./bin/atlasctl", "product", "chart", "verify"]),
        ("chart validate", ["./bin/atlasctl", "product", "chart", "validate"]),
        ("naming lint", ["./bin/atlasctl", "product", "naming", "lint"]),
        ("docs naming-lint", ["./bin/atlasctl", "product", "docs", "naming-lint"]),
        ("check", ["./bin/atlasctl", "product", "docker", "check"], ["./bin/atlasctl", "product", "chart", "validate"]),
    ]


def configure_product_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    parser = sub.add_parser("product", help="product release/build/chart wrapper commands")
    product_sub = parser.add_subparsers(dest="product_cmd", required=True)

    product_sub.add_parser("bootstrap", help="run product bootstrap lane")
    product_sub.add_parser("check", help="run canonical product verification lane")
    release = product_sub.add_parser("release", help="product release helpers")
    release_sub = release.add_subparsers(dest="product_release_cmd", required=True)
    release_sub.add_parser("plan", help="print release version/tag strategy")
    product_sub.add_parser("explain", help="print product lane plan and underlying commands")
    product_sub.add_parser("graph", help="print product lane dependency graph")

    docker = product_sub.add_parser("docker", help="product docker lanes")
    docker_sub = docker.add_subparsers(dest="product_docker_cmd", required=True)
    docker_sub.add_parser("build", help="run product docker build lane")
    docker_sub.add_parser("push", help="run product docker push lane")
    docker_sub.add_parser("release", help="run CI-only product docker release lane")
    docker_sub.add_parser("check", help="run product docker checks lane")
    docker_sub.add_parser("contracts", help="run product docker contracts lane")

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
        return run_product_lane(ctx, lane="bootstrap", steps=product_steps_bootstrap(ctx))
    if sub == "explain":
        for row in _plan_rows():
            name = row[0]
            cmds = row[1:]
            print(f"{name}:")
            for cmd in cmds:
                print(f"  {' '.join(cmd)}")
        return 0
    if sub == "graph":
        print("product.check -> product.docker.check")
        print("product.check -> product.chart.validate")
        print("product.docker.release -> product.docker.build")
        print("product.docker.release -> product.docker.contracts")
        print("product.docker.release -> product.docker.push")
        print("product.chart.verify -> product.chart.package")
        return 0
    if sub == "release" and getattr(ns, "product_release_cmd", "") == "plan":
        lines = [
            "product release plan",
            "- verify docker contracts and chart validation before release",
            "- compute immutable image tags (no `latest`)",
            "- publish chart package and verify app/chart version alignment",
            "- execute docker release in CI-only environment",
        ]
        print("\n".join(lines))
        return 0
    if sub == "check":
        return run_product_lane(
            ctx,
            lane="check",
            steps=[
                ProductStep("docker-check", ["./bin/atlasctl", "product", "docker", "check"]),
                ProductStep("chart-validate", ["./bin/atlasctl", "product", "chart", "validate"]),
            ],
        )

    if sub == "docker":
        dsub = getattr(ns, "product_docker_cmd", "")
        if dsub == "build":
            return run_product_lane(ctx, lane="docker build", steps=product_steps_docker_build(ctx))
        if dsub == "push":
            if str(os.environ.get("CI", "0")) != "1":
                print("docker-push is CI-only")
                return 2
            return run_product_lane(ctx, lane="docker push", steps=product_steps_docker_push(ctx))
        if dsub == "check":
            return run_product_lane(ctx, lane="docker check", steps=product_steps_docker_check(ctx))
        if dsub == "contracts":
            return run_product_lane(
                ctx,
                lane="docker contracts",
                steps=[ProductStep("docker-contracts", ["./bin/atlasctl", "check", "domain", "docker"])],
            )
        if dsub == "release":
            if not str(os.environ.get("CI", "")).strip():
                print("product docker release is CI-only; set CI=1 to run this lane")
                return 2
            return run_product_lane(ctx, lane="docker release", steps=product_steps_docker_release(ctx), meta={"ci_only": True})
        return 2

    if sub == "chart":
        csub = getattr(ns, "product_chart_cmd", "")
        if csub == "package":
            return run_product_lane(ctx, lane="chart package", steps=product_steps_chart_package(ctx))
        if csub == "verify":
            return run_product_lane(ctx, lane="chart verify", steps=product_steps_chart_verify(ctx))
        if csub == "validate":
            return run_product_lane(ctx, lane="chart validate", steps=product_steps_chart_validate(ctx))
        return 2

    if sub == "naming" and getattr(ns, "product_naming_cmd", "") == "lint":
        return run_product_lane(ctx, lane="naming lint", steps=product_steps_naming_lint(ctx))

    if sub == "docs" and getattr(ns, "product_docs_cmd", "") == "naming-lint":
        return run_product_lane(ctx, lane="docs naming-lint", steps=product_steps_docs_naming_lint(ctx))

    return 2
