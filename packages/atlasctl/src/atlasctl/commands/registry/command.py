from __future__ import annotations

import argparse
import json

from ...core.context import RunContext
from ...core.runtime.paths import write_text_file
from ...checks.registry.ssot import CHECKS_CATALOG_JSON


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


def run_registry_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    sub = str(getattr(ns, "registry_cmd", "") or "")
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
