from __future__ import annotations

import json
import re
import sqlite3
import subprocess
from pathlib import Path
from typing import Any


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def check_drift(repo_root: Path) -> list[str]:
    errors: list[str] = []
    contracts = repo_root / "docs/contracts"
    errors_json = load_json(contracts / "ERROR_CODES.json")
    metrics_json = load_json(contracts / "METRICS.json")
    spans_json = load_json(contracts / "TRACE_SPANS.json")
    endpoints_json = load_json(contracts / "ENDPOINTS.json")
    chart_json = load_json(contracts / "CHART_VALUES.json")

    if errors_json["codes"] != sorted(errors_json["codes"]):
        errors.append("ERROR_CODES.json codes not sorted")
    if chart_json["top_level_keys"] != sorted(chart_json["top_level_keys"]):
        errors.append("CHART_VALUES.json top_level_keys not sorted")

    rust_generated = (repo_root / "crates/bijux-atlas-api/src/generated/error_codes.rs").read_text(encoding="utf-8")
    for code in errors_json["codes"]:
        if f'"{code}"' not in rust_generated:
            errors.append(f"missing generated rust error code: {code}")

    openapi_snapshot = load_json(repo_root / "configs/openapi/v1/openapi.snapshot.json")
    openapi_codes = openapi_snapshot.get("components", {}).get("schemas", {}).get("ApiErrorCode", {}).get("enum", [])
    if sorted(openapi_codes) != sorted(errors_json["codes"]):
        errors.append("OpenAPI ApiErrorCode enum drift from ERROR_CODES.json")

    obs_metrics = load_json(repo_root / "ops/obs/contract/metrics-contract.json")
    obs_set = set(obs_metrics.get("required_metrics", {}).keys())
    contract_set = {m["name"] for m in metrics_json["metrics"]}
    if contract_set != obs_set:
        errors.append("METRICS.json drift from ops/obs/contract/metrics-contract.json")

    span_names = [s["name"] for s in spans_json["spans"]]
    obs_spans = obs_metrics.get("required_spans", [])
    if sorted(obs_spans) != sorted(span_names):
        errors.append("TRACE_SPANS.json drift from ops/obs/contract/metrics-contract.json")

    server_src = (repo_root / "crates/bijux-atlas-server/src/runtime/server_runtime_app.rs").read_text(encoding="utf-8")
    route_paths: set[str] = set()
    for path in re.findall(r'\.route\(\s*"([^"]+)"', server_src, flags=re.MULTILINE):
        normalized = re.sub(r":([A-Za-z_][A-Za-z0-9_]*)", r"{\1}", path)
        if normalized != "/":
            route_paths.add(normalized)
    contract_paths = {entry["path"] for entry in endpoints_json["endpoints"]}
    if route_paths != contract_paths:
        errors.append("ENDPOINTS.json drift from server routes")
    if contract_paths != set(openapi_snapshot.get("paths", {}).keys()):
        errors.append("ENDPOINTS.json drift from OpenAPI paths")
    return errors


def check_endpoints(repo_root: Path) -> list[str]:
    errors: list[str] = []
    contract = load_json(repo_root / "docs/contracts/ENDPOINTS.json")
    contract_paths = {entry["path"] for entry in contract["endpoints"]}
    for path in contract_paths:
        if path.startswith("/v") and not path.startswith("/v1/"):
            errors.append(f"non-v1 API path in v1 contract: {path}")
    server_src = (repo_root / "crates/bijux-atlas-server/src/runtime/server_runtime_app.rs").read_text(encoding="utf-8")
    route_paths: set[str] = set()
    for path in re.findall(r'\.route\(\s*"([^"]+)"', server_src, flags=re.MULTILINE):
        normalized = re.sub(r":([A-Za-z_][A-Za-z0-9_]*)", r"{\1}", path)
        if normalized != "/":
            route_paths.add(normalized)
    openapi = load_json(repo_root / "configs/openapi/v1/openapi.snapshot.json")
    if route_paths != contract_paths:
        errors.append("endpoint contract drift with server routing")
    if set(openapi.get("paths", {}).keys()) != contract_paths:
        errors.append("endpoint contract drift with OpenAPI")
    return errors


def check_error_codes(repo_root: Path) -> list[str]:
    errors: list[str] = []
    error_codes = load_json(repo_root / "docs/contracts/ERROR_CODES.json")["codes"]
    status_map = load_json(repo_root / "docs/contracts/ERROR_STATUS_MAP.json")["mappings"]
    errors_doc = (repo_root / "docs/contracts/errors.md").read_text(encoding="utf-8")
    openapi = load_json(repo_root / "configs/openapi/v1/openapi.snapshot.json")
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


def check_sqlite_indexes(repo_root: Path) -> list[str]:
    contract = load_json(repo_root / "docs/contracts/SQLITE_INDEXES.json")
    schema_sql = (repo_root / "crates/bijux-atlas-ingest/sql/schema_v4.sql").read_text(encoding="utf-8")
    conn = sqlite3.connect(":memory:")
    conn.executescript(schema_sql)
    existing_indexes = {
        row[0]
        for row in conn.execute("SELECT name FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%'")
    }
    expected_indexes: set[str] = set()
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


def check_chart_values(repo_root: Path) -> list[str]:
    contract = load_json(repo_root / "docs/contracts/CHART_VALUES.json")
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


def check_breaking(repo_root: Path, before: Path | None, after: Path | None) -> list[str]:
    def breaking_removed(previous: dict[str, Any], current: dict[str, Any]) -> list[str]:
        previous_endpoints = {(e["method"], e["path"]) for e in previous.get("endpoints", [])}
        current_endpoints = {(e["method"], e["path"]) for e in current.get("endpoints", [])}
        removed = sorted(previous_endpoints - current_endpoints)
        return [f"removed endpoints: {removed}"] if removed else []

    if before and after:
        return breaking_removed(load_json(before), load_json(after))
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
    show = subprocess.run(
        ["git", "show", f"{tags[0]}:docs/contracts/ENDPOINTS.json"],
        cwd=repo_root,
        text=True,
        capture_output=True,
        check=False,
    )
    if show.returncode != 0:
        return []
    return breaking_removed(json.loads(show.stdout), load_json(repo_root / "docs/contracts/ENDPOINTS.json"))
