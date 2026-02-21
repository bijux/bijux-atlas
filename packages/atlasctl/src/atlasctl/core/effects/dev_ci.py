from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Any

from ...contracts.output.base import build_output_base
from ..context import RunContext
from .run_meta import write_run_meta

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
    ("ci-pr", "PR checks lane (fast checks only)", "atlasctl check run --group all --json", "lane", "pr"),
    ("ci-nightly", "Nightly checks lane (includes slow)", "atlasctl check run --group all --all --json", "lane", "nightly"),
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
    shell = isinstance(cmd, str)
    display = cmd if isinstance(cmd, str) else " ".join(cmd)
    try:
        if verbose:
            proc = subprocess.run(cmd, cwd=ctx.repo_root, env=env, text=True, shell=shell, check=False)
            return {"command": display, "exit_code": proc.returncode}
        proc = subprocess.run(cmd, cwd=ctx.repo_root, env=env, text=True, shell=shell, capture_output=True, check=False)
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


def run_ci_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    verbose = bool(getattr(ns, "verbose", False) or ctx.verbose)
    if ns.ci_cmd == "list":
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
    if ns.ci_cmd == "all":
        step = _run_step(
            ctx,
            [
                sys.executable,
                "-m",
                "atlasctl.cli",
                "--quiet",
                "ci",
                "run",
                "--json",
                "--keep-going",
            ],
            verbose=verbose,
        )
        return _emit_result(ctx, ns, "all", [step])
    if ns.ci_cmd == "init":
        steps = [
            _run_step(ctx, [sys.executable, "-m", "atlasctl.cli", "--quiet", "ci", "init-iso-dirs", "--json"], verbose=verbose),
            _run_step(ctx, [sys.executable, "-m", "atlasctl.cli", "--quiet", "ci", "init-tmp", "--json"], verbose=verbose),
        ]
        return _emit_result(ctx, ns, "init", steps)
    if ns.ci_cmd == "artifacts":
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
    if ns.ci_cmd == "release":
        steps = [
            _run_step(ctx, [sys.executable, "-m", "atlasctl.cli", "--quiet", "ci", "release-compat-matrix-verify", "--json"], verbose=verbose),
            _run_step(ctx, [sys.executable, "-m", "atlasctl.cli", "--quiet", "ci", "release-build-artifacts", "--json"], verbose=verbose),
            _run_step(ctx, [sys.executable, "-m", "atlasctl.cli", "--quiet", "ci", "release-notes-render", "--json"], verbose=verbose),
        ]
        return _emit_result(ctx, ns, "release", steps)
    if ns.ci_cmd == "release-all":
        steps = [
            _run_step(ctx, [sys.executable, "-m", "atlasctl.cli", "--quiet", "ci", "release", "--json"], verbose=verbose),
            _run_step(ctx, [sys.executable, "-m", "atlasctl.cli", "--quiet", "ci", "release-publish-gh", "--json"], verbose=verbose),
            _run_step(ctx, [sys.executable, "-m", "atlasctl.cli", "--quiet", "ci", "reproducible-verify", "--json"], verbose=verbose),
            _run_step(ctx, [sys.executable, "-m", "atlasctl.cli", "--quiet", "ci", "security-advisory-render", "--json"], verbose=verbose),
        ]
        return _emit_result(ctx, ns, "release-all", steps)
    if ns.ci_cmd == "scripts":
        step = _run_step(ctx, ["make", "-s", "scripts-check"], verbose=verbose)
        return _emit_result(ctx, ns, "scripts", [step])
    if ns.ci_cmd == "pr":
        step = _run_step(
            ctx,
            [sys.executable, "-m", "atlasctl.cli", "--quiet", "--format", "json", "check", "run", "--group", "all", "--json"],
            verbose=verbose,
        )
        return _emit_result(ctx, ns, "pr", [step])
    if ns.ci_cmd == "nightly":
        step = _run_step(
            ctx,
            [sys.executable, "-m", "atlasctl.cli", "--quiet", "--format", "json", "check", "run", "--group", "all", "--all", "--json"],
            verbose=verbose,
        )
        return _emit_result(ctx, ns, "nightly", [step])
    if ns.ci_cmd == "run":
        out_dir = _ci_out_dir(ctx, getattr(ns, "out_dir", None))
        out_dir.mkdir(parents=True, exist_ok=True)
        junit_path = out_dir / "suite-ci.junit.xml"
        suite_cmd: list[str] = [
            sys.executable,
            "-m",
            "atlasctl.cli",
            "--quiet",
            "--format",
            "json",
            "--run-id",
            ctx.run_id,
            "suite",
            "run",
            "ci",
            "--json",
            "--junit",
            str(junit_path),
        ]
        lanes = list(getattr(ns, "lane", []) or [])
        if lanes:
            seen: set[str] = set()
            only_patterns: list[str] = []
            for lane in lanes:
                values = LANE_FILTERS.get(lane)
                if not values:
                    continue
                for pattern in values:
                    if pattern in seen:
                        continue
                    seen.add(pattern)
                    only_patterns.append(pattern)
            for pattern in only_patterns:
                suite_cmd.extend(["--only", pattern])
        if bool(getattr(ns, "fail_fast", False)):
            suite_cmd.append("--fail-fast")
        else:
            suite_cmd.append("--keep-going")
        execution_mode = "fail-fast" if bool(getattr(ns, "fail_fast", False)) else "keep-going"
        isolate_mode = "debug-no-isolate" if bool(getattr(ns, "no_isolate", False)) else "isolate"
        planned_cmd = suite_cmd if bool(getattr(ns, "no_isolate", False)) else [
            sys.executable,
            "-m",
            "atlasctl.cli",
            "env",
            "isolate",
            "--tag",
            f"ci-{ctx.run_id}",
            *suite_cmd,
        ]
        if bool(getattr(ns, "explain", False)):
            explain_payload = {
                "schema_version": 1,
                "tool": "atlasctl",
                "status": "ok",
                "run_id": ctx.run_id,
                "action": "ci-run-explain",
                "lane_filter": lanes or ["all"],
                "mode": isolate_mode,
                "execution": execution_mode,
                "artifacts": {
                    "json": str(out_dir / "suite-ci.report.json"),
                    "junit": str(junit_path),
                    "summary": str(out_dir / "suite-ci.summary.txt"),
                },
                "planned_steps": [{"id": "ci.step.001", "command": " ".join(planned_cmd)}],
            }
            if ns.json or ctx.output_format == "json":
                print(json.dumps(explain_payload, sort_keys=True))
            else:
                print(
                    "ci run plan: "
                    f"mode={isolate_mode} execution={execution_mode} lanes={','.join(lanes) if lanes else 'all'}"
                )
                print(f"- {' '.join(planned_cmd)}")
            return 0

        if bool(getattr(ns, "no_isolate", False)):
            proc = subprocess.run(
                suite_cmd,
                cwd=ctx.repo_root,
                text=True,
                capture_output=True,
                check=False,
            )
        else:
            tag = f"ci-{ctx.run_id}"
            proc = subprocess.run(
                [
                    sys.executable,
                    "-m",
                    "atlasctl.cli",
                    "env",
                    "isolate",
                    "--tag",
                    tag,
                    *suite_cmd,
                ],
                cwd=ctx.repo_root,
                text=True,
                capture_output=True,
                check=False,
            )
        if proc.stdout.strip():
            try:
                payload = json.loads(proc.stdout)
            except json.JSONDecodeError:
                payload = {
                    "status": "error",
                    "summary": {"passed": 0, "failed": 1, "skipped": 0},
                    "errors": [
                        "runtime error: suite output was not valid JSON. "
                        "Next: rerun with `atlasctl dev ci run --verbose --no-isolate`."
                    ],
                }
        else:
            payload = {"status": "error", "summary": {"passed": 0, "failed": 1, "skipped": 0}}
        suite_steps = [
            {
                "id": f"ci.step.{idx:03d}",
                "lane": _lane_for_label(str(row.get("label", ""))),
                "label": str(row.get("label", "")),
                "status": str(row.get("status", "unknown")),
            }
            for idx, row in enumerate(payload.get("results", []), start=1)
        ]
        (out_dir / "suite-ci.report.json").write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        summary = payload.get("summary", {})
        summary_txt = (
            f"run_id={ctx.run_id}\n"
            f"status={payload.get('status','error')}\n"
            f"passed={summary.get('passed', 0)} failed={summary.get('failed', 0)} skipped={summary.get('skipped', 0)}\n"
            f"lanes={','.join(lanes) if lanes else 'all'}\n"
            f"junit={junit_path}\n"
            f"json={out_dir / 'suite-ci.report.json'}\n"
        )
        (out_dir / "suite-ci.summary.txt").write_text(summary_txt, encoding="utf-8")
        meta_path = write_run_meta(ctx, out_dir, lane="ci")
        if ns.json or ctx.output_format == "json":
            print(
                json.dumps(
                    {
                        "schema_version": 1,
                        "tool": "atlasctl",
                        "status": "ok" if proc.returncode == 0 else "error",
                        "run_id": ctx.run_id,
                        "lane_filter": lanes or ["all"],
                        "mode": isolate_mode,
                        "execution": execution_mode,
                        "suite_result": payload,
                        "suite_steps": suite_steps,
                        "next": (
                            "rerun with `atlasctl dev ci run --verbose --no-isolate` for step-level diagnostics"
                            if proc.returncode != 0
                            else ""
                        ),
                        "artifacts": {
                            "json": str(out_dir / "suite-ci.report.json"),
                            "junit": str(junit_path),
                            "summary": str(out_dir / "suite-ci.summary.txt"),
                            "meta": str(meta_path),
                        },
                    },
                    sort_keys=True,
                )
            )
        elif proc.returncode == 0:
            print(f"ci run: pass (suite ci) run_id={ctx.run_id}")
        else:
            print(f"ci run: fail (suite ci) run_id={ctx.run_id} (next: rerun with --verbose --no-isolate)")
        return 0 if proc.returncode == 0 else 1
    if ns.ci_cmd == "report":
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
            "artifacts": {
                "json": str(report),
                "summary": str(summary),
                "meta": str(meta),
            },
        }
        if ns.json or ctx.output_format == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            print(f"latest ci report: {latest.name}")
            print(f"- json: {report}")
            print(f"- summary: {summary}")
            print(f"- meta: {meta}")
        return 0
    if ns.ci_cmd == "fast":
        step = _run_step(
            ctx,
            ["python3", "-m", "atlasctl.cli", "--quiet", "suite", "run", "fast", "--json"],
            verbose=verbose,
        )
        return _emit_result(ctx, ns, "fast", [step])
    if ns.ci_cmd == "contracts":
        steps = [
            _run_step(ctx, ["python3", "-m", "atlasctl.cli", "--quiet", "contracts", "check", "--checks", "endpoints"], verbose=verbose),
            _run_step(ctx, ["python3", "-m", "atlasctl.cli", "--quiet", "contracts", "check", "--checks", "error-codes"], verbose=verbose),
            _run_step(ctx, ["python3", "-m", "atlasctl.cli", "--quiet", "contracts", "check", "--checks", "breakage"], verbose=verbose),
        ]
        return _emit_result(ctx, ns, "contracts", steps)
    if ns.ci_cmd == "docs":
        step = _run_step(ctx, ["python3", "-m", "atlasctl.cli", "--quiet", "docs", "check", "--report", "text"], verbose=verbose)
        return _emit_result(ctx, ns, "docs", [step])
    if ns.ci_cmd == "ops":
        step = _run_step(ctx, ["python3", "-m", "atlasctl.cli", "--quiet", "ops", "check", "--report", "text"], verbose=verbose)
        return _emit_result(ctx, ns, "ops", [step])
    if ns.ci_cmd == "init-iso-dirs":
        env = os.environ.copy()
        iso_root = env.get("ISO_ROOT", "artifacts/isolate/tmp")
        paths = [
            env.get("CARGO_TARGET_DIR", "artifacts/isolate/tmp/target"),
            env.get("CARGO_HOME", "artifacts/isolate/tmp/cargo-home"),
            env.get("TMPDIR", "artifacts/isolate/tmp/tmp"),
            iso_root,
        ]
        step = _run_step(ctx, ["mkdir", "-p", *paths], verbose=verbose, env=env)
        return _emit_result(ctx, ns, "init-iso-dirs", [step])
    if ns.ci_cmd == "init-tmp":
        env = os.environ.copy()
        paths = [
            env.get("TMPDIR", "artifacts/isolate/tmp/tmp"),
            env.get("ISO_ROOT", "artifacts/isolate/tmp"),
        ]
        step = _run_step(ctx, ["mkdir", "-p", *paths], verbose=verbose, env=env)
        return _emit_result(ctx, ns, "init-tmp", [step])
    if ns.ci_cmd == "dependency-lock-refresh":
        steps = [
            _run_step(ctx, ["cargo", "update", "--workspace"], verbose=verbose),
            _run_step(ctx, ["cargo", "generate-lockfile"], verbose=verbose),
            _run_step(ctx, ["cargo", "check", "--workspace", "--locked"], verbose=verbose),
        ]
        return _emit_result(ctx, ns, "dependency-lock-refresh", steps)
    if ns.ci_cmd == "release-compat-matrix-verify":
        steps = [
            _run_step(ctx, ["python3", "-m", "atlasctl.cli", "--quiet", "ci", "init-tmp"], verbose=verbose),
            _run_step(ctx, ["make", "-s", "release-update-compat-matrix", "TAG="], verbose=verbose),
            _run_step(ctx, ["git", "diff", "--exit-code", "docs/reference/compatibility/umbrella-atlas-matrix.md"], verbose=verbose),
        ]
        return _emit_result(ctx, ns, "release-compat-matrix-verify", steps)
    if ns.ci_cmd == "release-build-artifacts":
        steps = [
            _run_step(ctx, ["python3", "-m", "atlasctl.cli", "--quiet", "ci", "init-iso-dirs"], verbose=verbose),
            _run_step(ctx, ["cargo", "build", "--locked", "--release", "--workspace", "--bins"], verbose=verbose),
            _run_step(ctx, ["mkdir", "-p", "artifacts/release"], verbose=verbose),
            _run_step(ctx, 'cp "${CARGO_TARGET_DIR:-target}/release/atlas-server" artifacts/release/', verbose=verbose),
            _run_step(ctx, 'cp "${CARGO_TARGET_DIR:-target}/release/bijux-atlas" artifacts/release/', verbose=verbose),
        ]
        return _emit_result(ctx, ns, "release-build-artifacts", steps)
    if ns.ci_cmd == "release-notes-render":
        step = _run_step(
            ctx,
            'mkdir -p artifacts/isolate/release-notes && '
            'sed -e "s/{{tag}}/${GITHUB_REF_NAME}/g" '
            '-e "s/{{date}}/$(date -u +%Y-%m-%d)/g" '
            '-e "s/{{commit}}/${GITHUB_SHA}/g" '
            '.github/release-notes-template.md > artifacts/isolate/release-notes/RELEASE_NOTES.md',
            verbose=verbose,
        )
        return _emit_result(ctx, ns, "release-notes-render", [step])
    if ns.ci_cmd == "release-publish-gh":
        step = _run_step(
            ctx,
            'gh release create "${GITHUB_REF_NAME}" --title "bijux-atlas ${GITHUB_REF_NAME}" '
            '--notes-file artifacts/isolate/release-notes/RELEASE_NOTES.md --verify-tag || '
            'gh release edit "${GITHUB_REF_NAME}" --title "bijux-atlas ${GITHUB_REF_NAME}" '
            '--notes-file artifacts/isolate/release-notes/RELEASE_NOTES.md',
            verbose=verbose,
        )
        return _emit_result(ctx, ns, "release-publish-gh", [step])
    if ns.ci_cmd == "cosign-sign":
        step = _run_step(ctx, '[ -n "${COSIGN_IMAGE_REF:-}" ] && cosign sign --yes "${COSIGN_IMAGE_REF}"', verbose=verbose)
        return _emit_result(ctx, ns, "cosign-sign", [step])
    if ns.ci_cmd == "cosign-verify":
        step = _run_step(
            ctx,
            '[ -n "${COSIGN_IMAGE_REF:-}" ] && [ -n "${COSIGN_CERT_IDENTITY:-}" ] && '
            'cosign verify --certificate-identity-regexp "${COSIGN_CERT_IDENTITY}" '
            '--certificate-oidc-issuer "https://token.actions.githubusercontent.com" "${COSIGN_IMAGE_REF}"',
            verbose=verbose,
        )
        return _emit_result(ctx, ns, "cosign-verify", [step])
    if ns.ci_cmd == "chart-package-release":
        step = _run_step(ctx, ["helm", "package", "ops/k8s/charts/bijux-atlas", "--destination", ".cr-release-packages"], verbose=verbose)
        return _emit_result(ctx, ns, "chart-package-release", [step])
    if ns.ci_cmd == "reproducible-verify":
        step = _run_step(
            ctx,
            'mkdir -p artifacts/isolate/reproducible-build && '
            'cargo build --release --locked --bin bijux-atlas --bin atlas-server && '
            'sha256sum "${CARGO_TARGET_DIR:-target}/release/bijux-atlas" "${CARGO_TARGET_DIR:-target}/release/atlas-server" > artifacts/isolate/reproducible-build/build1.sha256 && '
            'cargo clean && cargo build --release --locked --bin bijux-atlas --bin atlas-server && '
            'sha256sum "${CARGO_TARGET_DIR:-target}/release/bijux-atlas" "${CARGO_TARGET_DIR:-target}/release/atlas-server" > artifacts/isolate/reproducible-build/build2.sha256 && '
            'diff -u artifacts/isolate/reproducible-build/build1.sha256 artifacts/isolate/reproducible-build/build2.sha256',
            verbose=verbose,
        )
        return _emit_result(ctx, ns, "reproducible-verify", [step])
    if ns.ci_cmd == "security-advisory-render":
        step = _run_step(
            ctx,
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
            '"## Mitigation" "Upgrade to `${ADVISORY_FIXED_VERSION}` or newer." > "${FILE}"',
            verbose=verbose,
        )
        return _emit_result(ctx, ns, "security-advisory-render", [step])
    if ns.ci_cmd == "governance-check":
        steps = [
            _run_step(ctx, ["make", "-s", "layout-check"], verbose=verbose),
            _run_step(ctx, ["make", "-s", "docs-freeze"], verbose=verbose),
            _run_step(ctx, ["make", "-s", "ssot-check"], verbose=verbose),
            _run_step(ctx, ["make", "-s", "policy-enforcement-status"], verbose=verbose),
            _run_step(ctx, ["make", "-s", "policy-allow-env-lint"], verbose=verbose),
            _run_step(ctx, ["make", "-s", "scripts-lint"], verbose=verbose),
            _run_step(ctx, ["make", "-s", "ops-policy-audit"], verbose=verbose),
            _run_step(ctx, ["make", "-s", "ci-workflows-make-only"], verbose=verbose),
        ]
        return _emit_result(ctx, ns, "governance-check", steps)
    return 2


def configure_ci_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("ci", help="ci command group")
    p.add_argument("--verbose", action="store_true", help="show underlying tool command output")
    ci_sub = p.add_subparsers(dest="ci_cmd", required=True)
    ls = ci_sub.add_parser("list", help="list canonical CI lanes")
    ls.add_argument("--json", action="store_true", help="emit JSON output")
    ls.add_argument("--verbose", action="store_true", help="show underlying tool command output")
    ci_sub.add_parser("scripts", help="run scripts ci checks")
    run = ci_sub.add_parser("run", help="run canonical CI suite locally")
    run.add_argument("--json", action="store_true", help="emit JSON output")
    run.add_argument("--out-dir", help="output directory for CI artifacts")
    run.add_argument("--lane", action="append", choices=sorted(LANE_FILTERS.keys()), help="restrict suite run to a logical lane")
    mode = run.add_mutually_exclusive_group()
    mode.add_argument("--fail-fast", action="store_true", help="stop at first failing suite step")
    mode.add_argument("--keep-going", action="store_true", help="continue through all suite steps (default)")
    run.add_argument("--no-isolate", action="store_true", help="debug only: skip isolate wrapper around suite execution")
    run.add_argument("--explain", action="store_true", help="print planned CI run steps without executing")
    run.add_argument("--verbose", action="store_true", help="show underlying tool command output")
    for name in (
        "all",
        "init",
        "artifacts",
        "release",
        "release-all",
        "fast",
        "contracts",
        "docs",
        "ops",
        "init-iso-dirs",
        "init-tmp",
        "dependency-lock-refresh",
        "release-compat-matrix-verify",
        "release-build-artifacts",
        "release-notes-render",
        "release-publish-gh",
        "cosign-sign",
        "cosign-verify",
        "chart-package-release",
        "reproducible-verify",
        "security-advisory-render",
        "governance-check",
    ):
        sp = ci_sub.add_parser(name, help=f"run ci action: {name}")
        sp.add_argument("--json", action="store_true", help="emit JSON output")
        sp.add_argument("--verbose", action="store_true", help="show underlying tool command output")
