from __future__ import annotations

import argparse
import json
from pathlib import Path

from ..core.context import RunContext
from .catalog import lint_catalog, list_catalog_entries, check_schema_change_release_policy, check_schema_readme_sync
from .output.base import build_output_base
from .validate_self import validate_self
from .checks import (
    check_breaking,
    check_chart_values,
    check_drift,
    check_endpoints,
    check_error_codes,
    check_sqlite_indexes,
)
from .generators import generate_chart_schema, generate_contract_artifacts, generate_openapi, generate_schema_samples, generate_schema_catalog

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
    "samples": generate_schema_samples,
    "catalog": generate_schema_catalog,
}


def _iter_contract_payloads(repo_root: Path) -> list[tuple[str, dict[str, object]]]:
    files: list[Path] = []
    files.extend(sorted((repo_root / "packages/atlasctl/tests/goldens").rglob("*.json.golden")))
    files.extend(sorted((repo_root / "packages/atlasctl/tests/goldens/samples").rglob("*.json")))
    out: list[tuple[str, dict[str, object]]] = []
    for path in files:
        text = path.read_text(encoding="utf-8", errors="ignore").strip()
        if not text.startswith("{"):
            continue
        try:
            payload = json.loads(text)
        except json.JSONDecodeError:
            continue
        if not isinstance(payload, dict):
            continue
        schema_name = payload.get("schema_name")
        if isinstance(schema_name, str):
            out.append((path.relative_to(repo_root).as_posix(), payload))
    return out


def _as_json(ns: argparse.Namespace, report: str) -> bool:
    return bool(getattr(ns, "json", False) or report == "json")


def _emit(run_id: str, ns: argparse.Namespace, report: str, errors: list[str], warnings: list[str] | None = None, meta: dict[str, object] | None = None) -> int:
    payload = build_output_base(
        run_id=run_id,
        ok=not errors,
        errors=errors,
        warnings=warnings or [],
        meta=meta or {},
    )
    payload["status"] = "pass" if not errors else "fail"
    print(json.dumps(payload, sort_keys=True) if _as_json(ns, report) else json.dumps(payload, indent=2, sort_keys=True))
    return 0 if not errors else 1


def run_contracts_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    report = getattr(ns, "report", "text")
    as_json = _as_json(ns, report)
    if ns.contracts_cmd == "list":
        rows = [
            {"schema_name": entry.name, "schema_version": entry.version, "file": entry.file}
            for entry in sorted(list_catalog_entries(), key=lambda row: row.name)
        ]
        payload = build_output_base(run_id=ctx.run_id, ok=True, meta={"schemas": rows})
        payload["status"] = "ok"
        print(json.dumps(payload, sort_keys=True) if as_json else json.dumps(payload, indent=2, sort_keys=True))
        return 0
    if ns.contracts_cmd == "lint":
        errors = lint_catalog()
        return _emit(ctx.run_id, ns, report, errors, meta={"checks": ["catalog_lint"]})
    if ns.contracts_cmd == "validate":
        errors: list[str] = []
        errors.extend(lint_catalog())
        for entry in list_catalog_entries():
            schema_path = ctx.repo_root / "packages/atlasctl/src/atlasctl/contracts/schemas" / entry.file
            if not schema_path.exists():
                errors.append(f"missing schema file: {entry.file}")
                continue
            try:
                json.loads(schema_path.read_text(encoding="utf-8"))
            except json.JSONDecodeError as exc:
                errors.append(f"invalid json schema file {entry.file}: {exc}")
        return _emit(ctx.run_id, ns, report, errors, meta={"checks": ["catalog_lint", "schema_json_parse"]})
    if ns.contracts_cmd == "validate-self":
        errors: list[str] = []
        validated: list[str] = []
        for rel, payload in _iter_contract_payloads(ctx.repo_root):
            try:
                schema_name = str(payload["schema_name"])
                validate_self(schema_name, payload)
                validated.append(rel)
            except Exception as exc:
                errors.append(f"{rel}: {exc}")
        errors.extend(check_schema_readme_sync())
        errors.extend(check_schema_change_release_policy(ctx.repo_root))
        return _emit(
            ctx.run_id,
            ns,
            report,
            errors,
            meta={"validated_payloads": sorted(validated), "checks": ["self_schema_validation", "schema_readme_sync", "schema_change_release_policy"]},
        )
    if ns.contracts_cmd == "check":
        checks = ns.checks or list(CHECK_HANDLERS)
        errors: list[str] = []
        for check in checks:
            errors.extend(CHECK_HANDLERS[check](ctx.repo_root, ns))
        return _emit(ctx.run_id, ns, report, errors, meta={"checks": checks})
    if ns.contracts_cmd == "generate":
        generators = ns.generators or list(GEN_HANDLERS)
        errors: list[str] = []
        for generator in generators:
            errors.extend(GEN_HANDLERS[generator](ctx.repo_root))
        return _emit(ctx.run_id, ns, report, errors, meta={"generators": generators})
    return 2


def configure_contracts_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    parser = sub.add_parser("contracts", help="contracts checks and generators")
    parser.add_argument("--json", action="store_true", help="emit JSON output")
    subp = parser.add_subparsers(dest="contracts_cmd", required=True)

    lst = subp.add_parser("list", help="list schema ids and versions from catalog")
    lst.add_argument("--report", choices=["text", "json"], default="text")

    lint = subp.add_parser("lint", help="lint schema catalog naming/order/coverage rules")
    lint.add_argument("--report", choices=["text", "json"], default="text")
    validate = subp.add_parser("validate", help="validate schema catalog and schema JSON files")
    validate.add_argument("--report", choices=["text", "json"], default="text")
    validate_self_cmd = subp.add_parser("validate-self", help="validate contract payloads against declared schemas")
    validate_self_cmd.add_argument("--report", choices=["text", "json"], default="text")

    check = subp.add_parser("check", help="check contract breakage/drift/endpoints/error-codes/sqlite-indexes/chart-values")
    check.add_argument("--report", choices=["text", "json"], default="text")
    check.add_argument("--checks", nargs="*", choices=list(CHECK_HANDLERS))
    check.add_argument("--before", type=Path, help="fixture json for previous contract")
    check.add_argument("--after", type=Path, help="fixture json for next contract")

    generator = subp.add_parser("generate", help="generate openapi/chart schema/contract artifacts")
    generator.add_argument("--report", choices=["text", "json"], default="text")
    generator.add_argument("--generators", nargs="*", choices=list(GEN_HANDLERS))
