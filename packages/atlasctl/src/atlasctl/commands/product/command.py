from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
from pathlib import Path

from ...core.context import RunContext
from ...core.process import run_command
from ...core.runtime.paths import write_text_file
from ...core.schema.schema_utils import validate_json
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

PRODUCT_SCHEMA = Path("configs/product/artifact-manifest.schema.json")
PRODUCT_TOOLS_MANIFEST = Path("configs/product/external-tools-manifest.json")


def _product_manifest_path(ctx: RunContext) -> Path:
    return ctx.repo_root / "artifacts" / "evidence" / "product" / "build" / ctx.run_id / "artifact-manifest.json"


def _rel(repo_root: Path, path: Path) -> str:
    try:
        return path.relative_to(repo_root).as_posix()
    except ValueError:
        return path.as_posix()


def _sha256(path: Path) -> str:
    h = hashlib.sha256()
    h.update(path.read_bytes())
    return h.hexdigest()


def _artifact_rows(ctx: RunContext) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    chart_dir = ctx.repo_root / "artifacts" / "chart"
    if chart_dir.exists():
        for p in sorted(chart_dir.rglob("*")):
            if not p.is_file():
                continue
            rows.append(
                {
                    "id": f"chart:{p.name}",
                    "path": _rel(ctx.repo_root, p),
                    "kind": "helm-chart-package" if p.suffix == ".tgz" else "file",
                    "sha256": _sha256(p),
                    "size_bytes": p.stat().st_size,
                }
            )
    image_tag = str(os.environ.get("DOCKER_IMAGE", "bijux-atlas:local")).strip()
    if image_tag:
        rows.append(
            {
                "id": "docker-image-tag",
                "path": image_tag,
                "kind": "docker-image-tag",
                "sha256": hashlib.sha256(image_tag.encode("utf-8")).hexdigest(),
                "size_bytes": 0,
            }
        )
    rows.sort(key=lambda r: (str(r["kind"]), str(r["id"]), str(r["path"])))
    return rows


def _build_manifest_payload(ctx: RunContext) -> dict[str, object]:
    version = str(os.environ.get("IMAGE_VERSION") or os.environ.get("VERSION") or "local").strip() or "local"
    pins_path = ctx.repo_root / "configs" / "ops" / "pins.json"
    pins_digest = hashlib.sha256(pins_path.read_bytes()).hexdigest() if pins_path.exists() else ""
    git_sha = str(ctx.meta.get("git_sha", "") or "").strip()
    if not git_sha:
        proc = subprocess.run(["git", "rev-parse", "--short=12", "HEAD"], cwd=ctx.repo_root, text=True, capture_output=True, check=False)
        git_sha = (proc.stdout or "").strip() if proc.returncode == 0 else "unknown"
    payload: dict[str, object] = {
        "schema_version": 1,
        "kind": "product-artifact-manifest",
        "run_id": ctx.run_id,
        "version": version,
        "artifacts": _artifact_rows(ctx),
        "meta": {
            "allowed_roots": ["artifacts/chart", "artifacts/evidence/product"],
            "tool_versions_hint": "use atlasctl ops pins check for pinned tool validation",
            "git_sha": git_sha,
            "pins_digest": pins_digest,
            "schema_versions": {
                "product_artifact_manifest": 1,
                "product_lane_report": 1,
            },
        },
    }
    validate_json(payload, ctx.repo_root / PRODUCT_SCHEMA)
    return payload


def _write_artifact_manifest(ctx: RunContext) -> Path:
    out = _product_manifest_path(ctx)
    out.parent.mkdir(parents=True, exist_ok=True)
    payload = _build_manifest_payload(ctx)
    write_text_file(out, json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return out


def _validate_artifact_manifest(ctx: RunContext, path: Path | None = None) -> int:
    pins = run_command(["./bin/atlasctl", "ops", "pins", "check", "--report", "text"], ctx.repo_root, ctx=ctx)
    if pins.code != 0:
        print("ops pins check failed; product validation requires pinned tool versions to pass")
        return 1
    target = path or _product_manifest_path(ctx)
    if not target.exists():
        print(f"missing product artifact manifest: {_rel(ctx.repo_root, target)}")
        return 1
    payload = json.loads(target.read_text(encoding="utf-8"))
    validate_json(payload, ctx.repo_root / PRODUCT_SCHEMA)
    for row in payload.get("artifacts", []):
        if not isinstance(row, dict):
            continue
        kind = str(row.get("kind", ""))
        apath = str(row.get("path", ""))
        if kind == "docker-image-tag":
            continue
        p = ctx.repo_root / apath
        if not p.exists():
            print(f"missing artifact file: {apath}")
            return 1
        if str(row.get("sha256", "")) != _sha256(p):
            print(f"artifact checksum mismatch: {apath}")
            return 1
    print(f"product artifact manifest valid: {_rel(ctx.repo_root, target)}")
    return 0


def _validate_product_tools_manifest(ctx: RunContext) -> int:
    path = ctx.repo_root / PRODUCT_TOOLS_MANIFEST
    if not path.exists():
        print(f"missing product tools manifest: {_rel(ctx.repo_root, path)}")
        return 1
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        print("invalid product tools manifest json")
        return 1
    tools = payload.get("tools")
    if not isinstance(tools, list) or not all(isinstance(x, str) and x.strip() for x in tools):
        print("invalid product tools manifest: tools[] must be non-empty strings")
        return 1
    return 0


def _validate_release_tag_contract(tag: str) -> bool:
    import re

    return bool(re.fullmatch(r"v\d+\.\d+\.\d+(?:[-+][A-Za-z0-9._-]+)?", tag.strip()))


def _release_gates(ctx: RunContext) -> int:
    # 180: release blocked if bypass inventory is non-empty
    culprits = run_command(["./bin/atlasctl", "policies", "culprits", "--format", "json"], ctx.repo_root, ctx=ctx)
    if culprits.code != 0:
        print("release gate failed: policies culprits inventory command failed")
        return 1
    try:
        payload = json.loads(culprits.stdout or culprits.combined_output or "{}")
    except Exception:
        print("release gate failed: policies culprits output is not valid json")
        return 1
    count = int(payload.get("entry_count", payload.get("total", 0)) or 0)
    if count != 0:
        print(f"release gate failed: bypass inventory is non-empty ({count})")
        return 1

    # 181: ops schema/contracts drift gate
    ops_contracts = run_command(
        ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/validation/validate_ops_contracts.py"],
        ctx.repo_root,
        ctx=ctx,
    )
    if ops_contracts.code != 0:
        print("release gate failed: ops schema/contracts drift check failed")
        return 1

    # 182: floating pins gate
    pins = run_command(["./bin/atlasctl", "ops", "pins", "check", "--report", "text"], ctx.repo_root, ctx=ctx)
    if pins.code != 0:
        print("release gate failed: ops pins check failed")
        return 1

    # 183: docs inventory freshness gate
    docs_inv = run_command(
        ["python3", "packages/atlasctl/src/atlasctl/checks/tools/docs_domain/check_commands_inventory_ops_product.py"],
        ctx.repo_root,
        ctx=ctx,
    )
    if docs_inv.code != 0:
        print("release gate failed: commands inventory docs are out of date")
        return 1
    if _validate_product_tools_manifest(ctx) != 0:
        print("release gate failed: product external tools manifest invalid")
        return 1
    tag = str(os.environ.get("GITHUB_REF_NAME") or os.environ.get("RELEASE_TAG") or "").strip()
    if tag and not _validate_release_tag_contract(tag):
        print(f"release gate failed: tag does not match release tag contract ({tag})")
        return 1
    return 0


def _diff_manifest(ctx: RunContext, left: Path, right: Path, report_json: bool) -> int:
    l = json.loads(left.read_text(encoding="utf-8"))
    r = json.loads(right.read_text(encoding="utf-8"))
    li = {str(x.get("id")): x for x in l.get("artifacts", []) if isinstance(x, dict)}
    ri = {str(x.get("id")): x for x in r.get("artifacts", []) if isinstance(x, dict)}
    added = sorted(set(ri) - set(li))
    removed = sorted(set(li) - set(ri))
    changed = sorted(k for k in set(li).intersection(ri) if li[k] != ri[k])
    payload = {"schema_version": 1, "tool": "atlasctl", "kind": "product-artifact-diff", "status": "ok", "added": added, "removed": removed, "changed": changed}
    if report_json or ctx.output_format == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        print(f"added={len(added)} removed={len(removed)} changed={len(changed)}")
    return 0


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
    build = product_sub.add_parser("build", help="produce canonical product artifact set and manifest")
    build.add_argument("--plan", action="store_true", help="print planned artifacts without building")
    product_sub.add_parser("verify", help="alias of validate (manifest + hashes)")
    product_sub.add_parser("validate", help="validate product artifact manifest and artifact integrity")
    diff = product_sub.add_parser("diff", help="compare two product artifact manifests")
    diff.add_argument("left")
    diff.add_argument("right")
    inventory = product_sub.add_parser("inventory", help="print product artifact inventory")
    inventory.add_argument("--manifest", help="path to artifact manifest (defaults to current run)")
    artifact_index = product_sub.add_parser("artifact-index", help="write artifact index JSON derived from artifact manifest")
    artifact_index.add_argument("--manifest", help="path to artifact manifest (defaults to current run)")
    artifact_index.add_argument("--out", help="output path (defaults next to manifest)")
    integration = product_sub.add_parser("integration-smoke", help="ops+product integration smoke flow")
    integration.add_argument("--internal", action="store_true")
    integration.add_argument("--dry-run", action="store_true")
    publish = product_sub.add_parser("publish", help="publish product artifacts (internal/optional)")
    publish.add_argument("--internal", action="store_true")
    release_candidate = product_sub.add_parser("release-candidate", help="build -> verify -> sign -> publish dry flow")
    release_candidate.add_argument("--internal", action="store_true")

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
    if sub == "build":
        if bool(getattr(ns, "dry_run", False)) or bool(getattr(ns, "plan", False)):
            print("product build plan:")
            print("- docker image tag artifact (from DOCKER_IMAGE)")
            print("- chart packages under artifacts/chart/")
            print(f"- artifact manifest at {_rel(ctx.repo_root, _product_manifest_path(ctx))}")
            return 0
        code = run_product_lane(
            ctx,
            lane="build",
            steps=[
                ProductStep("docker-build", ["./bin/atlasctl", "product", "docker", "build"]),
                ProductStep("chart-package", ["./bin/atlasctl", "product", "chart", "package"]),
            ],
        )
        if code != 0:
            return code
        out = _write_artifact_manifest(ctx)
        print(f"artifact-manifest: {_rel(ctx.repo_root, out)}")
        return 0
    if sub == "verify":
        return _validate_artifact_manifest(ctx)
    if sub == "validate":
        return _validate_artifact_manifest(ctx)
    if sub == "diff":
        return _diff_manifest(ctx, Path(ns.left), Path(ns.right), report_json=False)
    if sub == "inventory":
        p = Path(getattr(ns, "manifest", "")).expanduser() if getattr(ns, "manifest", None) else _product_manifest_path(ctx)
        if not p.is_absolute():
            p = (ctx.repo_root / p).resolve()
        if not p.exists():
            print(f"missing product artifact manifest: {_rel(ctx.repo_root, p)}")
            return 1
        payload = json.loads(p.read_text(encoding="utf-8"))
        if ctx.output_format == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            print(f"product inventory version={payload.get('version')} artifacts={len(payload.get('artifacts', []))}")
            for row in payload.get("artifacts", []):
                if isinstance(row, dict):
                    print(f"- {row.get('id')} {row.get('path')}")
        return 0
    if sub == "artifact-index":
        p = Path(getattr(ns, "manifest", "")).expanduser() if getattr(ns, "manifest", None) else _product_manifest_path(ctx)
        if not p.is_absolute():
            p = (ctx.repo_root / p).resolve()
        if not p.exists():
            print(f"missing product artifact manifest: {_rel(ctx.repo_root, p)}")
            return 1
        payload = json.loads(p.read_text(encoding="utf-8"))
        out = Path(getattr(ns, "out", "")).expanduser() if getattr(ns, "out", None) else p.with_name("artifact-index.json")
        if not out.is_absolute():
            out = (ctx.repo_root / out).resolve()
        index = {
            "schema_version": 1,
            "kind": "product-artifact-index",
            "run_id": payload.get("run_id"),
            "version": payload.get("version"),
            "artifacts": payload.get("artifacts", []),
        }
        write_text_file(out, json.dumps(index, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(_rel(ctx.repo_root, out))
        return 0
    if sub == "publish":
        if not bool(getattr(ns, "internal", False)):
            print("product publish is internal-only; pass --internal")
            return 2
        print("product publish plan: validate -> push image/chart -> publish manifest index (not yet wired)")
        return 0
    if sub == "integration-smoke":
        if not bool(getattr(ns, "internal", False)):
            print("product integration-smoke is internal-only; pass --internal")
            return 2
        steps = [
            ["./bin/atlasctl", "ops", "stack", "up", "--report", "text"],
            ["./bin/atlasctl", "ops", "deploy", "apply", "--report", "text"],
            ["./bin/atlasctl", "ops", "datasets", "verify", "--report", "text"],
            ["./bin/atlasctl", "ops", "e2e", "run", "--report", "text", "--suite", "smoke"],
            ["./bin/atlasctl", "report", "unified", "--format", "json"],
        ]
        if bool(getattr(ns, "dry_run", False)) or bool(getattr(ns, "dry_run", False)):
            print("product integration-smoke plan:")
            for cmd in steps:
                print(f"- {' '.join(cmd)}")
            return 0
        for cmd in steps:
            proc = run_command(cmd, ctx.repo_root, ctx=ctx)
            if proc.combined_output:
                print(proc.combined_output.rstrip())
            if proc.code != 0:
                return proc.code
        return 0
    if sub == "release-candidate":
        if not bool(getattr(ns, "internal", False)):
            print("product release-candidate is internal-only; pass --internal")
            return 2
        gate = _release_gates(ctx)
        if gate != 0:
            return gate
        for cmd in (
            ["./bin/atlasctl", "product", "build"],
            ["./bin/atlasctl", "product", "verify"],
        ):
            proc = run_command(cmd, ctx.repo_root, ctx=ctx)
            if proc.combined_output:
                print(proc.combined_output.rstrip())
            if proc.code != 0:
                return proc.code
        manifest = _product_manifest_path(ctx)
        if not manifest.exists():
            print("missing manifest after product build")
            return 1
        sig = manifest.with_suffix(".sha256")
        sig.write_text(f"{_sha256(manifest)}  {manifest.name}\n", encoding="utf-8")
        print(f"signed (checksum): {_rel(ctx.repo_root, sig)}")
        print("release-candidate publish step: dry-run placeholder (not pushing)")
        return 0
    if sub == "release" and getattr(ns, "product_release_cmd", "") == "plan":
        lines = [
            "product release plan",
            "- verify docker contracts and chart validation before release",
            "- build canonical product artifact set and artifact-manifest.json",
            "- compute immutable image tags (no `latest`)",
            "- publish chart package and verify app/chart version alignment",
            "- execute docker release in CI-only environment",
            "- release-candidate flow: build -> verify -> checksum sign -> publish (dry/internal)",
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
