from __future__ import annotations

import argparse
import json
import os
import subprocess
from pathlib import Path
from typing import Any

from ....contracts.output.base import build_output_base
from ....core.context import RunContext


def _run_step(
    ctx: RunContext,
    cmd: list[str] | str,
    *,
    verbose: bool,
    env: dict[str, str] | None = None,
) -> dict[str, Any]:
    shell = isinstance(cmd, str)
    display = cmd if isinstance(cmd, str) else " ".join(cmd)
    if verbose:
        proc = subprocess.run(cmd, cwd=ctx.repo_root, env=env, text=True, shell=shell, check=False)
        return {"command": display, "exit_code": proc.returncode}
    proc = subprocess.run(cmd, cwd=ctx.repo_root, env=env, text=True, shell=shell, capture_output=True, check=False)
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
        print(f"ci {action}: fail")
    return 0 if ok else 1


def run_ci_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    verbose = bool(getattr(ns, "verbose", False) or ctx.verbose)
    if ns.ci_cmd == "scripts":
        step = _run_step(ctx, ["make", "-s", "scripts-check"], verbose=verbose)
        return _emit_result(ctx, ns, "scripts", [step])
    if ns.ci_cmd == "run":
        out_dir = Path(ns.out_dir) if ns.out_dir else (ctx.repo_root / "artifacts" / "evidence" / "ci" / ctx.run_id)
        out_dir.mkdir(parents=True, exist_ok=True)
        junit_path = out_dir / "suite-ci.junit.xml"
        proc = subprocess.run(
            [
                "python3",
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
            ],
            cwd=ctx.repo_root,
            text=True,
            capture_output=True,
            check=False,
        )
        payload = json.loads(proc.stdout) if proc.stdout.strip() else {"status": "error", "summary": {"passed": 0, "failed": 1, "skipped": 0}}
        (out_dir / "suite-ci.report.json").write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        summary = payload.get("summary", {})
        summary_txt = (
            f"run_id={ctx.run_id}\n"
            f"status={payload.get('status','error')}\n"
            f"passed={summary.get('passed', 0)} failed={summary.get('failed', 0)} skipped={summary.get('skipped', 0)}\n"
            f"junit={junit_path}\n"
            f"json={out_dir / 'suite-ci.report.json'}\n"
        )
        (out_dir / "suite-ci.summary.txt").write_text(summary_txt, encoding="utf-8")
        if ns.json or ctx.output_format == "json":
            print(
                json.dumps(
                    {
                        "schema_version": 1,
                        "tool": "atlasctl",
                        "status": "ok" if proc.returncode == 0 else "error",
                        "run_id": ctx.run_id,
                        "suite_result": payload,
                        "artifacts": {
                            "json": str(out_dir / "suite-ci.report.json"),
                            "junit": str(junit_path),
                            "summary": str(out_dir / "suite-ci.summary.txt"),
                        },
                    },
                    sort_keys=True,
                )
            )
        elif proc.returncode == 0:
            print(f"ci run: pass (suite ci) run_id={ctx.run_id}")
        else:
            print(f"ci run: fail (suite ci) run_id={ctx.run_id}")
        return proc.returncode
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
            'mkdir -p docs/security/advisories && '
            'DATE_UTC="$(date -u +%Y-%m-%d)"; '
            'FILE="docs/security/advisories/${ADVISORY_ID}.md"; '
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
    ci_sub.add_parser("scripts", help="run scripts ci checks")
    run = ci_sub.add_parser("run", help="run canonical CI suite locally")
    run.add_argument("--json", action="store_true", help="emit JSON output")
    run.add_argument("--out-dir", help="output directory for CI artifacts")
    run.add_argument("--verbose", action="store_true", help="show underlying tool command output")
    for name in (
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
