from __future__ import annotations

import argparse
import json
from pathlib import Path

from ...core.context import RunContext
from ...core.runtime.paths import write_text_file
from ...checks.registry import CHECKS_CATALOG_JSON, generate_registry_json
from ...checks.domains.internal import (
    check_registry_change_requires_docs_update,
    check_registry_change_requires_golden_update,
    check_registry_change_requires_owner_update,
)
from ...registry.spine import REGISTRY_SPINE_GENERATED_JSON, generate_registry_spine, load_registry
from ..internal.refactor_check_ids import run_refactor_check_ids


def _render_checks_index(payload: dict[str, object]) -> str:
    rows = payload.get("checks", [])
    by_domain: dict[str, list[dict[str, object]]] = {}
    for row in rows if isinstance(rows, list) else []:
        if not isinstance(row, dict):
            continue
        domain = str(row.get("domain", "unknown"))
        by_domain.setdefault(domain, []).append(row)
    lines = [
        "# Checks Index",
        "",
        "Generated from `packages/atlasctl/src/atlasctl/registry/checks_catalog.json`.",
        "",
    ]
    for domain, entries in sorted(by_domain.items()):
        lines.append(f"## {domain}")
        lines.append("")
        for entry in sorted(entries, key=lambda item: str(item.get("id", ""))):
            lines.append(f"- `{entry.get('id', '')}`: {entry.get('description', '')}")
        lines.append("")
    return "\n".join(lines).rstrip() + "\n"


def _render_commands_doc() -> str:
    reg = load_registry()
    lines = [
        "# Atlasctl CLI Commands (generated)",
        "",
        "Generated from the unified registry spine.",
        "",
    ]
    current_group = None
    for item in reg.commands:
        if item.internal:
            continue
        if item.group != current_group:
            current_group = item.group
            lines.extend([f"## {current_group}", ""])
        tags = ", ".join(item.tags[:5])
        lines.append(f"- `{item.name}` ({item.owner}): {item.help_text}" + (f" [{tags}]" if tags else ""))
    return "\n".join(lines).rstrip() + "\n"


def _render_ownership_doc(ctx: RunContext) -> str:
    reg = load_registry(ctx.repo_root)
    ops_owners_path = ctx.repo_root / "ops/inventory/owners.json"
    ops_payload = {}
    if ops_owners_path.exists():
        try:
            ops_payload = json.loads(ops_owners_path.read_text(encoding="utf-8"))
        except Exception:
            ops_payload = {}
    lines = [
        "# Ownership (generated)",
        "",
        "Generated from registry spine + ops inventory owners.",
        "",
        "## Command Owners",
        "",
    ]
    by_owner: dict[str, list[str]] = {}
    for cmd in reg.commands:
        by_owner.setdefault(cmd.owner, []).append(cmd.name)
    for owner in sorted(by_owner):
        lines.append(f"### {owner}")
        lines.append("")
        for name in sorted(by_owner[owner]):
            lines.append(f"- `atlasctl {name}`")
        lines.append("")
    lines.append("## Ops Inventory Owners")
    lines.append("")
    for key in sorted((ops_payload.get("paths") or {}).keys()):
        lines.append(f"- `{key}`: `{ops_payload['paths'][key]}`")
    return "\n".join(lines).rstrip() + "\n"


def _render_registry_spine_json(ctx: RunContext) -> str:
    return json.dumps(generate_registry_spine(ctx.repo_root), indent=2, sort_keys=True) + "\n"


def _write_if_changed(path: Path, rendered: str, *, check: bool) -> bool:
    current = path.read_text(encoding="utf-8") if path.exists() else ""
    changed = current != rendered
    if not check and changed:
        path.parent.mkdir(parents=True, exist_ok=True)
        write_text_file(path, rendered, encoding="utf-8")
    return changed


def _registry_invariant_errors(ctx: RunContext) -> list[str]:
    reg = load_registry(ctx.repo_root)
    errors: list[str] = []
    check_ids = [c.check_id for c in reg.checks]
    if len(check_ids) != len(set(check_ids)):
        errors.append("registry invariants: duplicate check ids")
    command_names = [c.name for c in reg.commands]
    if len(command_names) != len(set(command_names)):
        errors.append("registry invariants: duplicate command names")
    if any(not c.owner for c in reg.commands):
        errors.append("ownership invariants: every command must have an owner")
    if any(not c.owner for c in reg.checks):
        errors.append("ownership invariants: every check must have an owner")
    docs_cli = (ctx.repo_root / "docs/_generated/cli.md").read_text(encoding="utf-8", errors="ignore") if (ctx.repo_root / "docs/_generated/cli.md").exists() else ""
    for cmd in reg.commands:
        if cmd.internal:
            continue
        if f"`{cmd.name}`" not in docs_cli:
            errors.append(f"public surface invariants: public command missing from docs/_generated/cli.md: {cmd.name}")
            break
    for cmd in reg.commands:
        if cmd.stable and cmd.internal:
            errors.append(f"stability invariants: internal command cannot be stable: {cmd.name}")
            break
    return errors


def run_registry_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    sub = str(getattr(ns, "registry_cmd", "") or "")
    if sub == "gen":
        check = bool(getattr(ns, "check", False))
        _out, checks_changed = generate_registry_json(ctx.repo_root, check_only=check)
        spine_path = ctx.repo_root / REGISTRY_SPINE_GENERATED_JSON
        spine_changed = _write_if_changed(spine_path, _render_registry_spine_json(ctx), check=check)
        cli_doc_changed = _write_if_changed(ctx.repo_root / "docs/_generated/cli.md", _render_commands_doc(), check=check)
        checks_doc_changed = _write_if_changed(
            ctx.repo_root / "packages/atlasctl/docs/checks/index.md",
            _render_checks_index(json.loads((ctx.repo_root / CHECKS_CATALOG_JSON).read_text(encoding="utf-8"))),
            check=check,
        )
        owners_doc_changed = _write_if_changed(ctx.repo_root / "packages/atlasctl/docs/ownership.md", _render_ownership_doc(ctx), check=check)
        changed = any((checks_changed, spine_changed, cli_doc_changed, checks_doc_changed, owners_doc_changed))
        if check and changed:
            print("registry drift detected: run `./bin/atlasctl registry gen`")
            return 2
        print("registry generated" if not check else "registry outputs up-to-date")
        return 0
    if sub == "diff":
        tmp_check = argparse.Namespace(registry_cmd="gen", check=True)
        return run_registry_command(ctx, tmp_check)
    if sub == "validate":
        errors = _registry_invariant_errors(ctx)
        if errors:
            print(json.dumps({"schema_version": 1, "tool": "atlasctl", "kind": "registry-validate", "status": "error", "errors": errors}, sort_keys=True) if (ctx.output_format == "json" or bool(getattr(ns, "json", False))) else "\n".join(errors))
            return 2
        print(json.dumps({"schema_version": 1, "tool": "atlasctl", "kind": "registry-validate", "status": "ok"}, sort_keys=True) if (ctx.output_format == "json" or bool(getattr(ns, "json", False))) else "registry invariants ok")
        return 0
    if sub == "gate":
        rc = run_registry_command(ctx, argparse.Namespace(registry_cmd="diff"))
        errors = _registry_invariant_errors(ctx)
        for fn in (check_registry_change_requires_owner_update, check_registry_change_requires_docs_update, check_registry_change_requires_golden_update):
            code, errs = fn(ctx.repo_root)
            if code:
                errors.extend(errs)
        if rc not in {0}:
            errors.append("registry change gate: registry diff failed")
        if errors:
            print(json.dumps({"schema_version": 1, "tool": "atlasctl", "kind": "registry-gate", "status": "error", "errors": errors}, sort_keys=True) if (ctx.output_format == "json" or bool(getattr(ns, "json", False))) else "\n".join(errors))
            return 2
        print(json.dumps({"schema_version": 1, "tool": "atlasctl", "kind": "registry-gate", "status": "ok"}, sort_keys=True) if (ctx.output_format == "json" or bool(getattr(ns, "json", False))) else "registry gate ok")
        return 0
    if sub == "rename-check-id":
        code, touched = run_refactor_check_ids(ctx.repo_root, apply=bool(getattr(ns, "apply", False)))
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "kind": "registry-rename-check-id",
            "status": "ok" if code == 0 else "error",
            "apply": bool(getattr(ns, "apply", False)),
            "changed_count": len(touched),
            "changed_files": touched,
        }
        print(json.dumps(payload, sort_keys=True) if (ctx.output_format == "json" or bool(getattr(ns, "json", False))) else f"changed={len(touched)}")
        return code
    if sub == "select":
        reg = load_registry(ctx.repo_root)
        subject = str(getattr(ns, "subject", "checks"))
        tags = tuple(sorted(str(x).strip() for x in getattr(ns, "tags", []) if str(x).strip()))
        if subject == "checks":
            rows = reg.select_checks(
                domain=(str(getattr(ns, "domain", "")).strip() or None),
                tags=tags,
                severity=(str(getattr(ns, "severity", "")).strip() or None),
                suite=(str(getattr(ns, "suite", "")).strip() or None),
            )
            payload = {"checks": [r.check_id for r in rows]}
            print(json.dumps(payload, sort_keys=True) if (ctx.output_format == "json" or bool(getattr(ns, "json", False))) else "\n".join(r.check_id for r in rows))
            return 0
        rows = reg.select_commands(group=(str(getattr(ns, "group", "")).strip() or None), tags=tags)
        payload = {"commands": [r.name for r in rows]}
        print(json.dumps(payload, sort_keys=True) if (ctx.output_format == "json" or bool(getattr(ns, "json", False))) else "\n".join(r.name for r in rows))
        return 0
    if sub in {"", "checks"}:
        payload = json.loads((ctx.repo_root / CHECKS_CATALOG_JSON).read_text(encoding="utf-8"))
        if ctx.output_format == "json" or bool(getattr(ns, "json", False)):
            print(json.dumps(payload, sort_keys=True))
            return 0
        for row in payload.get("checks", []):
            print(str(row.get("id", "")))
        return 0
    if sub == "checks-index":
        payload = json.loads((ctx.repo_root / CHECKS_CATALOG_JSON).read_text(encoding="utf-8"))
        rendered = _render_checks_index(payload)
        out_path = ctx.repo_root / "packages/atlasctl/docs/checks/index.md"
        if bool(getattr(ns, "check", False)):
            current = out_path.read_text(encoding="utf-8") if out_path.exists() else ""
            if current != rendered:
                print("checks index drift: run `./bin/atlasctl registry checks-index`")
                return 2
            print("checks index up-to-date")
            return 0
        out_path.parent.mkdir(parents=True, exist_ok=True)
        write_text_file(out_path, rendered, encoding="utf-8")
        print(str(out_path.relative_to(ctx.repo_root)))
        return 0
    return 2


def configure_registry_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    parser = sub.add_parser("registry", help="registry domain commands")
    parser.add_argument("--json", action="store_true", help="emit JSON output")
    registry_sub = parser.add_subparsers(dest="registry_cmd", required=False)
    checks = registry_sub.add_parser("checks", help="print checks catalog from SSOT registry")
    checks.add_argument("--json", action="store_true", help="emit JSON output")
    checks_index = registry_sub.add_parser("checks-index", help="generate checks docs index from checks catalog")
    checks_index.add_argument("--check", action="store_true", help="fail if docs index is out of date")
    gen = registry_sub.add_parser("gen", help="generate deterministic registry catalogs/docs from SSOT")
    gen.add_argument("--check", action="store_true", help="fail on registry drift without writing files")
    registry_sub.add_parser("diff", help="fail if registry generated outputs drift from SSOT")
    validate = registry_sub.add_parser("validate", help="validate registry invariants (ids, owners, public surface, stability)")
    validate.add_argument("--json", action="store_true", help="emit JSON output")
    gate = registry_sub.add_parser("gate", help="single registry change gate (diff + invariants + change gates)")
    gate.add_argument("--json", action="store_true", help="emit JSON output")
    rename = registry_sub.add_parser("rename-check-id", help="safe check-id rename migration helper (updates docs/goldens refs)")
    rename.add_argument("--apply", action="store_true", help="apply edits (default dry-run)")
    rename.add_argument("--json", action="store_true", help="emit JSON output")
    select = registry_sub.add_parser("select", help="select checks/commands via registry selectors")
    select.add_argument("subject", choices=["checks", "commands"])
    select.add_argument("--json", action="store_true", help="emit JSON output")
    select.add_argument("--domain", help="checks selector: domain")
    select.add_argument("--severity", help="checks selector: severity")
    select.add_argument("--suite", help="checks selector: suite membership")
    select.add_argument("--group", help="commands selector: group")
    select.add_argument("--tag", dest="tags", action="append", default=[], help="selector tag (repeatable)")
