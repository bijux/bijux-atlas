from __future__ import annotations

import argparse
import re
from typing import Any

from ...checks.registry import check_tags, list_checks
from ...cli.surface_registry import command_registry
from ...core.effects import command_group
from ...core.runtime.serialize import dumps_json
from ...core.context import RunContext
from ...registry import CheckRecord, CommandRecord, SuiteRecord
from ..policies.runtime.command import _POLICIES_ITEMS
from ...suite.command import load_suites
from ...suite.manifests import load_first_class_suites


def _parse_tags(raw: str) -> tuple[str, ...]:
    return tuple(sorted({item.strip() for item in raw.split(",") if item.strip()}))


def _pattern_match(value: str, pattern: str | None) -> bool:
    if not pattern:
        return True
    if pattern in value:
        return True
    try:
        return re.search(pattern, value) is not None
    except re.error:
        return pattern.lower() in value.lower()


def _filter_records(records: list[Any], tags: tuple[str, ...], pattern: str | None, include_internal: bool) -> list[Any]:
    filtered: list[Any] = []
    for rec in records:
        record_tags = set(getattr(rec, "tags", ()))
        if tags and not record_tags.intersection(tags):
            continue
        if not include_internal and bool(getattr(rec, "internal", False)):
            continue
        name = getattr(rec, "id", None) or getattr(rec, "name", "")
        if not _pattern_match(str(name), pattern):
            continue
        filtered.append(rec)
    return filtered


def _check_records() -> list[CheckRecord]:
    out: list[CheckRecord] = []
    for check in list_checks():
        tags = check_tags(check)
        out.append(
            CheckRecord(
                id=str(check.canonical_id or check.check_id),
                title=check.title,
                domain=check.domain,
                tags=tags,
                effects=check.effects,
                owners=check.owners,
                internal=("internal" in tags),
            )
        )
    return sorted(out, key=lambda item: item.id)


def _command_records() -> list[CommandRecord]:
    out: list[CommandRecord] = []
    for spec in command_registry():
        tags = tuple(sorted({command_group(spec.name), "stable" if spec.stable else "unstable", "internal" if spec.internal else "public"}))
        out.append(
            CommandRecord(
                name=spec.name,
                help=spec.help_text,
                tags=tags,
                owner=spec.owner,
                aliases=spec.aliases,
                stable=spec.stable,
                internal=spec.internal,
            )
        )
    return sorted(out, key=lambda item: item.name)


def _suite_records(ctx: RunContext) -> list[SuiteRecord]:
    default_name, suites = load_suites(ctx.repo_root)
    manifests = load_first_class_suites()
    out: list[SuiteRecord] = []
    for spec in suites.values():
        tags = {"suite", spec.name, "default" if spec.name == default_name else "non-default"}
        internal = spec.name.startswith("_")
        if internal:
            tags.add("internal")
        else:
            tags.add("public")
        out.append(
            SuiteRecord(
                name=spec.name,
                includes=spec.includes,
                item_count=len(spec.items),
                complete=spec.complete,
                tags=tuple(sorted(tags)),
                internal=internal,
            )
        )
    for manifest in manifests.values():
        if manifest.name in suites:
            continue
        tags = {"suite", manifest.name, *manifest.markers, "first-class", "public" if not manifest.internal else "internal"}
        out.append(
            SuiteRecord(
                name=manifest.name,
                includes=(),
                item_count=len(manifest.check_ids),
                complete=True,
                tags=tuple(sorted(tags)),
                internal=manifest.internal,
            )
        )
    return sorted(out, key=lambda item: item.name)


def _render(payload: dict[str, object], as_json: bool) -> None:
    print(dumps_json(payload, pretty=not as_json))


def run_list_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    requested_format = str(getattr(ns, "format", "") or "").strip().lower()
    as_json = requested_format == "json" or ctx.output_format == "json" or bool(getattr(ns, "json", False))
    tags = _parse_tags(getattr(ns, "tags", ""))
    include_internal = bool(getattr(ns, "include_internal", False))
    pattern = getattr(ns, "pattern", None)
    if ns.list_kind == "checks":
        records = _filter_records(_check_records(), tags, pattern, include_internal)
        if requested_format == "tree":
            tree: dict[str, dict[str, list[str]]] = {}
            for rec in records:
                parts = rec.id.split("_")
                domain = parts[1] if len(parts) > 1 else rec.domain
                area = parts[2] if len(parts) > 2 else "general"
                tree.setdefault(domain, {}).setdefault(area, []).append(rec.id)
            if as_json:
                payload = {
                    "schema_version": 1,
                    "tool": "atlasctl",
                    "kind": "list-checks-tree",
                    "status": "ok",
                    "tree": [
                        {"domain": domain, "areas": [{"name": area, "checks": sorted(ids)} for area, ids in sorted(areas.items())]}
                        for domain, areas in sorted(tree.items())
                    ],
                }
                _render(payload, as_json=True)
                return 0
            for domain, areas in sorted(tree.items()):
                print(domain)
                for area, ids in sorted(areas.items()):
                    print(f"  {area}")
                    for check_id in sorted(ids):
                        print(f"    - {check_id}")
            return 0
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "kind": "list-checks",
            "status": "ok",
            "filters": {"tags": list(tags), "pattern": pattern or "", "include_internal": include_internal},
            "items": [
                {
                    "id": rec.id,
                    "canonical_id": rec.id,
                    "title": rec.title,
                    "domain": rec.domain,
                    "tags": list(rec.tags),
                    "effects": list(rec.effects),
                    "owners": list(rec.owners),
                    "internal": rec.internal,
                }
                for rec in records
            ],
        }
        _render(payload, as_json)
        return 0
    if ns.list_kind == "commands":
        records = _filter_records(_command_records(), tags, pattern, include_internal)
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "kind": "list-commands",
            "status": "ok",
            "filters": {"tags": list(tags), "pattern": pattern or "", "include_internal": include_internal},
            "items": [
                {
                    "name": rec.name,
                    "help": rec.help,
                    "tags": list(rec.tags),
                    "owner": rec.owner,
                    "aliases": list(rec.aliases),
                    "stable": rec.stable,
                    "internal": rec.internal,
                }
                for rec in records
            ],
        }
        _render(payload, as_json)
        return 0
    if ns.list_kind == "policies":
        records = [item for item in _POLICIES_ITEMS if _pattern_match(item, pattern)]
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "kind": "list-policies",
            "status": "ok",
            "filters": {"pattern": pattern or ""},
            "items": records,
        }
        _render(payload, as_json)
        return 0

    records = _filter_records(_suite_records(ctx), tags, pattern, include_internal)
    manifests = load_first_class_suites()
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "kind": "list-suites",
        "status": "ok",
        "filters": {"tags": list(tags), "pattern": pattern or "", "include_internal": include_internal},
        "items": [
            {
                "name": rec.name,
                "includes": list(rec.includes),
                "item_count": rec.item_count,
                "complete": rec.complete,
                "tags": list(rec.tags),
                "internal": rec.internal,
                "markers": list(manifests[rec.name].markers) if rec.name in manifests else [],
                "check_count": len(manifests[rec.name].check_ids) if rec.name in manifests else None,
                "kind": "first-class" if rec.name in manifests else "pyproject",
            }
            for rec in records
        ],
    }
    _render(payload, as_json)
    return 0


def configure_list_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    parser = sub.add_parser("list", help="list checks, commands, and suites from canonical registries")
    parser.add_argument("list_kind", choices=["checks", "commands", "suites", "policies"])
    parser.add_argument("--json", action="store_true", help="emit JSON output")
    parser.add_argument("--format", choices=["text", "json", "tree"], default="text", help="render format for list output")
    parser.add_argument("--tags", default="", help="comma-separated tag filters")
    parser.add_argument("--include-internal", action="store_true", help=argparse.SUPPRESS)
    parser.add_argument("--pattern", help="substring or regex pattern filter")
