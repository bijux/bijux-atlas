from __future__ import annotations

import argparse
import json
import re
import subprocess
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Callable

from ..core.context import RunContext
from ..core.fs import ensure_evidence_path


@dataclass(frozen=True)
class DocsCheck:
    check_id: str
    description: str
    cmd: list[str]
    actionable: str


def _check(check_id: str, description: str, cmd: list[str], actionable: str) -> DocsCheck:
    return DocsCheck(check_id, description, cmd, actionable)


DOCS_LINT_CHECKS: list[DocsCheck] = [
    _check(
        "docs-terminology-units",
        "Validate terminology and units SSOT usage",
        ["python3", "scripts/areas/docs/check_terminology_units_ssot.py"],
        "Align terminology and units references with docs SSOT conventions.",
    ),
    _check(
        "docs-status-lint",
        "Validate document status contract",
        ["python3", "scripts/areas/docs/lint_doc_status.py"],
        "Fix missing/invalid status frontmatter values.",
    ),
    _check(
        "docs-index-pages",
        "Validate index pages contract",
        ["./scripts/areas/docs/check_index_pages.sh"],
        "Ensure each docs directory has an index page where required.",
    ),
    _check(
        "docs-title-case",
        "Validate title case contract",
        ["./scripts/areas/docs/check_title_case.sh"],
        "Normalize page titles to the required style.",
    ),
    _check(
        "docs-no-orphans",
        "Validate no orphan docs",
        ["python3", "scripts/areas/docs/check_no_orphan_docs.py"],
        "Add nav links or remove orphaned docs pages.",
    ),
]


def _run_check(cmd: list[str], repo_root: Path) -> tuple[int, str]:
    proc = subprocess.run(cmd, cwd=repo_root, text=True, capture_output=True, check=False)
    output = (proc.stdout or "") + (proc.stderr or "")
    return proc.returncode, output.strip()


def _mkdocs_nav_file_refs(mkdocs_text: str) -> list[str]:
    refs: list[str] = []
    for line in mkdocs_text.splitlines():
        stripped = line.strip()
        if not stripped.startswith("- ") and ": " not in stripped:
            continue
        m = re.search(r":\s*([A-Za-z0-9_./-]+\.md)\s*$", stripped)
        if m:
            refs.append(m.group(1))
    return refs


def _mkdocs_missing_files(repo_root: Path) -> list[str]:
    mkdocs = repo_root / "mkdocs.yml"
    text = mkdocs.read_text(encoding="utf-8")
    refs = _mkdocs_nav_file_refs(text)
    missing = []
    for ref in refs:
        p = repo_root / "docs" / ref
        if not p.exists():
            missing.append(ref)
    return sorted(set(missing))


def _run_docs_checks(
    ctx: RunContext,
    checks: list[DocsCheck],
    report_format: str,
    fail_fast: bool,
    emit_artifacts: bool,
    runner: Callable[[list[str], Path], tuple[int, str]] = _run_check,
) -> int:
    started_at = datetime.now(timezone.utc).isoformat()
    rows: list[dict[str, object]] = []
    for check in checks:
        code, output = runner(check.cmd, ctx.repo_root)
        row: dict[str, object] = {
            "id": check.check_id,
            "description": check.description,
            "status": "pass" if code == 0 else "fail",
            "command": " ".join(check.cmd),
            "actionable": check.actionable,
        }
        if code != 0:
            row["error"] = output
        rows.append(row)
        if fail_fast and code != 0:
            break
    ended_at = datetime.now(timezone.utc).isoformat()
    failed_count = len([r for r in rows if r["status"] == "fail"])
    payload = {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "run_id": ctx.run_id,
        "status": "fail" if failed_count else "pass",
        "started_at": started_at,
        "ended_at": ended_at,
        "failed_count": failed_count,
        "total_count": len(rows),
        "checks": rows,
    }

    if emit_artifacts:
        out = ensure_evidence_path(ctx, ctx.evidence_root / "docs" / "check" / ctx.run_id / "report.json")
        out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    if report_format == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        print(
            "docs checks: "
            f"status={payload['status']} "
            f"checks={payload['total_count']} "
            f"failed={payload['failed_count']}"
        )
        for row in rows:
            if row["status"] == "fail":
                first = str(row.get("error", "")).splitlines()[:1]
                print(f"- FAIL {row['id']}: {first[0] if first else 'check failed'}")
                print(f"  fix: {row['actionable']}")

    return 0 if failed_count == 0 else 1


def _run_simple(ctx: RunContext, cmd: list[str], report: str) -> int:
    code, output = _run_check(cmd, ctx.repo_root)
    payload = {
        "schema_version": 1,
        "tool": "bijux-atlas",
        "run_id": ctx.run_id,
        "status": "pass" if code == 0 else "fail",
        "command": " ".join(cmd),
        "output": output,
    }
    if report == "json":
        print(json.dumps(payload, sort_keys=True))
    else:
        print(output)
    return code


def _generate_docs_inventory(repo_root: Path, out: Path) -> None:
    out.parent.mkdir(parents=True, exist_ok=True)
    lines = [
        "# Docs Inventory",
        "",
        "Generated by `bijux-atlas docs inventory`.",
        "",
        "## Command Surface",
        "",
    ]
    commands = [
        "docs check",
        "docs lint",
        "docs link-check",
        "docs public-surface-check",
        "docs no-internal-target-refs",
        "docs ops-entrypoints-check",
        "docs nav-check",
        "docs generated-check",
        "docs glossary-check",
        "docs contracts-index",
        "docs runbook-map",
        "docs evidence-policy-page",
        "docs inventory",
    ]
    for cmd in commands:
        lines.append(f"- `{cmd}`")
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")


def _generate_docs_evidence_policy(repo_root: Path, out_rel: str = "docs/_generated/evidence-policy.md") -> str:
    out = repo_root / out_rel
    out.parent.mkdir(parents=True, exist_ok=True)
    lines = [
        "# Evidence Policy",
        "",
        "Generated by `bijux-atlas docs evidence-policy-page`.",
        "",
        "- Runtime evidence location: `artifacts/evidence/`",
        "- Committed generated docs location: `docs/_generated/`",
        "- Ops committed generated location: `ops/_generated_committed/`",
        "- Runtime evidence must not be committed to git.",
    ]
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return out_rel


def run_docs_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    if ns.docs_cmd == "check":
        checks = DOCS_LINT_CHECKS + [
            _check(
                "docs-link-check",
                "Validate markdown links",
                ["./scripts/areas/public/check-markdown-links.sh"],
                "Fix broken internal links and anchors.",
            ),
            _check(
                "docs-public-surface",
                "Validate docs public surface",
                ["python3", "scripts/areas/docs/check_public_surface_docs.py"],
                "Regenerate/align docs public surface JSON and docs references.",
            ),
            _check(
                "docs-no-internal-target-refs",
                "Validate no internal make target refs",
                ["python3", "scripts/areas/docs/check_docs_make_only.py"],
                "Replace internal make targets with public targets in docs.",
            ),
            _check(
                "docs-ops-entrypoints",
                "Validate ops docs entrypoint policy",
                ["python3", "scripts/areas/layout/check_ops_external_entrypoints.py"],
                "Reference only make targets and ops/run entrypoints in docs.",
            ),
            _check(
                "docs-generated",
                "Validate generated docs are up-to-date",
                ["python3", "scripts/areas/docs/check_docs_freeze_drift.py"],
                "Regenerate docs outputs and commit deterministic updates.",
            ),
        ]
        return _run_docs_checks(ctx, checks, ns.report, ns.fail_fast, ns.emit_artifacts)

    if ns.docs_cmd == "lint":
        if ns.fix:
            code, output = _run_check(["python3", "scripts/areas/docs/rewrite_legacy_terms.py", "docs"], ctx.repo_root)
            if code != 0:
                if output:
                    print(output)
                return code
        return _run_docs_checks(ctx, DOCS_LINT_CHECKS, ns.report, ns.fail_fast, ns.emit_artifacts)

    if ns.docs_cmd == "link-check":
        return _run_simple(ctx, ["./scripts/areas/public/check-markdown-links.sh"], ns.report)

    if ns.docs_cmd == "public-surface-check":
        return _run_simple(ctx, ["python3", "scripts/areas/docs/check_public_surface_docs.py"], ns.report)

    if ns.docs_cmd == "no-internal-target-refs":
        return _run_simple(ctx, ["python3", "scripts/areas/docs/check_docs_make_only.py"], ns.report)

    if ns.docs_cmd == "ops-entrypoints-check":
        return _run_simple(ctx, ["python3", "scripts/areas/layout/check_ops_external_entrypoints.py"], ns.report)

    if ns.docs_cmd == "nav-check":
        missing = _mkdocs_missing_files(ctx.repo_root)
        payload = {
            "schema_version": 1,
            "tool": "bijux-atlas",
            "run_id": ctx.run_id,
            "status": "pass" if not missing else "fail",
            "missing_files": missing,
        }
        if ns.report == "json":
            print(json.dumps(payload, sort_keys=True))
        else:
            if missing:
                print("mkdocs nav check failed:")
                for item in missing:
                    print(f"- missing docs/{item}")
            else:
                print("mkdocs nav check passed")
        return 0 if not missing else 1

    if ns.docs_cmd == "generated-check":
        return _run_simple(ctx, ["python3", "scripts/areas/docs/check_docs_freeze_drift.py"], ns.report)

    if ns.docs_cmd == "glossary-check":
        return _run_simple(ctx, ["python3", "scripts/areas/docs/lint_glossary_links.py"], ns.report)

    if ns.docs_cmd == "contracts-index":
        if ns.fix:
            return _run_simple(ctx, ["python3", "scripts/areas/docs/generate_contracts_index_doc.py"], ns.report)
        return _run_simple(ctx, ["python3", "scripts/areas/docs/check_contracts_index_nav.py"], ns.report)

    if ns.docs_cmd == "runbook-map":
        if ns.fix:
            return _run_simple(ctx, ["python3", "scripts/areas/docs/generate_runbook_map_index.py"], ns.report)
        return _run_simple(ctx, ["python3", "scripts/areas/docs/check_runbook_map_registration.py"], ns.report)

    if ns.docs_cmd == "evidence-policy-page":
        out_rel = ns.out or "docs/_generated/evidence-policy.md"
        written = _generate_docs_evidence_policy(ctx.repo_root, out_rel)
        payload = {
            "schema_version": 1,
            "tool": "bijux-atlas",
            "run_id": ctx.run_id,
            "status": "pass",
            "output": written,
        }
        print(json.dumps(payload, sort_keys=True) if ns.report == "json" else payload["output"])
        return 0

    if ns.docs_cmd == "inventory":
        out = Path(ns.out or "docs/_generated/docs-inventory.md")
        _generate_docs_inventory(ctx.repo_root, ctx.repo_root / out)
        payload = {
            "schema_version": 1,
            "tool": "bijux-atlas",
            "run_id": ctx.run_id,
            "status": "pass",
            "output": out.as_posix(),
        }
        print(json.dumps(payload, sort_keys=True) if ns.report == "json" else payload["output"])
        return 0

    if ns.docs_cmd == "extract-code":
        return _run_simple(ctx, ["python3", "scripts/areas/docs/extract_code_blocks.py"], ns.report)

    if ns.docs_cmd == "render-diagrams":
        return _run_simple(ctx, ["bash", "scripts/areas/docs/render_diagrams.sh"], ns.report)

    if ns.docs_cmd == "lint-spelling":
        return _run_simple(ctx, ["python3", "scripts/areas/docs/spellcheck_docs.py", ns.path], ns.report)

    return 2


def configure_docs_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("docs", help="docs checks and generation commands")
    docs_sub = p.add_subparsers(dest="docs_cmd", required=True)

    check = docs_sub.add_parser("check", help="run canonical docs check suite")
    check.add_argument("--report", choices=["text", "json"], default="text")
    check.add_argument("--fail-fast", action="store_true")
    check.add_argument("--emit-artifacts", action="store_true")
    check.add_argument("--fix", action="store_true")

    lint = docs_sub.add_parser("lint", help="run docs lint checks")
    lint.add_argument("--report", choices=["text", "json"], default="text")
    lint.add_argument("--fail-fast", action="store_true")
    lint.add_argument("--emit-artifacts", action="store_true")
    lint.add_argument("--fix", action="store_true")

    for name, help_text in (
        ("link-check", "run internal markdown link checks"),
        ("public-surface-check", "validate docs public-surface contract"),
        ("no-internal-target-refs", "forbid internal make target references in docs"),
        ("ops-entrypoints-check", "ensure docs mention only make targets and ops/run entrypoints"),
        ("nav-check", "validate mkdocs nav references existing docs files"),
        ("generated-check", "validate generated docs are up-to-date"),
        ("glossary-check", "validate glossary and banned terms policy"),
        ("contracts-index", "validate or generate docs contracts index"),
        ("runbook-map", "validate or generate docs runbook map index"),
        ("evidence-policy-page", "generate docs evidence policy page"),
        ("inventory", "generate docs command inventory page"),
        ("extract-code", "extract code blocks from docs"),
        ("render-diagrams", "render docs diagrams"),
        ("lint-spelling", "run docs spelling checks"),
    ):
        cmd = docs_sub.add_parser(name, help=help_text)
        cmd.add_argument("--report", choices=["text", "json"], default="text")
        cmd.add_argument("--fix", action="store_true")
        if name == "inventory":
            cmd.add_argument("--out")
        if name == "evidence-policy-page":
            cmd.add_argument("--out")
        if name == "lint-spelling":
            cmd.add_argument("--path", default="docs")
