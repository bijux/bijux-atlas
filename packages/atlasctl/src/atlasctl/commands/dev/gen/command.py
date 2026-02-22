from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path

from ....contracts.command import run_contracts_command
from ....core.context import RunContext
from ....core.exec import run
from ....core.runtime.paths import write_text_file
from ....checks.registry.ssot import generate_registry_json, legacy_checks, toml_entry_from_check, write_registry_toml
from ....checks.registry.catalog import check_tags

SELF_CLI = ["python3", "-m", "atlasctl.cli"]


def _run(ctx: RunContext, cmd: list[str]) -> int:
    proc = run(cmd, cwd=ctx.repo_root, text=True)
    return proc.returncode


def _run_capture(ctx: RunContext, args: list[str]):
    env = os.environ.copy()
    src_path = str(ctx.repo_root / "packages/atlasctl/src")
    existing = env.get("PYTHONPATH", "")
    env["PYTHONPATH"] = f"{src_path}:{existing}" if existing else src_path
    return run(
        [sys.executable, "-m", "atlasctl.cli", *args],
        cwd=ctx.repo_root,
        text=True,
        capture_output=True,
        env=env,
    )


def _write(path: Path, value: str) -> None:
    write_text_file(path, value, encoding="utf-8")


def _generate_goldens(ctx: RunContext) -> tuple[int, dict[str, str]]:
    targets: dict[str, list[str]] = {
        "help/help.json.golden": ["--quiet", "help", "--json"],
        "list/commands.json.golden": ["--quiet", "commands", "--json"],
        "contracts/surface.json.golden": ["--quiet", "surface", "--json"],
        "contracts/explain.check.json.golden": ["--quiet", "--json", "explain", "check"],
        "check/check-list.json.golden": ["--quiet", "--json", "check", "list"],
        "help/cli_help_snapshot.txt": ["--help"],
    }
    out_dir = ctx.repo_root / "packages/atlasctl/tests/goldens"
    written: dict[str, str] = {}
    for rel_name, cmd in targets.items():
        proc = _run_capture(ctx, cmd)
        if proc.returncode != 0:
            msg = proc.stderr.strip() or proc.stdout.strip() or f"failed to generate {rel_name}"
            return 1, {"error": f"{rel_name}: {msg}"}
        content = proc.stdout if rel_name.endswith(".txt") else proc.stdout.strip() + "\n"
        path = out_dir / rel_name
        _write(path, content)
        written[path.name] = path.relative_to(ctx.repo_root).as_posix()
    commands = json.loads((out_dir / "help/help.json.golden").read_text(encoding="utf-8"))["commands"]
    command_names = "\n".join(row["name"] for row in commands) + "\n"
    cmd_path = out_dir / "help/cli_help_commands.expected.txt"
    _write(cmd_path, command_names)
    written["cli_help_commands.expected.txt"] = cmd_path.relative_to(ctx.repo_root).as_posix()
    entries: list[dict[str, str]] = []
    for path in sorted(out_dir.rglob("*")):
        if not path.is_file():
            continue
        rel = path.relative_to(out_dir).as_posix()
        if rel == "MANIFEST.json" or rel.startswith("__pycache__/"):
            continue
        entries.append({"name": path.name, "path": rel})
    manifest = {"schema_version": 1, "generated_by": "atlasctl", "entries": entries}
    _write(out_dir / "MANIFEST.json", json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    written["MANIFEST.json"] = (out_dir / "MANIFEST.json").relative_to(ctx.repo_root).as_posix()
    return 0, written


def run_gen_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    sub = ns.gen_cmd
    if sub == "checks-registry":
        checks = sorted(legacy_checks(), key=lambda c: c.check_id)
        rows = sorted((toml_entry_from_check(check, groups=check_tags(check)) for check in checks), key=lambda row: str(row.get("id", "")))
        write_registry_toml(ctx.repo_root, rows)
        out, _changed = generate_registry_json(ctx.repo_root, check_only=False)
        print(json.dumps({"schema_version": 1, "tool": "atlasctl", "status": "ok", "registry_toml": "packages/atlasctl/src/atlasctl/checks/REGISTRY.toml", "registry_json": str(out.relative_to(ctx.repo_root))}, sort_keys=True))
        return 0
    if sub == "contracts":
        return run_contracts_command(
            ctx,
            argparse.Namespace(contracts_cmd="generate", report="text", generators=["artifacts"]),
        )
    if sub == "openapi":
        return run_contracts_command(
            ctx,
            argparse.Namespace(contracts_cmd="generate", report="text", generators=["openapi"]),
        )
    if sub == "ops-surface":
        return _run(ctx, ["python3", "packages/atlasctl/src/atlasctl/checks/layout/ops/generation/generate_ops_surface_meta.py"])
    if sub == "make-targets":
        return _run(ctx, [*SELF_CLI, "make", "inventory"])
    if sub == "surface":
        return _run(ctx, ["python3", "-m", "atlasctl.cli", "docs", "generate-repo-surface", "--report", "text"])
    if sub == "scripting-surface":
        out = ctx.repo_root / "docs/_generated/scripts-surface.md"
        lines = [
            "# Scripts Surface",
            "",
            "Generated file. Do not edit manually.",
            "",
            "## scripts/bin",
            "",
        ]
        for p in sorted((ctx.repo_root / "scripts/bin").glob("*")):
            if p.is_file():
                lines.append(f"- `{p.relative_to(ctx.repo_root).as_posix()}`")
        lines.extend(["", "## checks", ""])
        for p in sorted((ctx.repo_root / "scripts/areas/check").glob("*")):
            if p.is_file():
                lines.append(f"- `{p.relative_to(ctx.repo_root).as_posix()}`")
        lines.extend(["", "## root bin shims", ""])
        for p in sorted((ctx.repo_root / "bin").glob("*")):
            if p.is_file():
                lines.append(f"- `{p.relative_to(ctx.repo_root).as_posix()}`")
        write_text_file(out, "\n".join(lines) + "\n", encoding="utf-8")
        print(json.dumps({"status": "pass", "file": str(out)}, sort_keys=True))
        return 0
    if sub == "goldens":
        code, written = _generate_goldens(ctx)
        payload = {"schema_version": 1, "tool": "atlasctl", "status": "ok" if code == 0 else "error", "written": written}
        print(json.dumps(payload, sort_keys=True))
        return code
    return 2


def configure_gen_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("gen", help="generation commands mapped from scripts/areas")
    p_sub = p.add_subparsers(dest="gen_cmd", required=True)
    p_sub.add_parser("contracts", help="generate contracts artifacts")
    p_sub.add_parser("openapi", help="generate openapi snapshot and telemetry artifacts")
    p_sub.add_parser("goldens", help="generate test golden snapshots under packages/atlasctl/tests/goldens")
    p_sub.add_parser("ops-surface", help="generate ops surface metadata")
    p_sub.add_parser("make-targets", help="generate make targets inventory artifacts")
    p_sub.add_parser("surface", help="generate repo public surface artifacts")
    p_sub.add_parser("scripting-surface", help="generate scripts/CLI surface artifacts")
    p_sub.add_parser("checks-registry", help="generate checks REGISTRY.toml and REGISTRY.generated.json")
