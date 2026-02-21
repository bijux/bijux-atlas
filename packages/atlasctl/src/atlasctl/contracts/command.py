from __future__ import annotations

import argparse
import json
from pathlib import Path

from ..core.context import RunContext
from .catalog import lint_catalog, list_catalog_entries
from .checks import (
    check_breaking,
    check_chart_values,
    check_drift,
    check_endpoints,
    check_error_codes,
    check_sqlite_indexes,
)
from .generators import generate_chart_schema, generate_contract_artifacts, generate_openapi

CHECK_HANDLERS = {
    "breakage": lambda root, ns: check_breaking(root, ns.before, ns.after),
    "drift": lambda root, _ns: check_drift(root),
    "endpoints": lambda root, _ns: check_endpoints(root),
    "error-codes": lambda root, _ns: check_error_codes(root),
    "sqlite-indexes": lambda root, _ns: check_sqlite_indexes(root),
    "chart-values": lambda root, _ns: check_chart_values(root),
}

GEN_HANDLERS = {
    "openapi": generate_openapi,
    "chart-schema": generate_chart_schema,
    "artifacts": generate_contract_artifacts,
}


def _emit(report: str, errors: list[str]) -> int:
    payload = {
        "schema_version": 1,
        "tool": "atlasctl",
        "status": "pass" if not errors else "fail",
        "errors": errors,
    }
    print(json.dumps(payload, sort_keys=True) if report == "json" else json.dumps(payload, indent=2, sort_keys=True))
    return 0 if not errors else 1


def run_contracts_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    report = getattr(ns, "report", "text")
    if ns.contracts_cmd == "list":
        rows = [
            {"schema_name": entry.name, "schema_version": entry.version, "file": entry.file}
            for entry in sorted(list_catalog_entries(), key=lambda row: row.name)
        ]
        payload = {"schema_version": 1, "tool": "atlasctl", "status": "ok", "schemas": rows}
        print(json.dumps(payload, sort_keys=True) if report == "json" else json.dumps(payload, indent=2, sort_keys=True))
        return 0
    if ns.contracts_cmd == "lint":
        errors = lint_catalog()
        return _emit(report, errors)
    if ns.contracts_cmd == "check":
        checks = ns.checks or list(CHECK_HANDLERS)
        errors: list[str] = []
        for check in checks:
            errors.extend(CHECK_HANDLERS[check](ctx.repo_root, ns))
        return _emit(report, errors)
    if ns.contracts_cmd == "generate":
        generators = ns.generators or list(GEN_HANDLERS)
        errors: list[str] = []
        for generator in generators:
            errors.extend(GEN_HANDLERS[generator](ctx.repo_root))
        return _emit(report, errors)
    return 2


def configure_contracts_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    parser = sub.add_parser("contracts", help="contracts checks and generators")
    subp = parser.add_subparsers(dest="contracts_cmd", required=True)

    lst = subp.add_parser("list", help="list schema ids and versions from catalog")
    lst.add_argument("--report", choices=["text", "json"], default="text")

    lint = subp.add_parser("lint", help="lint schema catalog naming/order/coverage rules")
    lint.add_argument("--report", choices=["text", "json"], default="text")

    check = subp.add_parser("check", help="check contract breakage/drift/endpoints/error-codes/sqlite-indexes/chart-values")
    check.add_argument("--report", choices=["text", "json"], default="text")
    check.add_argument("--checks", nargs="*", choices=list(CHECK_HANDLERS))
    check.add_argument("--before", type=Path, help="fixture json for previous contract")
    check.add_argument("--after", type=Path, help="fixture json for next contract")

    generator = subp.add_parser("generate", help="generate openapi/chart schema/contract artifacts")
    generator.add_argument("--report", choices=["text", "json"], default="text")
    generator.add_argument("--generators", nargs="*", choices=list(GEN_HANDLERS))
