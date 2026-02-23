from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Any

from ...contracts.output.base import build_output_base
from ...core.runtime.paths import write_text_file
from ..context import RunContext
from ..exec import run as process_run
from .run_meta import write_run_meta
from .dev_ci_suite import run_suite_ci

LANE_FILTERS: dict[str, tuple[str, ...]] = {
    "fmt": ("*fmt*",),
    "lint": ("*lint*", "*clippy*"),
    "test": ("*test*",),
    "contracts": ("*contract*", "*schema*"),
    "docs": ("check docs*", "cmd atlasctl docs*"),
    "ops": ("check ops*", "cmd atlasctl ops*"),
    "rust": ("check repo*", "cmd *cargo*"),
}

CI_LANES: tuple[tuple[str, str, str, str, str], ...] = (
    ("ci", "canonical CI suite run", "atlasctl ci run --json", "lane", "ci"),
    ("ci-fast", "fast CI lane", "atlasctl ci fast --json", "lane", "fast"),
    ("ci-all", "all CI lanes", "atlasctl ci all --json", "lane", "all"),
    ("ci-contracts", "contracts CI lane", "atlasctl ci contracts --json", "lane", "contracts"),
    ("ci-docs", "docs CI lane", "atlasctl ci docs --json", "lane", "docs"),
    ("ci-ops", "ops CI lane", "atlasctl ci ops --json", "lane", "ops"),
    ("ci-release", "release CI lane", "atlasctl ci release --json", "lane", "release"),
    ("ci-release-all", "release full CI lane", "atlasctl ci release-all --json", "lane", "release-all"),
    ("ci-pr", "PR lane (fmt+lint+test+repo-fast checks)", "atlasctl ci pr --json", "lane", "ci-pr"),
    ("ci-nightly", "Nightly lane (includes slow checks and full tests)", "atlasctl ci nightly --json", "lane", "ci-nightly"),
    ("ci-deps", "dependency lock refresh lane", "atlasctl ci deps --json", "lane", "deps"),
    ("ci-init", "initialize CI isolate/tmp dirs", "atlasctl ci init --json", "helper", ""),
    ("ci-artifacts", "print CI artifact locations", "atlasctl ci artifacts --json", "helper", ""),
)


def _ci_out_dir(ctx: RunContext, override: str | None) -> Path:
    if override:
        path = Path(override)
        return path if path.is_absolute() else (ctx.repo_root / path)
    return ctx.repo_root / "artifacts" / "evidence" / "ci" / ctx.run_id


def _lane_for_label(label: str) -> str:
    low = label.lower()
    if "contract" in low or "schema" in low:
        return "contracts"
    if "docs" in low:
        return "docs"
    if "ops" in low or "k8s" in low:
        return "ops"
    if "fmt" in low:
        return "fmt"
    if "lint" in low or "clippy" in low:
        return "lint"
    if "test" in low:
        return "test"
    return "rust"


def _run_step(
    ctx: RunContext,
    cmd: list[str] | str,
    *,
    verbose: bool,
    env: dict[str, str] | None = None,
) -> dict[str, Any]:
    display = cmd if isinstance(cmd, str) else " ".join(cmd)
    command = ["bash", "-lc", cmd] if isinstance(cmd, str) else cmd
    try:
        if verbose:
            proc = process_run(command, cwd=ctx.repo_root, env=env, text=True, capture_output=False)
            return {"command": display, "exit_code": proc.returncode}
        proc = process_run(command, cwd=ctx.repo_root, env=env, text=True, capture_output=True)
    except OSError as exc:
        return {
            "command": display,
            "exit_code": 1,
            "stdout": "",
            "stderr": (
                f"runtime error: {exc}. "
                "Next: run `./bin/atlasctl install doctor` and ensure required tools are installed."
            ),
        }
    return {
        "command": display,
        "exit_code": proc.returncode,
        "stdout": proc.stdout or "",
        "stderr": proc.stderr or "",
    }


def _emit_result(ctx: RunContext, ns: argparse.Namespace, action: str, steps: list[dict[str, Any]]) -> int:
    ok = all(int(step.get("exit_code", 1)) == 0 for step in steps)
    errors = [step["command"] for step in steps if int(step.get("exit_code", 1)) != 0]
    payload = build_output_base(
        run_id=ctx.run_id,
        ok=ok,
        errors=errors,
        meta={"action": action, "steps": steps},
        version=2,
    )
    payload["status"] = "ok" if ok else "error"
    if ns.json or ctx.output_format == "json":
        print(json.dumps(payload, sort_keys=True))
    elif ok:
        print(f"ci {action}: pass")
    else:
        print(f"ci {action}: fail (next: rerun with --verbose and inspect failing step output)")
    return 0 if ok else 1


def _print_lanes(ctx: RunContext, ns: argparse.Namespace) -> int:
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok",
        "kind": "ci-lanes-list",
        "lanes": [
            {
                "name": name,
                "description": description,
                "atlasctl": atlasctl,
                "kind": kind,
                "suite": suite,
            }
            for name, description, atlasctl, kind, suite in CI_LANES
        ],
    }
    if ns.json or ctx.output_format == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        for lane in payload["lanes"]:
            print(f"{lane['name']}\t{lane['atlasctl']}\t{lane['description']}")
    return 0


def _print_artifacts(ctx: RunContext, ns: argparse.Namespace) -> int:
    root = ctx.repo_root / "artifacts" / "evidence" / "ci"
    latest = None
    if root.exists():
        runs = [p for p in root.iterdir() if p.is_dir()]
        if runs:
            latest = max(runs, key=lambda p: p.stat().st_mtime)
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok",
        "root": str(root),
        "current_run_dir": str(root / ctx.run_id),
        "latest_run_dir": str(latest) if latest else "",
        "next": "run `./bin/atlasctl ci run --json` to populate artifacts",
    }
    if ns.json or ctx.output_format == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        print(f"ci artifacts root: {payload['root']}")
        print(f"current run: {payload['current_run_dir']}")
        if payload["latest_run_dir"]:
            print(f"latest run: {payload['latest_run_dir']}")
    return 0


def _run_report(ctx: RunContext, ns: argparse.Namespace) -> int:
    if not bool(getattr(ns, "latest", False)):
        msg = "only `atlasctl dev ci report --latest` is supported"
        print(json.dumps({"status": "error", "message": msg}, sort_keys=True) if (ns.json or ctx.output_format == "json") else msg)
        return 2
    root = ctx.repo_root / "artifacts" / "evidence" / "ci"
    if not root.exists():
        payload = {"status": "error", "message": "no ci evidence runs found", "next": "run `./bin/atlasctl dev ci run` first"}
        print(json.dumps(payload, sort_keys=True) if (ns.json or ctx.output_format == "json") else payload["message"])
        return 1
    runs = [p for p in root.iterdir() if p.is_dir()]
    if not runs:
        payload = {"status": "error", "message": "no ci evidence runs found", "next": "run `./bin/atlasctl dev ci run` first"}
        print(json.dumps(payload, sort_keys=True) if (ns.json or ctx.output_format == "json") else payload["message"])
        return 1
    latest = max(runs, key=lambda p: p.stat().st_mtime)
    report = latest / "suite-ci.report.json"
    summary = latest / "suite-ci.summary.txt"
    meta = latest / "run.meta.json"
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok",
        "run_id": latest.name,
        "latest": str(latest),
        "artifacts": {"json": str(report), "summary": str(summary), "meta": str(meta)},
    }
    if ns.json or ctx.output_format == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        print(f"latest ci report: {latest.name}")
        print(f"- json: {report}")
        print(f"- summary: {summary}")
        print(f"- meta: {meta}")
    return 0


def run_ci_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    verbose = bool(getattr(ns, "verbose", False) or ctx.verbose)
    cmd = ns.ci_cmd

    if cmd == "list":
        return _print_lanes(ctx, ns)
    if cmd == "artifacts":
        return _print_artifacts(ctx, ns)
    if cmd == "run":
        return run_suite_ci(ctx, ns, ci_out_dir=_ci_out_dir, lane_for_label=_lane_for_label, lane_filters=LANE_FILTERS)
    if cmd == "report":
        return _run_report(ctx, ns)

    env = os.environ.copy()
    steps_by_cmd: dict[str, list[list[str] | str]] = {
        "all": [[sys.executable, "-m", "atlasctl.cli", "--quiet", "--format", "json", "suite", "run", "all", "--json", "--keep-going"]],
        "init": [
            [sys.executable, "-m", "atlasctl.cli", "--quiet", "ci", "init-iso-dirs", "--json"],
            [sys.executable, "-m", "atlasctl.cli", "--quiet", "ci", "init-tmp", "--json"],
        ],
        "release": [
            [sys.executable, "-m", "atlasctl.cli", "--quiet", "ci", "release-compat-matrix-verify", "--json"],
            [sys.executable, "-m", "atlasctl.cli", "--quiet", "ci", "release-build-artifacts", "--json"],
            [sys.executable, "-m", "atlasctl.cli", "--quiet", "ci", "release-notes-render", "--json"],
        ],
        "release-all": [
            [sys.executable, "-m", "atlasctl.cli", "--quiet", "ci", "release", "--json"],
            [sys.executable, "-m", "atlasctl.cli", "--quiet", "ci", "release-publish-gh", "--json"],
            [sys.executable, "-m", "atlasctl.cli", "--quiet", "ci", "reproducible-verify", "--json"],
            [sys.executable, "-m", "atlasctl.cli", "--quiet", "ci", "security-advisory-render", "--json"],
        ],
        "scripts": [
            ["./bin/atlasctl", "--quiet", "lint", "scripts", "--report", "json"],
            ["./bin/atlasctl", "--quiet", "check", "run", "--group", "repo", "--json"],
        ],
        "pr": [
            ["./bin/atlasctl", "--quiet", "--format", "json", "suite", "run", "ci-pr", "--json"],
        ],
        "nightly": [
            ["./bin/atlasctl", "--quiet", "--format", "json", "suite", "run", "ci-nightly", "--json"],
        ],
        "fast": [["python3", "-m", "atlasctl.cli", "--quiet", "suite", "run", "fast", "--json"]],
        "contracts": [
            ["python3", "-m", "atlasctl.cli", "--quiet", "contracts", "check", "--checks", "endpoints"],
            ["python3", "-m", "atlasctl.cli", "--quiet", "contracts", "check", "--checks", "error-codes"],
            ["python3", "-m", "atlasctl.cli", "--quiet", "contracts", "check", "--checks", "breakage"],
        ],
        "docs": [["python3", "-m", "atlasctl.cli", "--quiet", "docs", "check", "--report", "text"]],
        "ops": [["python3", "-m", "atlasctl.cli", "--quiet", "ops", "check", "--report", "text"]],
        "dependency-lock-refresh": [
            ["cargo", "update", "--workspace"],
            ["cargo", "generate-lockfile"],
            ["cargo", "check", "--workspace", "--locked"],
        ],
        "deps": [
            [sys.executable, "-m", "atlasctl.cli", "--quiet", "ci", "dependency-lock-refresh", "--json"],
        ],
        "release-compat-matrix-verify": [
            ["python3", "-m", "atlasctl.cli", "--quiet", "ci", "init-tmp"],
            ["make", "-s", "release-update-compat-matrix", "TAG="],
            ["git", "diff", "--exit-code", "docs/reference/compatibility/umbrella-atlas-matrix.md"],
        ],
        "release-build-artifacts": [
            ["python3", "-m", "atlasctl.cli", "--quiet", "ci", "init-iso-dirs"],
            ["cargo", "build", "--locked", "--release", "--workspace", "--bins"],
            ["mkdir", "-p", "artifacts/release"],
            'cp "${CARGO_TARGET_DIR:-target}/release/atlas-server" artifacts/release/',
            'cp "${CARGO_TARGET_DIR:-target}/release/bijux-atlas" artifacts/release/',
        ],
        "release-notes-render": [
            'mkdir -p artifacts/isolate/release-notes && '
            'sed -e "s/{{tag}}/${GITHUB_REF_NAME}/g" '
            '-e "s/{{date}}/$(date -u +%Y-%m-%d)/g" '
            '-e "s/{{commit}}/${GITHUB_SHA}/g" '
            '.github/release-notes-template.md > artifacts/isolate/release-notes/RELEASE_NOTES.md'
        ],
        "release-publish-gh": [
            'gh release create "${GITHUB_REF_NAME}" --title "bijux-atlas ${GITHUB_REF_NAME}" '
            '--notes-file artifacts/isolate/release-notes/RELEASE_NOTES.md --verify-tag || '
            'gh release edit "${GITHUB_REF_NAME}" --title "bijux-atlas ${GITHUB_REF_NAME}" '
            '--notes-file artifacts/isolate/release-notes/RELEASE_NOTES.md'
        ],
        "cosign-sign": ['[ -n "${COSIGN_IMAGE_REF:-}" ] && cosign sign --yes "${COSIGN_IMAGE_REF}"'],
        "cosign-verify": [
            '[ -n "${COSIGN_IMAGE_REF:-}" ] && [ -n "${COSIGN_CERT_IDENTITY:-}" ] && '
            'cosign verify --certificate-identity-regexp "${COSIGN_CERT_IDENTITY}" '
            '--certificate-oidc-issuer "https://token.actions.githubusercontent.com" "${COSIGN_IMAGE_REF}"'
        ],
        "chart-package-release": [["helm", "package", "ops/k8s/charts/bijux-atlas", "--destination", ".cr-release-packages"]],
        "reproducible-verify": [
            'mkdir -p artifacts/isolate/reproducible-build && '
            'cargo build --release --locked --bin bijux-atlas --bin atlas-server && '
            'sha256sum "${CARGO_TARGET_DIR:-target}/release/bijux-atlas" "${CARGO_TARGET_DIR:-target}/release/atlas-server" > artifacts/isolate/reproducible-build/build1.sha256 && '
            'cargo clean && cargo build --release --locked --bin bijux-atlas --bin atlas-server && '
            'sha256sum "${CARGO_TARGET_DIR:-target}/release/bijux-atlas" "${CARGO_TARGET_DIR:-target}/release/atlas-server" > artifacts/isolate/reproducible-build/build2.sha256 && '
            'diff -u artifacts/isolate/reproducible-build/build1.sha256 artifacts/isolate/reproducible-build/build2.sha256'
        ],
        "security-advisory-render": [
            'mkdir -p artifacts/evidence/security/advisories && '
            'DATE_UTC="$(date -u +%Y-%m-%d)"; '
            'FILE="artifacts/evidence/security/advisories/${ADVISORY_ID}.md"; '
            "printf '%s\\n' "
            '"# Security Advisory ${ADVISORY_ID}" "" '
            '"- Published: ${DATE_UTC}" '
            '"- Severity: ${ADVISORY_SEVERITY}" '
            '"- Affected versions: ${ADVISORY_AFFECTED_VERSIONS}" '
            '"- Fixed version: ${ADVISORY_FIXED_VERSION}" "" '
            '"## Summary" "${ADVISORY_SUMMARY}" "" '
            '"## Mitigation" "Upgrade to `${ADVISORY_FIXED_VERSION}` or newer." > "${FILE}"'
        ],
        "governance-check": [
            ["make", "-s", "atlasctl-check-layout"],
            ["make", "-s", "docs-freeze"],
            ["make", "-s", "ssot-check"],
            ["make", "-s", "policy-enforcement-status"],
            ["make", "-s", "policy-allow-env-lint"],
            ["make", "-s", "scripts-lint"],
            ["make", "-s", "ops-policy-audit"],
            ["make", "-s", "ci-workflows-make-only"],
        ],
    }

    if cmd == "init-iso-dirs":
        paths = [
            env.get("CARGO_TARGET_DIR", "artifacts/isolate/tmp/target"),
            env.get("CARGO_HOME", "artifacts/isolate/tmp/cargo-home"),
            env.get("TMPDIR", "artifacts/isolate/tmp/tmp"),
            env.get("ISO_ROOT", "artifacts/isolate/tmp"),
        ]
        return _emit_result(ctx, ns, "init-iso-dirs", [_run_step(ctx, ["mkdir", "-p", *paths], verbose=verbose, env=env)])
    if cmd == "init-tmp":
        paths = [env.get("TMPDIR", "artifacts/isolate/tmp/tmp"), env.get("ISO_ROOT", "artifacts/isolate/tmp")]
        return _emit_result(ctx, ns, "init-tmp", [_run_step(ctx, ["mkdir", "-p", *paths], verbose=verbose, env=env)])

    steps_spec = steps_by_cmd.get(cmd)
    if steps_spec is None:
        return 2
    steps = [_run_step(ctx, step, verbose=verbose) for step in steps_spec]
    return _emit_result(ctx, ns, cmd, steps)
