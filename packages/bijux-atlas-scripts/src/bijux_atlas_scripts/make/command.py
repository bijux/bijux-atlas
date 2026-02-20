from __future__ import annotations

import argparse
import json
import subprocess
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Callable

from ..core.context import RunContext
from ..core.fs import ensure_evidence_path
from .explain import LEGACY_TARGET_RE
from .help import render_advanced, render_all, render_gates, render_help, render_list
from .public_targets import entry_map, load_ownership, public_entries, public_names
from .target_graph import parse_make_targets, render_tree


@dataclass(frozen=True)
class MakeCheck:
    check_id: str
    description: str
    cmd: list[str]
    actionable: str


def _check(check_id: str, description: str, script: str, actionable: str) -> MakeCheck:
    return MakeCheck(check_id, description, ["python3", script], actionable)


def _check_cmd(check_id: str, description: str, cmd: list[str], actionable: str) -> MakeCheck:
    return MakeCheck(check_id, description, cmd, actionable)


CHECKS: list[MakeCheck] = [
    _check(
        "public-surface",
        "Validate public make surface contract",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_public_surface.py",
        "Run make inventory and keep public targets in SSOT.",
    ),
    _check(
        "no-dead-entrypoints",
        "Validate referenced scripts and targets exist",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_no_dead_entrypoints.py",
        "Update stale references or remove dead entrypoints.",
    ),
    _check(
        "no-orphan-docs-refs",
        "Validate documented commands exist",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_no_orphan_docs_refs.py",
        "Fix docs references or add missing command help coverage.",
    ),
    _check(
        "no-orphan-configs",
        "Validate config files are referenced or declared internal",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_no_orphan_configs.py",
        "Add docs/contract references or annotate internal config ownership.",
    ),
    _check(
        "no-orphan-owners",
        "Validate ownership coverage",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_no_orphan_owners.py",
        "Add missing owners for areas, paths, and command surfaces.",
    ),
    _check_cmd(
        "docs-public-surface",
        "Validate docs public surface consistency",
        ["python3", "-m", "bijux_atlas_scripts.cli", "docs", "public-surface-check", "--report", "json"],
        "Regenerate docs/_generated surfaces and align nav references.",
    ),
    _check_cmd(
        "suite-id-docs",
        "Validate suite id docs coverage",
        ["python3", "-m", "bijux_atlas_scripts.cli", "docs", "suite-id-docs-check", "--report", "json"],
        "Document missing suite ids or remove stale suite references.",
    ),
    _check(
        "ci-entrypoints",
        "Validate CI workflows only call allowed targets",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_ci_entrypoints.py",
        "Update workflow jobs to call approved public targets only.",
    ),
    _check(
        "help-excludes-internal",
        "Ensure help excludes internal targets",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_help_excludes_internal.py",
        "Move internal targets out of public help rendering.",
    ),
    _check(
        "public-target-ownership",
        "Ensure public target ownership coverage",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_make_target_ownership.py",
        "Add missing target owners in makefiles/ownership.json.",
    ),
    _check(
        "public-target-docs",
        "Ensure public target docs coverage",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_public_targets_documented.py",
        "Document missing targets under docs/_generated/make-targets.md.",
    ),
    _check(
        "public-target-budget",
        "Ensure public target budget",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_public_target_budget.py",
        "Trim public targets or increase budget with governance approval.",
    ),
    _check(
        "public-target-descriptions",
        "Validate public target descriptions",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_public_target_descriptions.py",
        "Add concise help descriptions for all public targets.",
    ),
    _check(
        "public-target-aliases",
        "Validate public target aliases",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_public_target_aliases.py",
        "Remove undocumented aliases or document supported aliases.",
    ),
    _check(
        "internal-target-doc-refs",
        "Validate internal targets are not in docs",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_internal_targets_not_in_docs.py",
        "Replace internal target references in docs with public targets.",
    ),
    _check(
        "makefile-boundaries",
        "Validate makefile target boundaries",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_makefile_target_boundaries.py",
        "Keep top-level/public and internal target boundaries strict.",
    ),
    _check(
        "makefiles-contract",
        "Validate makefiles contract",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_makefiles_contract.py",
        "Regenerate makefile contract artifacts and align file ownership.",
    ),
    _check(
        "makefiles-headers",
        "Validate makefile header contract",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_makefile_headers.py",
        "Add or correct required makefile scope headers.",
    ),
    _check(
        "makefiles-index-drift",
        "Validate makefiles index drift",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_makefiles_index_drift.py",
        "Regenerate makefile index docs and commit deterministic output.",
    ),
    _check(
        "make-targets-catalog-drift",
        "Validate make targets catalog drift",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_make_targets_catalog_drift.py",
        "Regenerate make targets catalog and commit updates.",
    ),
    _check(
        "cargo-dev-metadata",
        "Validate cargo-dev metadata consistency",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_cargo_dev_metadata.py",
        "Align cargo-dev metadata with declared make targets.",
    ),
    _check(
        "root-no-cargo-dev-deps",
        "Validate root has no cargo-dev deps",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_root_no_cargo_dev_deps.py",
        "Move cargo-dev-only dependencies out of the root lane.",
    ),
    _check(
        "cargo-invocation-scope",
        "Validate cargo invocation scoping",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_cargo_invocations_scoped.py",
        "Use lane wrappers to scope cargo invocations correctly.",
    ),
    _check(
        "root-diff-alarm",
        "Validate root diff alarm contract",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_root_diff_alarm.py",
        "Update root diff alarm allowlist or reduce root-level churn.",
    ),
    _check(
        "help-output-determinism",
        "Validate help output determinism",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_help_output_determinism.py",
        "Remove nondeterministic ordering from help rendering sources.",
    ),
    _check(
        "help-snapshot",
        "Validate help snapshot",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_help_snapshot.py",
        "Update help snapshot intentionally after reviewing target surface changes.",
    ),
    _check(
        "no-legacy-target-names",
        "Validate no legacy target names",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_no_legacy_target_names.py",
        "Rename or delete legacy targets; do not keep compatibility aliases.",
    ),
    _check(
        "root-mk-size-budget",
        "Validate root.mk size budget",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_root_mk_size_budget.py",
        "Move lane-specific logic to dedicated makefiles to stay within budget.",
    ),
    _check(
        "root-makefile-hygiene",
        "Validate root makefile hygiene",
        "packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/check_root_makefile_hygiene.py",
        "Fix ordering, phony coverage, and structural hygiene issues in root.mk.",
    ),
]


def _surface_payload() -> dict[str, object]:
    owners = load_ownership()
    entries = public_entries()
    targets = []
    for item in entries:
        name = str(item.get("name", ""))
        meta = owners.get(name, {}) if isinstance(owners, dict) else {}
        targets.append(
            {
                "name": name,
                "description": str(item.get("description", "")),
                "area": str(item.get("area", "")),
                "lanes": item.get("lanes", []),
                "owner": str(meta.get("owner", "unknown")),
            }
        )
    return {"schema_version": 1, "targets": targets}


def _contracts_report(
    checks: list[dict[str, object]],
    started_at: str,
    ended_at: str,
    run_id: str,
) -> dict[str, object]:
    failed = [c for c in checks if c.get("status") == "fail"]
    return {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "run_id": run_id,
        "status": "fail" if failed else "pass",
        "started_at": started_at,
        "ended_at": ended_at,
        "checks": checks,
        "failed_count": len(failed),
        "total_count": len(checks),
    }


def _run_check(cmd: list[str], repo_root: Path) -> tuple[int, str]:
    proc = subprocess.run(cmd, cwd=repo_root, text=True, capture_output=True, check=False)
    output = (proc.stdout or "") + (proc.stderr or "")
    return proc.returncode, output.strip()


def run_contracts_check(
    ctx: RunContext,
    fail_fast: bool,
    emit_artifacts: bool,
    as_json: bool,
    runner: Callable[[list[str], Path], tuple[int, str]] = _run_check,
) -> int:
    repo_root = ctx.repo_root
    started_at = datetime.now(timezone.utc).isoformat()
    results: list[dict[str, object]] = []

    for check in CHECKS:
        code, output = runner(check.cmd, repo_root)
        status = "pass" if code == 0 else "fail"
        item = {
            "id": check.check_id,
            "description": check.description,
            "status": status,
            "command": " ".join(check.cmd),
            "actionable": check.actionable,
        }
        if status == "fail":
            item["error"] = output
        results.append(item)
        if fail_fast and status == "fail":
            break

    ended_at = datetime.now(timezone.utc).isoformat()
    payload = _contracts_report(results, started_at, ended_at, ctx.run_id)

    if emit_artifacts:
        out = ensure_evidence_path(
            ctx,
            ctx.evidence_root / "make" / ctx.run_id / "contracts-check.json",
        )
        out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    schema_path = repo_root / "configs/contracts/make-contracts-check-output.schema.json"
    schema = json.loads(schema_path.read_text(encoding="utf-8"))
    import jsonschema

    jsonschema.validate(payload, schema)

    if as_json:
        print(json.dumps(payload, sort_keys=True))
    else:
        print(
            "make contracts-check: "
            f"status={payload['status']} "
            f"checks={payload['total_count']} "
            f"failed={payload['failed_count']}"
        )
        for row in payload["checks"]:
            if row["status"] == "fail":
                first_line = (
                    row.get("error", "").splitlines()[:1][0]
                    if row.get("error")
                    else "check failed"
                )
                print(f"- FAIL {row['id']}: {first_line}")
                print(f"  fix: {row['actionable']}")
    return 0 if payload["status"] == "pass" else 1


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
        for line in render_tree(graph, target):
            print(line)
        return 0

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

    explain = make_sub.add_parser("explain", help="explain a public target")
    explain.add_argument("target")

    graph = make_sub.add_parser("graph", help="print public target dependency graph")
    graph.add_argument("target")

    surface = make_sub.add_parser("surface", help="emit public target machine surface")
    surface.add_argument("--pretty", action="store_true", help="render human-readable output")

    inv = make_sub.add_parser("inventory", help="generate make target inventory artifacts")
    inv.add_argument("--out-dir", default="docs/_generated")
    inv.add_argument("--check", action="store_true", help="fail if generated outputs differ")

    cc = make_sub.add_parser("contracts-check", help="run make/gates contract checks")
    cc.add_argument("--json", action="store_true")
    cc.add_argument("--fail-fast", action="store_true")
    cc.add_argument("--emit-artifacts", action="store_true")
