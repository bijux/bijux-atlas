from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

from ..core.context import RunContext
from ..core.fs import ensure_evidence_path
from .contracts_check import run_contracts_check
from .explain import LEGACY_TARGET_RE
from .help import render_advanced, render_all, render_gates, render_help, render_list
from .public_targets import entry_map, load_ownership, public_entries, public_names
from .target_graph import parse_make_targets, render_tree


def _iter_makefiles(repo_root: Path) -> list[Path]:
    return [repo_root / "Makefile", *sorted((repo_root / "makefiles").glob("*.mk"))]


def _inventory_direct_make_logic(repo_root: Path) -> dict[str, object]:
    rows: list[dict[str, object]] = []
    files = _iter_makefiles(repo_root)
    current_target = ""
    for path in files:
        rel = path.relative_to(repo_root).as_posix()
        for lineno, line in enumerate(path.read_text(encoding="utf-8", errors="ignore").splitlines(), start=1):
            if line.startswith("\t"):
                body = line.strip()
                if not body:
                    continue
                if "atlasctl" in body or "$(ATLAS_SCRIPTS)" in body:
                    continue
                rows.append({"target": current_target or "<unknown>", "file": rel, "line": lineno, "recipe": body})
                continue
            if ":" in line and not line.startswith("#"):
                current_target = line.split(":", 1)[0].strip()
    return {"schema_version": 1, "tool": "atlasctl", "status": "ok", "items": rows}


def _load_run_allowlist(repo_root: Path) -> set[str]:
    cfg = repo_root / "configs/make/run-allowlist.json"
    if not cfg.exists():
        return set()
    payload = json.loads(cfg.read_text(encoding="utf-8"))
    rows = payload.get("allow_make_run_targets", [])
    if not isinstance(rows, list):
        return set()
    return {str(item).strip() for item in rows if str(item).strip()}


def _is_public_or_allowlisted_target(repo_root: Path, target: str) -> bool:
    return target in set(public_names()) or target in _load_run_allowlist(repo_root)


def _run_make_lint(repo_root: Path, as_json: bool) -> int:
    from ..checks.make import CHECKS

    rows: list[dict[str, object]] = []
    for check in CHECKS:
        code, errors = check.fn(repo_root)
        rows.append(
            {
                "id": check.check_id,
                "title": check.title,
                "status": "pass" if code == 0 else "fail",
                "errors": list(errors),
            }
        )
    failed = [row for row in rows if row["status"] == "fail"]
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok" if not failed else "fail",
        "total_count": len(rows),
        "failed_count": len(failed),
        "checks": rows,
    }
    if as_json:
        print(json.dumps(payload, sort_keys=True))
    else:
        print(f"make lint: {payload['status']} ({payload['failed_count']}/{payload['total_count']} failed)")
        for row in failed:
            print(f"- FAIL {row['id']}")
            for err in row["errors"][:8]:
                print(f"  - {err}")
    return 0 if not failed else 1


def _rewrite_makefiles(repo_root: Path, write: bool, limit: int) -> tuple[int, dict[str, object]]:
    replacements: list[dict[str, object]] = []
    patterns = [
        ("python3 ./packages/atlasctl/src/atlasctl/", "$(ATLAS_SCRIPTS) run ./packages/atlasctl/src/atlasctl/"),
        ("bash ops/", "$(ATLAS_SCRIPTS) run ./ops/"),
        ("sh ops/", "$(ATLAS_SCRIPTS) run ./ops/"),
    ]
    for path in _iter_makefiles(repo_root):
        original = path.read_text(encoding="utf-8", errors="ignore")
        updated = original
        file_changes = 0
        for before, after in patterns:
            count = updated.count(before)
            if count:
                updated = updated.replace(before, after)
                file_changes += count
        if file_changes == 0:
            continue
        rel = path.relative_to(repo_root).as_posix()
        replacements.append({"file": rel, "replacements": file_changes})
        if write:
            path.write_text(updated, encoding="utf-8")
        if len(replacements) >= limit:
            break
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok",
        "write": write,
        "changed_files": len(replacements),
        "items": replacements,
    }
    return 0, payload


def run_make_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    entries = public_entries()
    if ns.make_cmd == "help":
        if ns.mode == "gates":
            render_gates(entries)
        elif ns.mode == "list":
            render_list(entries)
        elif ns.mode == "advanced":
            render_advanced(entries)
        elif ns.mode == "all":
            render_all()
        else:
            render_help(entries)
        return 0

    if ns.make_cmd == "list":
        render_list(entries)
        return 0

    if ns.make_cmd == "lint":
        return _run_make_lint(ctx.repo_root, ns.json or ctx.output_format == "json")

    if ns.make_cmd == "rewrite":
        code, payload = _rewrite_makefiles(ctx.repo_root, bool(ns.write), int(ns.limit))
        if ns.json or ctx.output_format == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            mode = "applied" if ns.write else "preview"
            print(f"make rewrite ({mode}): changed_files={payload['changed_files']}")
            for item in payload["items"]:
                print(f"- {item['file']}: replacements={item['replacements']}")
        return code

    if ns.make_cmd == "list-public-targets":
        rows = [
            {
                "name": entry["name"],
                "description": entry.get("description", ""),
                "lanes": list(entry.get("lanes", [])),
            }
            for entry in sorted(entries, key=lambda row: str(row.get("name", "")))
        ]
        payload = {"schema_version": 1, "tool": "atlasctl", "status": "ok", "targets": rows}
        print(json.dumps(payload, sort_keys=True) if ns.json or ctx.output_format == "json" else "\n".join(f"- {row['name']}: {row['description']}" for row in rows))
        return 0

    if ns.make_cmd == "surface":
        payload = _surface_payload()
        if ns.pretty:
            print("Public Make Targets:")
            for row in payload["targets"]:
                print(f"- {row['name']} ({row['owner']}): {row['description']}")
        else:
            print(json.dumps(payload, sort_keys=True))
        return 0

    if ns.make_cmd == "inventory":
        payload = _surface_payload()
        out_dir = Path(ns.out_dir)
        if not out_dir.is_absolute():
            out_dir = (ctx.repo_root / out_dir).resolve()
        out_dir.mkdir(parents=True, exist_ok=True)

        json_path = out_dir / "make-targets.json"
        md_path = out_dir / "make-targets.md"
        json_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        lines = [
            "# Make Targets Inventory",
            "",
            "Generated by `atlasctl make inventory`.",
            "",
            "## Public Make Targets",
            "",
        ]
        for row in sorted(payload["targets"], key=lambda r: str(r["name"])):
            lines.append(f"- `{row['name']}` ({row['owner']}): {row['description']}")
        md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

        if ns.check:
            md_rel = str(md_path.relative_to(ctx.repo_root))
            json_rel = str(json_path.relative_to(ctx.repo_root))
            tracked = subprocess.run(
                ["git", "diff", "--", md_rel, json_rel],
                cwd=ctx.repo_root,
                text=True,
                capture_output=True,
                check=False,
            )
            if tracked.returncode != 0:
                print(tracked.stdout + tracked.stderr)
                return 1

            budget_cfg = ctx.repo_root / "configs/make/public-targets.json"
            if budget_cfg.exists():
                cfg = json.loads(budget_cfg.read_text(encoding="utf-8"))
                max_targets = int(cfg.get("max_public_targets", 20))
                count_targets = len([t for t in payload["targets"] if t.get("name") != "[global]"])
                if count_targets > max_targets:
                    print(
                        json.dumps(
                            {
                                "status": "fail",
                                "reason": "public target budget exceeded",
                                "count": count_targets,
                                "max": max_targets,
                            },
                            sort_keys=True,
                        )
                    )
                    return 1

            docs_path = ctx.repo_root / "docs/development/make-targets.md"
            if docs_path.exists():
                docs_text = docs_path.read_text(encoding="utf-8")
                missing = [
                    str(row["name"])
                    for row in payload["targets"]
                    if row.get("name") != "[global]" and f"- `{row['name']}`" not in docs_text
                ]
                if missing:
                    print(
                        json.dumps(
                            {
                                "status": "fail",
                                "reason": "docs coverage missing for public targets",
                                "missing": sorted(missing),
                            },
                            sort_keys=True,
                        )
                    )
                    return 1

        print(json.dumps({"status": "pass", "json": str(json_path), "md": str(md_path)}, sort_keys=True))
        return 0

    if ns.make_cmd == "catalog":
        inv_ns = argparse.Namespace(make_cmd="inventory", out_dir=ns.out_dir, check=ns.check)
        return run_make_command(ctx, inv_ns)

    if ns.make_cmd == "explain":
        target = ns.target
        if LEGACY_TARGET_RE.search(target):
            print(f"legacy target names are forbidden: {target}")
            return 2
        entries = entry_map()
        if target not in entries:
            print(f"not public: {target}")
            return 1
        entry = entries[target]
        print(f"target: {target}")
        print(f"description: {entry['description']}")
        print(f"lanes: {', '.join(entry['lanes'])}")
        print("atlasctl mapping: atlasctl make run <target>")
        graph = parse_make_targets(ctx.repo_root / "makefiles")
        print("internal expansion tree:")
        for line in render_tree(graph, target):
            print(f"  {line}")
        return 0

    if ns.make_cmd == "graph":
        target = ns.target
        if target not in set(public_names()):
            print(f"not public: {target}")
            return 1
        graph = parse_make_targets(ctx.repo_root / "makefiles")
        if ns.json or ctx.output_format == "json":
            payload = {
                "schema_version": 1,
                "tool": "atlasctl",
                "status": "ok",
                "root": target,
                "graph": [{"target": name, "deps": graph.get(name, [])} for name in sorted(graph)],
                "tree": render_tree(graph, target),
            }
            print(json.dumps(payload, sort_keys=True))
            return 0
        for line in render_tree(graph, target):
            print(line)
        return 0

    if ns.make_cmd == "list-targets":
        graph = parse_make_targets(ctx.repo_root / "makefiles")
        rows = [{"name": name, "deps": graph.get(name, [])} for name in sorted(graph)]
        payload = {"schema_version": 1, "tool": "atlasctl", "status": "ok", "targets": rows}
        print(json.dumps(payload, sort_keys=True) if ns.json or ctx.output_format == "json" else json.dumps(payload, indent=2, sort_keys=True))
        return 0

    if ns.make_cmd == "inventory-logic":
        payload = _inventory_direct_make_logic(ctx.repo_root)
        rendered = json.dumps(payload, sort_keys=True) if ns.json or ctx.output_format == "json" else json.dumps(payload, indent=2, sort_keys=True)
        if ns.out_file:
            ensure_evidence_path(ctx, Path(ns.out_file)).write_text(rendered + "\n", encoding="utf-8")
        print(rendered)
        return 0

    if ns.make_cmd == "run":
        if not _is_public_or_allowlisted_target(ctx.repo_root, ns.target):
            print(f"make run target is not public/allowlisted: {ns.target}")
            return 2
        run_id = ctx.run_id
        isolate_dir = ctx.repo_root / "artifacts" / "isolate" / run_id / "atlasctl-make"
        isolate_dir.mkdir(parents=True, exist_ok=True)
        cmd = ["make", "-s", ns.target, *ns.args]
        env = dict(**os.environ)
        env["RUN_ID"] = run_id
        env["ISO_ROOT"] = str(isolate_dir)
        proc = subprocess.run(cmd, cwd=ctx.repo_root, text=True, capture_output=True, check=False, env=env)
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok" if proc.returncode == 0 else "fail",
            "command": "make run",
            "target": ns.target,
            "args": ns.args,
            "run_id": run_id,
            "isolate_dir": str(isolate_dir.relative_to(ctx.repo_root)),
            "exit_code": proc.returncode,
            "stdout": proc.stdout or "",
            "stderr": proc.stderr or "",
        }
        out = ensure_evidence_path(ctx, ctx.evidence_root / "make" / run_id / f"run-{ns.target.replace('/', '_')}.json")
        out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        if ns.json or ctx.output_format == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            print(f"make run target={ns.target} status={payload['status']} exit={proc.returncode}")
        return 0 if proc.returncode == 0 else 1

    if ns.make_cmd == "doctor":
        cmd = [sys.executable, "-m", "atlasctl.cli", "--quiet", "--format", "json", "suite", "run", "ci"]
        proc = subprocess.run(cmd, cwd=ctx.repo_root, text=True, capture_output=True, check=False)
        if proc.returncode == 0:
            if ns.json or ctx.output_format == "json":
                print(proc.stdout.strip())
            else:
                print("make doctor: pass (suite ci)")
            return 0
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "fail",
            "command": "make doctor",
            "suggestions": [
                "atlasctl suite explain ci",
                "atlasctl check domain make --fail-fast",
                "atlasctl report last-fail",
            ],
            "suite_output": proc.stdout.strip(),
            "suite_error": proc.stderr.strip(),
        }
        print(json.dumps(payload, sort_keys=True) if ns.json or ctx.output_format == "json" else "make doctor: fail (run `atlasctl suite explain ci`)")
        return 1

    if ns.make_cmd == "prereqs":
        run_id = ns.run_id or f"prereqs-{ctx.run_id}"
        cmd = [
            sys.executable,
            "-m",
            "atlasctl.cli",
            "--quiet",
            "run",
            "./packages/atlasctl/src/atlasctl/checks/layout/makefiles/tools/make_prereqs.py",
            "--run-id",
            run_id,
        ]
        proc = subprocess.run(cmd, cwd=ctx.repo_root, text=True, capture_output=True, check=False)
        if ns.json or ctx.output_format == "json":
            print(
                json.dumps(
                    {
                        "schema_version": 1,
                        "tool": "atlasctl",
                        "status": "ok" if proc.returncode == 0 else "fail",
                        "command": "make prereqs",
                        "run_id": run_id,
                        "stdout": proc.stdout,
                        "stderr": proc.stderr,
                    },
                    sort_keys=True,
                )
            )
        else:
            print(f"make prereqs: {'pass' if proc.returncode == 0 else 'fail'}")
        return proc.returncode

    if ns.make_cmd == "contracts-check":
        return run_contracts_check(
            ctx,
            fail_fast=ns.fail_fast,
            emit_artifacts=ns.emit_artifacts,
            as_json=ns.json,
        )

    return 2


def configure_make_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("make", help="make target intelligence and contracts checks")
    make_sub = p.add_subparsers(dest="make_cmd", required=True)

    help_p = make_sub.add_parser("help", help="render curated make help output")
    help_p.add_argument("--mode", choices=["help", "gates", "list", "advanced", "all"], default="help")

    make_sub.add_parser("list", help="list curated public make targets")
    lint = make_sub.add_parser("lint", help="run make policy lint checks")
    lint.add_argument("--json", action="store_true", help="emit JSON output")
    rewrite = make_sub.add_parser("rewrite", help="rewrite obvious direct script/python make invocations to atlasctl wrappers")
    rewrite.add_argument("--write", action="store_true", help="apply rewrites in-place")
    rewrite.add_argument("--limit", type=int, default=200, help="maximum files to rewrite in one run")
    rewrite.add_argument("--json", action="store_true", help="emit JSON output")

    explain = make_sub.add_parser("explain", help="explain a public target")
    explain.add_argument("target")

    graph = make_sub.add_parser("graph", help="print public target dependency graph")
    graph.add_argument("target")
    graph.add_argument("--json", action="store_true", help="emit JSON output")

    make_sub.add_parser("list-targets", help="list all parsed make targets deterministically").add_argument(
        "--json", action="store_true", help="emit JSON output"
    )
    list_public = make_sub.add_parser("list-public-targets", help="list SSOT public make targets")
    list_public.add_argument("--json", action="store_true", help="emit JSON output")
    inv_logic = make_sub.add_parser("inventory-logic", help="inventory targets with direct non-atlasctl logic")
    inv_logic.add_argument("--json", action="store_true", help="emit JSON output")
    inv_logic.add_argument("--out-file", default="", help="write output under evidence root")

    run_p = make_sub.add_parser("run", help="run a make target via atlasctl wrapper")
    run_p.add_argument("target")
    run_p.add_argument("args", nargs=argparse.REMAINDER)
    run_p.add_argument("--json", action="store_true", help="emit JSON output")

    doctor = make_sub.add_parser("doctor", help="diagnose make lane failures from suite ci")
    doctor.add_argument("--json", action="store_true", help="emit JSON output")

    prereqs = make_sub.add_parser("prereqs", help="run make prereqs diagnostics via atlasctl")
    prereqs.add_argument("--run-id", default="")
    prereqs.add_argument("--json", action="store_true", help="emit JSON output")

    surface = make_sub.add_parser("surface", help="emit public target machine surface")
    surface.add_argument("--pretty", action="store_true", help="render human-readable output")

    inv = make_sub.add_parser("inventory", help="generate make target inventory artifacts")
    inv.add_argument("--out-dir", default="docs/_generated")
    inv.add_argument("--check", action="store_true", help="fail if generated outputs differ")
    catalog = make_sub.add_parser("catalog", help="generate or validate make target catalog drift")
    catalog.add_argument("--out-dir", default="docs/_generated")
    catalog.add_argument("--check", action="store_true", help="fail if catalog outputs differ")

    cc = make_sub.add_parser("contracts-check", help="run make/gates contract checks")
    cc.add_argument("--json", action="store_true")
    cc.add_argument("--fail-fast", action="store_true")
    cc.add_argument("--emit-artifacts", action="store_true")
