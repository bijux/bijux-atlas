from __future__ import annotations

import json
import os
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from ...core.context import RunContext
from ...core.exec import run as process_run
from ...reporting.writer import write_json_report


@dataclass(frozen=True)
class ProductStep:
    name: str
    command: list[str]
    allow_failure: bool = False


def _now_iso() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def _product_report_path(lane: str, run_id: str) -> str:
    lane_slug = lane.replace(" ", "-")
    return f"product/{lane_slug}/{run_id}/report.json"


def _run_step(ctx: RunContext, step: ProductStep, env: dict[str, str] | None = None) -> dict[str, Any]:
    proc = process_run(step.command, cwd=ctx.repo_root, env=env, text=True, capture_output=True)
    return {
        "name": step.name,
        "command": step.command,
        "exit_code": int(proc.returncode),
        "stdout": proc.stdout or "",
        "stderr": proc.stderr or "",
        "status": "pass" if proc.returncode == 0 else ("allowed-fail" if step.allow_failure else "fail"),
    }


def run_product_lane(
    ctx: RunContext,
    *,
    lane: str,
    steps: list[ProductStep],
    meta: dict[str, object] | None = None,
) -> int:
    rows: list[dict[str, Any]] = []
    for step in steps:
        row = _run_step(ctx, step, env=os.environ.copy())
        rows.append(row)
        if row["status"] == "fail":
            break

    status = "ok" if all(row["status"] != "fail" for row in rows) else "error"
    payload: dict[str, Any] = {
        "schema_version": 1,
        "tool": "atlasctl",
        "kind": "product-lane-report",
        "run_id": ctx.run_id,
        "lane": lane,
        "status": status,
        "started_at": _now_iso(),
        "steps": rows,
        "summary": {
            "total": len(rows),
            "failed": sum(1 for row in rows if row["status"] == "fail"),
            "passed": sum(1 for row in rows if row["status"] == "pass"),
        },
        "allowed_write_roots": [str(ctx.evidence_root), str(ctx.scripts_artifact_root)],
        "meta": meta or {},
    }
    report_path = write_json_report(ctx, _product_report_path(lane, ctx.run_id), payload)
    payload["report_path"] = str(report_path)
    if ctx.output_format == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        print(f"product {lane}: {'pass' if status == 'ok' else 'fail'}")
        print(f"report: {report_path}")
    return 0 if status == "ok" else 1


def product_steps_bootstrap(ctx: RunContext) -> list[ProductStep]:
    return [
        ProductStep("python-version", ["python3", "--version"]),
        ProductStep("pip-check", ["bash", "-lc", "command -v pip >/dev/null 2>&1"]),
        ProductStep("docs-reqs-install", ["python3", "-m", "pip", "install", "-r", "configs/docs/requirements.txt"]),
        ProductStep("k6-presence", ["bash", "-lc", "command -v k6 >/dev/null 2>&1 || echo 'k6 not found (optional for non-perf workflows)'"], allow_failure=True),
        ProductStep("kind-presence", ["bash", "-lc", "command -v kind >/dev/null 2>&1 || echo 'kind not found (required for k8s e2e)'"], allow_failure=True),
        ProductStep("kubectl-presence", ["bash", "-lc", "command -v kubectl >/dev/null 2>&1 || echo 'kubectl not found (required for k8s e2e)'"], allow_failure=True),
    ]


def product_steps_docker_build(ctx: RunContext) -> list[ProductStep]:
    image_tag = os.environ.get("DOCKER_IMAGE", "bijux-atlas:local")
    image_version = os.environ.get("IMAGE_VERSION")
    vcs_ref = os.environ.get("VCS_REF")
    build_date = os.environ.get("BUILD_DATE")
    rust_version = os.environ.get("RUST_VERSION", "1.84.1")
    if image_version is None:
        out = process_run(["git", "rev-parse", "--short=12", "HEAD"], cwd=ctx.repo_root, text=True, capture_output=True)
        image_version = (out.stdout or "").strip() or "unknown"
    if vcs_ref is None:
        out = process_run(["git", "rev-parse", "HEAD"], cwd=ctx.repo_root, text=True, capture_output=True)
        vcs_ref = (out.stdout or "").strip() or "unknown"
    if build_date is None:
        build_date = _now_iso()
    provenance = os.environ.get("IMAGE_PROVENANCE", image_tag)
    return [
        ProductStep(
            "docker-build",
            [
                "docker",
                "build",
                "--pull=false",
                "-t",
                image_tag,
                "-f",
                "docker/images/runtime/Dockerfile",
                "--build-arg",
                f"RUST_VERSION={rust_version}",
                "--build-arg",
                f"IMAGE_VERSION={image_version}",
                "--build-arg",
                f"VCS_REF={vcs_ref}",
                "--build-arg",
                f"BUILD_DATE={build_date}",
                "--build-arg",
                f"IMAGE_PROVENANCE={provenance}",
                ".",
            ],
        )
    ]


def product_steps_docker_push(_: RunContext) -> list[ProductStep]:
    image_tag = os.environ.get("DOCKER_IMAGE", "")
    if not image_tag:
        return [ProductStep("docker-push-missing-image", ["bash", "-lc", "echo 'DOCKER_IMAGE is required for docker-push' >&2; exit 2"])]
    return [ProductStep("docker-push", ["docker", "push", image_tag])]


def product_steps_docker_check(ctx: RunContext) -> list[ProductStep]:
    image_tag = os.environ.get("DOCKER_IMAGE", "bijux-atlas:local")
    return [
        ProductStep("docker-contracts", ["./bin/atlasctl", "product", "docker", "contracts"]),
        *product_steps_docker_build(ctx),
        ProductStep("docker-smoke", ["./bin/atlasctl", "docker", "smoke", "--image", image_tag]),
    ]


def product_steps_docker_release(ctx: RunContext) -> list[ProductStep]:
    return [
        ProductStep("docker-check", ["./bin/atlasctl", "product", "docker", "check"]),
        ProductStep("docker-push", ["./bin/atlasctl", "product", "docker", "push"]),
    ]


def product_steps_chart_package(_: RunContext) -> list[ProductStep]:
    return [
        ProductStep("chart-artifacts-dir", ["mkdir", "-p", "artifacts/chart"]),
        ProductStep("helm-package", ["helm", "package", "ops/k8s/charts/bijux-atlas", "--destination", "artifacts/chart"]),
    ]


def product_steps_chart_verify(_: RunContext) -> list[ProductStep]:
    return [
        ProductStep("helm-lint", ["helm", "lint", "ops/k8s/charts/bijux-atlas"]),
        ProductStep("helm-template", ["bash", "-lc", "helm template atlas ops/k8s/charts/bijux-atlas >/dev/null"]),
    ]


def product_steps_chart_validate(_: RunContext) -> list[ProductStep]:
    return [
        ProductStep("chart-verify", ["./bin/atlasctl", "product", "chart", "verify"]),
        ProductStep("contracts-generate-chart-schema", ["./bin/atlasctl", "contracts", "generate", "--generators", "chart-schema"]),
        ProductStep("contracts-check-chart-values", ["./bin/atlasctl", "contracts", "check", "--checks", "chart-values"]),
    ]


def product_steps_naming_lint(_: RunContext) -> list[ProductStep]:
    return [
        ProductStep("docs-durable-naming-check", ["./bin/atlasctl", "docs", "durable-naming-check", "--report", "text"]),
        ProductStep("docs-duplicate-topics-check", ["./bin/atlasctl", "docs", "duplicate-topics-check", "--report", "text"]),
    ]


def product_steps_docs_naming_lint(_: RunContext) -> list[ProductStep]:
    return [
        ProductStep("docs-naming-inventory", ["./bin/atlasctl", "docs", "naming-inventory", "--report", "text"]),
        ProductStep("docs-legacy-terms-check", ["./bin/atlasctl", "docs", "legacy-terms-check", "--report", "text"]),
        ProductStep("docs-observability-checklist", ["./bin/atlasctl", "docs", "observability-docs-checklist", "--report", "text"]),
        ProductStep("docs-no-orphan-docs-check", ["./bin/atlasctl", "docs", "no-orphan-docs-check", "--report", "text"]),
        ProductStep("docs-script-locations-check", ["./bin/atlasctl", "docs", "script-locations-check", "--report", "text"]),
        ProductStep("docs-runbook-map-registration-check", ["./bin/atlasctl", "docs", "runbook-map-registration-check", "--report", "text"]),
        ProductStep("docs-contract-doc-pairs-check", ["./bin/atlasctl", "docs", "contract-doc-pairs-check", "--report", "text"]),
        ProductStep("load-suite-manifest-validate", ["python3", "packages/atlasctl/src/atlasctl/load/contracts/validate_suite_manifest.py"]),
        ProductStep("docs-index-pages-check", ["./bin/atlasctl", "docs", "index-pages-check", "--report", "text"]),
    ]
