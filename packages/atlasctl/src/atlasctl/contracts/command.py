from __future__ import annotations

import argparse
import json
import re
import sqlite3
import subprocess
from pathlib import Path
from typing import Any

from ..core.context import RunContext


def _load(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def _check_drift(repo_root: Path) -> list[str]:
    errors: list[str] = []
    contracts = repo_root / "docs/contracts"
    errors_json = _load(contracts / "ERROR_CODES.json")
    metrics_json = _load(contracts / "METRICS.json")
    spans_json = _load(contracts / "TRACE_SPANS.json")
    endpoints_json = _load(contracts / "ENDPOINTS.json")
    chart_json = _load(contracts / "CHART_VALUES.json")

    if errors_json["codes"] != sorted(errors_json["codes"]):
        errors.append("ERROR_CODES.json codes not sorted")
    if chart_json["top_level_keys"] != sorted(chart_json["top_level_keys"]):
        errors.append("CHART_VALUES.json top_level_keys not sorted")

    rust_generated = (repo_root / "crates/bijux-atlas-api/src/generated/error_codes.rs").read_text(encoding="utf-8")
    for code in errors_json["codes"]:
        if f'"{code}"' not in rust_generated:
            errors.append(f"missing generated rust error code: {code}")

    openapi_snapshot = _load(repo_root / "configs/openapi/v1/openapi.snapshot.json")
    openapi_codes = openapi_snapshot.get("components", {}).get("schemas", {}).get("ApiErrorCode", {}).get("enum", [])
    if sorted(openapi_codes) != sorted(errors_json["codes"]):
        errors.append("OpenAPI ApiErrorCode enum drift from ERROR_CODES.json")

    obs_metrics = _load(repo_root / "ops/obs/contract/metrics-contract.json")
    obs_set = set(obs_metrics.get("required_metrics", {}).keys())
    contract_set = {m["name"] for m in metrics_json["metrics"]}
    if contract_set != obs_set:
        errors.append("METRICS.json drift from ops/obs/contract/metrics-contract.json")

    span_names = [s["name"] for s in spans_json["spans"]]
    obs_spans = obs_metrics.get("required_spans", [])
    if sorted(obs_spans) != sorted(span_names):
        errors.append("TRACE_SPANS.json drift from ops/obs/contract/metrics-contract.json")

    server_src = (repo_root / "crates/bijux-atlas-server/src/runtime/server_runtime_app.rs").read_text(encoding="utf-8")
    route_paths = set()
    for p in re.findall(r'\.route\(\s*"([^"]+)"', server_src, flags=re.MULTILINE):
        p = re.sub(r":([A-Za-z_][A-Za-z0-9_]*)", r"{\1}", p)
        if p != "/":
            route_paths.add(p)
    contract_paths = {e["path"] for e in endpoints_json["endpoints"]}
    if route_paths != contract_paths:
        errors.append("ENDPOINTS.json drift from server routes")
    if contract_paths != set(openapi_snapshot.get("paths", {}).keys()):
        errors.append("ENDPOINTS.json drift from OpenAPI paths")
    return errors


def _check_endpoints(repo_root: Path) -> list[str]:
    errors: list[str] = []
    contract = _load(repo_root / "docs/contracts/ENDPOINTS.json")
    contract_paths = {e["path"] for e in contract["endpoints"]}
    for path in contract_paths:
        if path.startswith("/v") and not path.startswith("/v1/"):
            errors.append(f"non-v1 API path in v1 contract: {path}")
    server_src = (repo_root / "crates/bijux-atlas-server/src/runtime/server_runtime_app.rs").read_text(encoding="utf-8")
    route_paths = set()
    for p in re.findall(r'\.route\(\s*"([^"]+)"', server_src, flags=re.MULTILINE):
        p = re.sub(r":([A-Za-z_][A-Za-z0-9_]*)", r"{\1}", p)
        if p != "/":
            route_paths.add(p)
    openapi = _load(repo_root / "configs/openapi/v1/openapi.snapshot.json")
    openapi_paths = set(openapi.get("paths", {}).keys())
    if route_paths != contract_paths:
        errors.append("endpoint contract drift with server routing")
    if openapi_paths != contract_paths:
        errors.append("endpoint contract drift with OpenAPI")
    return errors


def _check_error_codes(repo_root: Path) -> list[str]:
    errors: list[str] = []
    error_codes = _load(repo_root / "docs/contracts/ERROR_CODES.json")["codes"]
    status_map = _load(repo_root / "docs/contracts/ERROR_STATUS_MAP.json")["mappings"]
    errors_doc = (repo_root / "docs/contracts/errors.md").read_text(encoding="utf-8")
    openapi = _load(repo_root / "configs/openapi/v1/openapi.snapshot.json")
    openapi_codes = openapi.get("components", {}).get("schemas", {}).get("ApiErrorCode", {}).get("enum", [])
    if sorted(error_codes) != sorted(openapi_codes):
        errors.append("OpenAPI error code enum drift")
    rust_generated = (repo_root / "crates/bijux-atlas-api/src/generated/error_codes.rs").read_text(encoding="utf-8")
    for code in error_codes:
        if f'"{code}"' not in rust_generated:
            errors.append(f"missing generated code in rust constants: {code}")
        if f"### `{code}`" not in errors_doc:
            errors.append(f"missing docs entry in docs/contracts/errors.md: {code}")
        if not status_map.get(code):
            errors.append(f"missing HTTP status mapping for code: {code}")
    return errors


def _check_sqlite_indexes(repo_root: Path) -> list[str]:
    contract = _load(repo_root / "docs/contracts/SQLITE_INDEXES.json")
    schema_sql = (repo_root / "crates/bijux-atlas-ingest/sql/schema_v4.sql").read_text(encoding="utf-8")
    conn = sqlite3.connect(":memory:")
    conn.executescript(schema_sql)
    existing_indexes = {
        row[0]
        for row in conn.execute("SELECT name FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%'")
    }
    expected_indexes = set()
    for names in contract["required_indexes"].values():
        expected_indexes.update(names)
    errors: list[str] = []
    missing_indexes = sorted(expected_indexes - existing_indexes)
    if missing_indexes:
        errors.append("missing required indexes: " + ", ".join(missing_indexes))
    existing_tables = {row[0] for row in conn.execute("SELECT name FROM sqlite_master WHERE type='table'")}
    missing_vtables = sorted(set(contract.get("required_virtual_tables", [])) - existing_tables)
    if missing_vtables:
        errors.append("missing required virtual tables: " + ", ".join(missing_vtables))
    return errors


def _check_chart_values(repo_root: Path) -> list[str]:
    contract = _load(repo_root / "docs/contracts/CHART_VALUES.json")
    expected = set(contract["top_level_keys"])
    values_text = (repo_root / "ops/k8s/charts/bijux-atlas/values.yaml").read_text(encoding="utf-8")
    actual = {m.group(1) for m in re.finditer(r"^([A-Za-z][A-Za-z0-9_]*)\s*:", values_text, flags=re.MULTILINE)}
    missing_in_contract = sorted(actual - expected)
    extra_in_contract = sorted(expected - actual)
    errors: list[str] = []
    if missing_in_contract:
        errors.append("chart values missing in CHART_VALUES.json: " + ", ".join(missing_in_contract))
    if extra_in_contract:
        errors.append("extra contract keys not in values.yaml: " + ", ".join(extra_in_contract))
    return errors


def _breaking_removed(prev: dict[str, Any], cur: dict[str, Any]) -> list[str]:
    out: list[str] = []
    prev_endpoints = {(e["method"], e["path"]) for e in prev.get("endpoints", [])}
    cur_endpoints = {(e["method"], e["path"]) for e in cur.get("endpoints", [])}
    removed = sorted(prev_endpoints - cur_endpoints)
    if removed:
        out.append(f"removed endpoints: {removed}")
    return out


def _check_breaking(repo_root: Path, before: Path | None, after: Path | None) -> list[str]:
    if before and after:
        prev = _load(before)
        cur = _load(after)
        return _breaking_removed(prev, cur)
    # default conservative: compare previous tag if available
    proc = subprocess.run(
        ["git", "tag", "--list", "--sort=-creatordate", "v*"],
        cwd=repo_root,
        text=True,
        capture_output=True,
        check=False,
    )
    tags = [t.strip() for t in proc.stdout.splitlines() if t.strip()] if proc.returncode == 0 else []
    if not tags:
        return []
    base_ref = tags[0]
    show = subprocess.run(
        ["git", "show", f"{base_ref}:docs/contracts/ENDPOINTS.json"],
        cwd=repo_root,
        text=True,
        capture_output=True,
        check=False,
    )
    if show.returncode != 0:
        return []
    prev = json.loads(show.stdout)
    cur = _load(repo_root / "docs/contracts/ENDPOINTS.json")
    return _breaking_removed(prev, cur)


def _gen_openapi(repo_root: Path) -> list[str]:
    out = repo_root / "configs/openapi/v1/openapi.generated.json"
    proc = subprocess.run(
        ["cargo", "run", "--quiet", "-p", "bijux-atlas-api", "--bin", "atlas-openapi", "--", "--out", str(out)],
        cwd=repo_root,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        return [proc.stderr.strip() or "openapi generation failed"]
    return []


def _gen_chart_schema(repo_root: Path) -> list[str]:
    contract = _load(repo_root / "docs/contracts/CHART_VALUES.json")
    keys = contract["top_level_keys"]
    schema = {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "bijux-atlas chart values",
        "type": "object",
        "additionalProperties": False,
        "properties": {k: {"description": f"Chart values key `{k}` from SSOT contract."} for k in keys},
    }
    out = repo_root / "ops/k8s/charts/bijux-atlas/values.schema.json"
    out.write_text(json.dumps(schema, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return []


def _gen_contract_artifacts(repo_root: Path) -> list[str]:
    contracts = repo_root / "docs/contracts"
    out_gen = repo_root / "docs/_generated/contracts"
    out_gen.mkdir(parents=True, exist_ok=True)

    error_codes = _load(contracts / "ERROR_CODES.json")["codes"]
    metrics = _load(contracts / "METRICS.json")["metrics"]
    trace_spans = _load(contracts / "TRACE_SPANS.json")["spans"]
    endpoints = _load(contracts / "ENDPOINTS.json")["endpoints"]
    chart_keys = _load(contracts / "CHART_VALUES.json")["top_level_keys"]

    core_generated_dir = repo_root / "crates/bijux-atlas-core/src/generated"
    core_generated_dir.mkdir(parents=True, exist_ok=True)
    (core_generated_dir / "mod.rs").write_text("pub mod error_codes;\n", encoding="utf-8")

    core_rust_path = core_generated_dir / "error_codes.rs"
    core_rust = [
        "// @generated by atlasctl contracts generate",
        "use serde::{Deserialize, Serialize};",
        "",
        "#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]",
        "#[non_exhaustive]",
        "pub enum ErrorCode {",
    ]
    for code in error_codes:
        core_rust.append(f"    {code},")
    core_rust.extend(
        [
            "}",
            "",
            "impl ErrorCode {",
            "    #[must_use]",
            "    pub const fn as_str(self) -> &'static str {",
            "        match self {",
        ]
    )
    for code in error_codes:
        core_rust.append(f'            Self::{code} => "{code}",')
    core_rust.extend(
        [
            "        }",
            "    }",
            "",
            "    pub fn parse(value: &str) -> Option<Self> {",
            "        match value {",
        ]
    )
    for code in error_codes:
        core_rust.append(f'            "{code}" => Some(Self::{code}),')
    core_rust.extend(["            _ => None,", "        }", "    }", "}", "", "pub const ERROR_CODES: &[&str] = &["])
    for code in error_codes:
        core_rust.append(f'    "{code}",')
    core_rust.append("];\n")
    core_rust_path.write_text("\n".join(core_rust), encoding="utf-8")

    api_rust_path = repo_root / "crates/bijux-atlas-api/src/generated/error_codes.rs"
    api_rust_path.parent.mkdir(parents=True, exist_ok=True)
    api_rust = ["// @generated by atlasctl contracts generate", "pub const API_ERROR_CODES: &[&str] = &["]
    for code in error_codes:
        api_rust.append(f'    "{code}",')
    api_rust.extend(["];", "", "pub type ApiErrorCode = bijux_atlas_core::ErrorCode;", ""])
    api_rust_path.write_text("\n".join(api_rust), encoding="utf-8")

    server_gen_dir = repo_root / "crates/bijux-atlas-server/src/telemetry/generated"
    server_gen_dir.mkdir(parents=True, exist_ok=True)
    (server_gen_dir / "mod.rs").write_text(
        "// @generated by atlasctl contracts generate\npub mod metrics_contract;\npub mod trace_spans_contract;\n",
        encoding="utf-8",
    )
    metrics_rs = [
        "// @generated by atlasctl contracts generate",
        "pub const CONTRACT_METRIC_NAMES: &[&str] = &[",
    ]
    for metric in metrics:
        metrics_rs.append(f'    "{metric["name"]}",')
    metrics_rs.append("];\n")
    (server_gen_dir / "metrics_contract.rs").write_text("\n".join(metrics_rs), encoding="utf-8")

    spans_rs = [
        "// @generated by atlasctl contracts generate",
        "pub const CONTRACT_TRACE_SPAN_NAMES: &[&str] = &[",
    ]
    for span in trace_spans:
        spans_rs.append(f'    "{span["name"]}",')
    spans_rs.append("];\n")
    (server_gen_dir / "trace_spans_contract.rs").write_text("\n".join(spans_rs), encoding="utf-8")

    for rust_file in (
        core_generated_dir / "mod.rs",
        core_rust_path,
        api_rust_path,
        server_gen_dir / "mod.rs",
        server_gen_dir / "metrics_contract.rs",
        server_gen_dir / "trace_spans_contract.rs",
    ):
        subprocess.run(["rustfmt", str(rust_file)], cwd=repo_root, text=True, check=False)

    (out_gen / "ERROR_CODES.md").write_text(
        "# Error Codes (Generated)\n\n" + "\n".join(f"- `{code}`" for code in error_codes) + "\n",
        encoding="utf-8",
    )
    (out_gen / "METRICS.md").write_text(
        "# Metrics (Generated)\n\n"
        + "\n".join(f"- `{metric['name']}` labels: {', '.join(metric['labels'])}" for metric in metrics)
        + "\n",
        encoding="utf-8",
    )
    (out_gen / "TRACE_SPANS.md").write_text(
        "# Trace Spans (Generated)\n\n"
        + "\n".join(
            f"- `{span['name']}` attrs: {', '.join(span['required_attributes'])}" for span in trace_spans
        )
        + "\n",
        encoding="utf-8",
    )
    (out_gen / "ENDPOINTS.md").write_text(
        "# Endpoints (Generated)\n\n"
        + "\n".join(
            f"- `{endpoint['method']} {endpoint['path']}` telemetry: `{endpoint['telemetry_class']}`"
            for endpoint in endpoints
        )
        + "\n",
        encoding="utf-8",
    )
    (out_gen / "CHART_VALUES.md").write_text(
        "# Chart Values Keys (Generated)\n\n" + "\n".join(f"- `{key}`" for key in chart_keys) + "\n",
        encoding="utf-8",
    )
    return []


def run_contracts_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    report = getattr(ns, "report", "text")
    errors: list[str] = []
    if ns.contracts_cmd == "check":
        checks = ns.checks or ["breakage", "drift", "endpoints", "error-codes", "sqlite-indexes", "chart-values"]
        for check in checks:
            if check == "breakage":
                errors.extend(_check_breaking(ctx.repo_root, ns.before, ns.after))
            elif check == "drift":
                errors.extend(_check_drift(ctx.repo_root))
            elif check == "endpoints":
                errors.extend(_check_endpoints(ctx.repo_root))
            elif check == "error-codes":
                errors.extend(_check_error_codes(ctx.repo_root))
            elif check == "sqlite-indexes":
                errors.extend(_check_sqlite_indexes(ctx.repo_root))
            elif check == "chart-values":
                errors.extend(_check_chart_values(ctx.repo_root))
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "pass" if not errors else "fail",
            "errors": errors,
        }
        if report == "json":
            rendered = json.dumps(payload, sort_keys=True)
        else:
            rendered = json.dumps(payload, indent=2, sort_keys=True)
        print(rendered)
        return 0 if not errors else 1

    if ns.contracts_cmd == "generate":
        gens = ns.generators or ["openapi", "chart-schema", "artifacts"]
        for gen in gens:
            if gen == "openapi":
                errors.extend(_gen_openapi(ctx.repo_root))
            elif gen == "chart-schema":
                errors.extend(_gen_chart_schema(ctx.repo_root))
            elif gen == "artifacts":
                errors.extend(_gen_contract_artifacts(ctx.repo_root))
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "pass" if not errors else "fail",
            "errors": errors,
        }
        if report == "json":
            rendered = json.dumps(payload, sort_keys=True)
        else:
            rendered = json.dumps(payload, indent=2, sort_keys=True)
        print(rendered)
        return 0 if not errors else 1
    return 2


def configure_contracts_parser(sub: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:
    p = sub.add_parser("contracts", help="contracts checks and generators")
    subp = p.add_subparsers(dest="contracts_cmd", required=True)

    check = subp.add_parser(
        "check",
        help="check contract breakage/drift/endpoints/error-codes/sqlite-indexes/chart-values",
    )
    check.add_argument("--report", choices=["text", "json"], default="text")
    check.add_argument(
        "--checks",
        nargs="*",
        choices=["breakage", "drift", "endpoints", "error-codes", "sqlite-indexes", "chart-values"],
    )
    check.add_argument("--before", type=Path, help="fixture json for previous contract")
    check.add_argument("--after", type=Path, help="fixture json for next contract")

    gen = subp.add_parser("generate", help="generate openapi/chart schema/contract artifacts")
    gen.add_argument("--report", choices=["text", "json"], default="text")
    gen.add_argument("--generators", nargs="*", choices=["openapi", "chart-schema", "artifacts"])
