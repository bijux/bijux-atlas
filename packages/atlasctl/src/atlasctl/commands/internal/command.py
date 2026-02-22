from __future__ import annotations

import argparse
import json
import os
import sys
import re
from pathlib import Path

from ...core.context import RunContext
from ...core.exec import run
from ...core.runtime.paths import write_text_file
from .legacy_inventory import run_legacy_inventory
from .legacy_targets import run_legacy_targets
from .refactor_check_ids import run_refactor_check_ids

_INTERNAL_FORWARD: dict[str, str] = {
    "self-check": "self-check",
    "doctor": "doctor",
}
_INTERNAL_ITEMS: tuple[str, ...] = (
    "doctor",
    "legacy",
    "legacy-targets",
    "refactor-check-ids",
    "self-check",
    "fix-next",
    "migration-progress",
)


_LEGACY_DOC_PATTERNS = (
    re.compile(r"\bpython3?\s+-m\s+atlasctl\.cli\b"),
    re.compile(r"\./bin/bijux-atlas\b"),
    re.compile(r"\bscripts migration\b", re.IGNORECASE),
)
_LEGACY_ALIAS_PATTERN = re.compile(r"\$\((ATLAS_SCRIPTS|SCRIPTS|PY_RUN)\)|\b(ATLAS_SCRIPTS|SCRIPTS|PY_RUN)\b")


def _forward(ctx: RunContext, *args: str) -> int:
    env = os.environ.copy()
    src_path = str(ctx.repo_root / "packages/atlasctl/src")
    existing = env.get("PYTHONPATH", "")
    env["PYTHONPATH"] = f"{src_path}:{existing}" if existing else src_path
    proc = run(
        [sys.executable, "-m", "atlasctl.cli", *args],
        cwd=ctx.repo_root,
        env=env,
        text=True,
    )
    return proc.returncode


def _run_repo_checks_json(ctx: RunContext) -> dict[str, object]:
    proc = run(
        ["./bin/atlasctl", "check", "run", "--group", "repo", "--json"],
        cwd=ctx.repo_root,
        text=True,
        capture_output=True,
    )
    if proc.returncode not in (0, 1, 2):
        raise RuntimeError(f"check runner failed to execute: code={proc.returncode}")
    stdout = (proc.stdout or "").strip()
    if not stdout:
        return {"rows": [], "status": "error", "error": "empty check output"}
    # check run --json outputs pure json payload.
    return json.loads(stdout)


def _extract_paths(text: str) -> list[str]:
    candidates = re.findall(r"(?:^|[\s:])((?:packages|makefiles|docs|configs|.github)/[A-Za-z0-9_./-]+)", text)
    return sorted({item.strip() for item in candidates if item.strip()})[:3]


def _run_fix_next(ctx: RunContext, ns: argparse.Namespace) -> int:
    payload = _run_repo_checks_json(ctx) if bool(getattr(ns, "from_last_fail", False)) else _run_repo_checks_json(ctx)
    rows = payload.get("rows", []) if isinstance(payload, dict) else []
    failed = [row for row in rows if isinstance(row, dict) and str(row.get("status", "")).upper() == "FAIL"]
    top_n = int(getattr(ns, "top", 8) or 8)
    selected = failed[:top_n]
    output_rows: list[dict[str, object]] = []
    for row in selected:
        reason = str(row.get("reason", "") or "")
        hint = ""
        hints = row.get("hints", [])
        if isinstance(hints, list) and hints:
            hint = str(hints[0])
        elif row.get("hint"):
            hint = str(row.get("hint"))
        output_rows.append(
            {
                "id": row.get("id", ""),
                "owners": row.get("owners", []),
                "reason": reason,
                "hint": hint,
                "paths": _extract_paths(reason),
            }
        )
    result = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "ok",
        "kind": "fix-next",
        "source": "repo-check-run",
        "total_failed": len(failed),
        "top": output_rows,
    }
    if bool(getattr(ns, "json", False)):
        print(json.dumps(result, sort_keys=True))
    else:
        print(f"failed checks: {len(failed)}")
        for idx, row in enumerate(output_rows, start=1):
            paths = ", ".join(row["paths"]) if row["paths"] else "-"
            print(f"[{idx}] {row['id']}")
            print(f"    paths: {paths}")
            if row["reason"]:
                print(f"    reason: {row['reason']}")
            if row["hint"]:
                print(f"    hint: {row['hint']}")
    return 1 if failed else 0


def _collect_migration_progress(ctx: RunContext) -> dict[str, object]:
    docs_roots = [ctx.repo_root / "docs", ctx.repo_root / "packages/atlasctl/docs"]
    docs_legacy_hits = 0
    for root in docs_roots:
        if not root.exists():
            continue
        for md in sorted(root.rglob("*.md")):
            text = md.read_text(encoding="utf-8", errors="ignore")
            for pattern in _LEGACY_DOC_PATTERNS:
                docs_legacy_hits += len(pattern.findall(text))
    make_alias_hits = 0
    for mk in sorted((ctx.repo_root / "makefiles").glob("*.mk")):
        text = mk.read_text(encoding="utf-8", errors="ignore")
        make_alias_hits += len(_LEGACY_ALIAS_PATTERN.findall(text))
    ops_script_paths: list[str] = []
    for root in (
        ctx.repo_root / "ops" / "run",
        ctx.repo_root / "ops" / "k8s",
        ctx.repo_root / "ops" / "obs",
        ctx.repo_root / "ops" / "load",
        ctx.repo_root / "ops" / "datasets",
        ctx.repo_root / "ops" / "stack",
        ctx.repo_root / "ops" / "e2e",
    ):
        if not root.exists():
            continue
        for path in sorted(root.rglob("*")):
            if path.is_file() and path.suffix in {".sh", ".py"}:
                ops_script_paths.append(path.relative_to(ctx.repo_root).as_posix())
    payload: dict[str, object] = {
        "schema_version": 1,
        "tool": "atlasctl",
        "kind": "migration-progress",
        "status": "ok",
        "metrics": {
            "docs_legacy_cli_invocations": docs_legacy_hits,
            "make_legacy_alias_tokens": make_alias_hits,
            "ops_script_entrypoints": len(ops_script_paths),
        },
        "ops_script_inventory": ops_script_paths,
    }
    return payload


def _run_migration_progress(ctx: RunContext, ns: argparse.Namespace) -> int:
    payload = _collect_migration_progress(ctx)
    out_path = ctx.repo_root / "artifacts/reports/atlasctl/migration-progress.json"
    write_text_file(out_path, json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    script_inv_path = ctx.repo_root / "artifacts/reports/atlasctl/ops-script-deprecation-inventory.json"
    write_text_file(
        script_inv_path,
        json.dumps(
            {
                "schema_version": 1,
                "tool": "atlasctl",
                "kind": "ops-script-deprecation-inventory",
                "status": "ok",
                "count": int((payload.get("metrics", {}) or {}).get("ops_script_entrypoints", 0)),
                "scripts": payload.get("ops_script_inventory", []),
            },
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )
    baseline_path = ctx.repo_root / "configs/policy/migration-progress-baseline.json"
    if baseline_path.exists():
        baseline = json.loads(baseline_path.read_text(encoding="utf-8"))
        limits = baseline.get("max", {})
        for key, value in (payload.get("metrics", {}) or {}).items():
            max_value = int(limits.get(key, value))
            if int(value) > max_value:
                payload["status"] = "error"
                payload["error"] = f"migration regression: {key}={value} > max={max_value}"
                break
    if bool(getattr(ns, "json", False)):
        print(json.dumps(payload, sort_keys=True))
    else:
        print(f"migration progress: {out_path.relative_to(ctx.repo_root).as_posix()}")
        print(f"ops script inventory: {script_inv_path.relative_to(ctx.repo_root).as_posix()}")
        for key, value in (payload.get("metrics", {}) or {}).items():
            print(f"- {key}: {value}")
        if payload.get("status") != "ok":
            print(str(payload.get("error", "migration progress regression detected")))
    return 0 if payload.get("status") == "ok" else 1


def run_internal_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    sub = getattr(ns, "internal_cmd", "")
    if not sub and bool(getattr(ns, "list", False)):
        if bool(getattr(ns, "json", False)):
            print(json.dumps({"schema_version": 1, "tool": "atlasctl", "status": "ok", "group": "internal", "items": list(_INTERNAL_ITEMS)}, sort_keys=True))
        else:
            for item in _INTERNAL_ITEMS:
                print(item)
        return 0
    if sub == "legacy":
        action = getattr(ns, "legacy_cmd", "") or "inventory"
        if action == "inventory":
            return run_legacy_inventory(ctx, getattr(ns, "report", "text"))
        return 2
    if sub == "legacy-targets":
        return run_legacy_targets(ctx, getattr(ns, "report", "text"))
    if sub == "refactor-check-ids":
        code, touched = run_refactor_check_ids(ctx.repo_root, apply=bool(getattr(ns, "apply", False)))
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "ok",
            "kind": "refactor-check-ids",
            "apply": bool(getattr(ns, "apply", False)),
            "changed_files": touched,
            "changed_count": len(touched),
            "alias_expires_on": "2026-12-31",
        }
        if bool(getattr(ns, "json", False)):
            print(json.dumps(payload, sort_keys=True))
        else:
            print(f"changed={len(touched)}")
            for rel in touched:
                print(rel)
        return code
    if sub == "fix-next":
        return _run_fix_next(ctx, ns)
    if sub == "migration-progress":
        return _run_migration_progress(ctx, ns)
    forwarded = _INTERNAL_FORWARD.get(sub)
    if not forwarded:
        return 2
    return _forward(ctx, forwarded, *getattr(ns, "args", []))


def configure_internal_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    parser = sub.add_parser("internal", help="internal control-plane group (legacy inventory and diagnostics)")
    parser.add_argument("--list", action="store_true", help="list available internal commands")
    parser.add_argument("--json", action="store_true", help="emit machine-readable JSON output")
    internal_sub = parser.add_subparsers(dest="internal_cmd", required=False)
    legacy = internal_sub.add_parser("legacy", help="internal legacy reports")
    legacy_sub = legacy.add_subparsers(dest="legacy_cmd", required=False)
    inventory = legacy_sub.add_parser("inventory", help="emit legacy inventory report")
    inventory.add_argument("--report", choices=["text", "json"], default="text")
    legacy.add_argument("--report", choices=["text", "json"], default="text")
    for name, help_text in (("self-check", "forward to `atlasctl self-check`"), ("doctor", "forward to `atlasctl doctor`")):
        sp = internal_sub.add_parser(name, help=help_text)
        sp.add_argument("args", nargs=argparse.REMAINDER)
    legacy_targets = internal_sub.add_parser("legacy-targets", help="list deprecated legacy targets with expiry")
    legacy_targets.add_argument("--report", choices=["text", "json"], default="text")
    refactor = internal_sub.add_parser("refactor-check-ids", help="rewrite legacy check ids to checks_* canonical ids")
    refactor.add_argument("--apply", action="store_true", help="apply edits in-place (default: dry-run)")
    fix_next = internal_sub.add_parser("fix-next", help="print top failing checks with file hints")
    fix_next.add_argument("--from-last-fail", action="store_true", help="resolve from latest repo check failures")
    fix_next.add_argument("--top", type=int, default=8, help="max failing checks to print")
    fix_next.add_argument("--json", action="store_true", help="emit JSON payload")
    progress = internal_sub.add_parser("migration-progress", help="emit migration progress artifact and enforce no-regression baseline")
    progress.add_argument("--json", action="store_true", help="emit JSON payload")
