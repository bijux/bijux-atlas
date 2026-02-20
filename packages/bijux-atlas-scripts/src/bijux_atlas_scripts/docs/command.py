from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Callable

from ..core.context import RunContext
from ..core.fs import ensure_evidence_path
from ..make.target_graph import parse_make_targets


@dataclass(frozen=True)
class DocsCheck:
    check_id: str
    description: str
    cmd: list[str] | None
    fn: Callable[[RunContext], tuple[int, str]] | None
    actionable: str


def _check(
    check_id: str,
    description: str,
    cmd: list[str] | None,
    actionable: str,
    fn: Callable[[RunContext], tuple[int, str]] | None = None,
) -> DocsCheck:
    return DocsCheck(check_id, description, cmd, fn, actionable)


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

DOCS_GENERATE_COMMANDS: list[list[str]] = [
    ["python3", "scripts/areas/docs/generate_crates_map.py"],
    ["python3", "scripts/areas/docs/generate_architecture_map.py"],
    ["python3", "scripts/areas/docs/generate_k8s_values_doc.py"],
    ["python3", "scripts/areas/docs/generate_concept_graph.py"],
    ["python3", "scripts/areas/docs/generate_openapi_docs.py"],
    ["python3", "scripts/areas/docs/generate_observability_surface.py"],
    ["python3", "scripts/areas/docs/generate_ops_badge.py"],
    ["python3", "scripts/areas/docs/generate_ops_schema_docs.py"],
    ["python3", "scripts/areas/docs/generate_ops_surface.py"],
    ["python3", "scripts/areas/docs/generate_ops_contracts_doc.py"],
    ["python3", "scripts/areas/docs/generate_make_targets_catalog.py"],
    ["python3", "scripts/areas/docs/generate_config_keys_doc.py"],
    ["python3", "scripts/areas/docs/generate_env_vars_doc.py"],
    ["python3", "scripts/areas/docs/generate_contracts_index_doc.py"],
    ["python3", "scripts/areas/docs/generate_chart_contract_index.py"],
    ["python3", "scripts/areas/ops/generate_k8s_test_surface.py"],
    ["python3", "scripts/areas/docs/generate_runbook_map_index.py"],
]


def _run_check(cmd: list[str], repo_root: Path) -> tuple[int, str]:
    proc = subprocess.run(cmd, cwd=repo_root, text=True, capture_output=True, check=False)
    output = (proc.stdout or "") + (proc.stderr or "")
    return proc.returncode, output.strip()


def _load_public_targets(repo_root: Path) -> set[str]:
    payload = json.loads((repo_root / "configs/ops/public-surface.json").read_text(encoding="utf-8"))
    return set(payload.get("make_targets", []))


def _load_public_surface_exceptions(repo_root: Path) -> set[str]:
    path = repo_root / "configs/ops/public-surface-doc-exceptions.txt"
    if not path.exists():
        return set()
    return {line.strip() for line in path.read_text(encoding="utf-8").splitlines() if line.strip() and not line.startswith("#")}


def _check_public_surface_docs(ctx: RunContext) -> tuple[int, str]:
    public_targets = _load_public_targets(ctx.repo_root)
    exceptions = _load_public_surface_exceptions(ctx.repo_root)
    docs_roots = [ctx.repo_root / "docs" / "operations", ctx.repo_root / "docs" / "quickstart", ctx.repo_root / "docs" / "development"]
    make_re = re.compile(r"\bmake\s+([a-zA-Z0-9_.-]+)")
    ops_script_re = re.compile(r"\./(ops/[^\s`]+(?:\.sh|\.py))")
    errs: list[str] = []
    for base in docs_roots:
        if not base.exists():
            continue
        for md in base.rglob("*.md"):
            text = md.read_text(encoding="utf-8", errors="ignore")
            rel = md.relative_to(ctx.repo_root).as_posix()
            for target in make_re.findall(text):
                if target == "ops-":
                    continue
                if not (target.startswith("ops-") or target in {"root", "root-local", "gates", "explain", "help"}):
                    continue
                key = f"{rel}::make {target}"
                if target not in public_targets and key not in exceptions:
                    errs.append(f"{rel}: non-public make target referenced: {target}")
            for script in ops_script_re.findall(text):
                key = f"{rel}::./{script}"
                if script.startswith("ops/run/"):
                    continue
                if key not in exceptions:
                    errs.append(f"{rel}: non-public ops script referenced: ./{script}")
    return (0, "") if not errs else (1, "\n".join(errs))


def _check_docs_make_only(ctx: RunContext) -> tuple[int, str]:
    docs = sorted((ctx.repo_root / "docs" / "operations").rglob("*.md"))
    patterns = [
        re.compile(r"(^|\s)(\./)?scripts/[\w./-]+"),
        re.compile(r"(^|\s)(\./)?ops/.+/scripts/[\w./-]+"),
    ]
    violations: list[str] = []
    for doc in docs:
        rel = doc.relative_to(ctx.repo_root).as_posix()
        for idx, line in enumerate(doc.read_text(encoding="utf-8").splitlines(), start=1):
            if line.strip().startswith("#"):
                continue
            if "`" not in line and "scripts/" not in line:
                continue
            if any(pat.search(line) for pat in patterns):
                violations.append(f"{rel}:{idx}: direct script path in docs; reference `make <target>` instead")
    return (0, "") if not violations else (1, "\n".join(violations))


def _snapshot_hashes(path: Path) -> dict[str, str]:
    if path.is_file():
        return {str(path): hashlib.sha256(path.read_bytes()).hexdigest()}
    if path.is_dir():
        out: dict[str, str] = {}
        for child in sorted(p for p in path.rglob("*") if p.is_file()):
            out[str(child)] = hashlib.sha256(child.read_bytes()).hexdigest()
        return out
    return {}


def _check_docs_freeze_drift(ctx: RunContext) -> tuple[int, str]:
    targets = [
        ctx.repo_root / "docs" / "_generated" / "contracts",
        ctx.repo_root / "docs" / "_generated" / "contracts" / "chart-contract-index.md",
        ctx.repo_root / "docs" / "_generated" / "openapi",
        ctx.repo_root / "docs" / "contracts" / "errors.md",
        ctx.repo_root / "docs" / "contracts" / "metrics.md",
        ctx.repo_root / "docs" / "contracts" / "tracing.md",
        ctx.repo_root / "docs" / "contracts" / "endpoints.md",
        ctx.repo_root / "docs" / "contracts" / "config-keys.md",
        ctx.repo_root / "docs" / "contracts" / "chart-values.md",
    ]
    before: dict[str, str] = {}
    for target in targets:
        before.update(_snapshot_hashes(target))
    cmds = [
        ["python3", "-m", "bijux_atlas_scripts.cli", "contracts", "generate", "--generators", "artifacts"],
        ["python3", "scripts/areas/docs/generate_chart_contract_index.py"],
    ]
    for cmd in cmds:
        code, output = _run_check(cmd, ctx.repo_root)
        if code != 0:
            return 1, output
    after: dict[str, str] = {}
    for target in targets:
        after.update(_snapshot_hashes(target))
    if before == after:
        return 0, ""
    changed = sorted({*before.keys(), *after.keys()})
    drift = [str(Path(path).relative_to(ctx.repo_root)) for path in changed if before.get(path) != after.get(path)]
    return 1, "\n".join(drift)


def _resolve_openapi_schema(schema: dict[str, object], schemas: dict[str, dict[str, object]]) -> dict[str, object]:
    ref = schema.get("$ref")
    if not isinstance(ref, str):
        return schema
    name = ref.split("/")[-1]
    target = schemas.get(name, {})
    return _resolve_openapi_schema(target, schemas)


def _validate_openapi_example(
    value: object,
    schema: dict[str, object],
    schemas: dict[str, dict[str, object]],
    path: str,
) -> list[str]:
    resolved = _resolve_openapi_schema(schema, schemas)
    errs: list[str] = []
    typ = resolved.get("type")
    if typ == "object":
        if not isinstance(value, dict):
            return [f"{path}: expected object"]
        required = resolved.get("required", [])
        if isinstance(required, list):
            for req in required:
                if isinstance(req, str) and req not in value:
                    errs.append(f"{path}: missing required field `{req}`")
        props = resolved.get("properties", {})
        if isinstance(props, dict):
            for key, item in value.items():
                prop_schema = props.get(key)
                if isinstance(prop_schema, dict):
                    errs.extend(_validate_openapi_example(item, prop_schema, schemas, f"{path}.{key}"))
    elif typ == "array":
        if not isinstance(value, list):
            return [f"{path}: expected array"]
        items = resolved.get("items", {})
        if isinstance(items, dict):
            for idx, item in enumerate(value):
                errs.extend(_validate_openapi_example(item, items, schemas, f"{path}[{idx}]"))
    elif typ == "string" and not isinstance(value, str):
        errs.append(f"{path}: expected string")
    elif typ == "integer" and not isinstance(value, int):
        errs.append(f"{path}: expected integer")
    elif typ == "number" and not isinstance(value, (int, float)):
        errs.append(f"{path}: expected number")
    elif typ == "boolean" and not isinstance(value, bool):
        errs.append(f"{path}: expected boolean")
    return errs


def _check_openapi_examples(ctx: RunContext) -> tuple[int, str]:
    openapi_path = ctx.repo_root / "configs" / "openapi" / "v1" / "openapi.generated.json"
    openapi = json.loads(openapi_path.read_text(encoding="utf-8"))
    schemas_obj = openapi.get("components", {}).get("schemas", {})
    schemas: dict[str, dict[str, object]] = {
        k: v for k, v in schemas_obj.items() if isinstance(k, str) and isinstance(v, dict)
    }
    errors: list[str] = []
    paths = openapi.get("paths", {})
    if isinstance(paths, dict):
        for route, methods in paths.items():
            if not isinstance(methods, dict):
                continue
            for method, op in methods.items():
                if not isinstance(op, dict):
                    continue
                responses = op.get("responses", {})
                if not isinstance(responses, dict):
                    continue
                for status, resp in responses.items():
                    if not isinstance(resp, dict):
                        continue
                    content = resp.get("content", {})
                    if not isinstance(content, dict):
                        continue
                    for media, media_obj in content.items():
                        if not isinstance(media_obj, dict):
                            continue
                        schema = media_obj.get("schema")
                        if not isinstance(schema, dict):
                            continue
                        example = media_obj.get("example")
                        if example is not None:
                            errors.extend(
                                _validate_openapi_example(example, schema, schemas, f"{method} {route} {status} {media}")
                            )
                        examples = media_obj.get("examples", {})
                        if isinstance(examples, dict):
                            for ex_name, ex in examples.items():
                                if not isinstance(ex, dict) or "value" not in ex:
                                    continue
                                errors.extend(
                                    _validate_openapi_example(
                                        ex["value"],
                                        schema,
                                        schemas,
                                        f"{method} {route} {status} {media} examples.{ex_name}",
                                    )
                                )
    return (0, "") if not errors else (1, "\n".join(errors[:200]))


def _read_json(path: Path) -> dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


def _as_str_list(value: object) -> list[str]:
    if isinstance(value, list):
        return [str(v) for v in value]
    if isinstance(value, dict):
        return [str(k) for k in value.keys()]
    return []


def _render_observability_surface(ctx: RunContext) -> str:
    metrics = _read_json(ctx.repo_root / "ops/obs/contract/metrics-contract.json")
    alerts = _read_json(ctx.repo_root / "ops/obs/contract/alerts-contract.json")
    dashboard = _read_json(ctx.repo_root / "ops/obs/contract/dashboard-panels-contract.json")
    logs = _read_json(ctx.repo_root / "ops/obs/contract/logs-fields-contract.json")

    metric_names = sorted(_as_str_list(metrics.get("required_metrics", [])))
    alert_names = sorted(_as_str_list(alerts.get("required_alerts", [])))
    dashboard_panels = sorted(_as_str_list(dashboard.get("required_panels", [])))
    log_fields = sorted(_as_str_list(logs.get("required_fields", [])))

    lines = [
        "# Observability Surface",
        "",
        "Generated from observability contract SSOT files:",
        "- `ops/obs/contract/metrics-contract.json`",
        "- `ops/obs/contract/alerts-contract.json`",
        "- `ops/obs/contract/dashboard-panels-contract.json`",
        "- `ops/obs/contract/logs-fields-contract.json`",
        "",
        "## Metrics",
    ]
    lines += [f"- `{name}`" for name in metric_names] or ["- _none_"]
    lines += ["", "## Alerts"]
    lines += [f"- `{name}`" for name in alert_names] or ["- _none_"]
    lines += ["", "## Dashboard Panels"]
    lines += [f"- `{name}`" for name in dashboard_panels] or ["- _none_"]
    lines += ["", "## Log Fields"]
    lines += [f"- `{name}`" for name in log_fields] or ["- _none_"]
    lines += ["", "## Verification", "```bash", "make ops-observability-validate", "```", ""]
    return "\n".join(lines)


def _check_observability_surface_drift(ctx: RunContext) -> tuple[int, str]:
    target = ctx.repo_root / "docs/_generated/observability-surface.md"
    before = target.read_text(encoding="utf-8") if target.exists() else ""
    rendered = _render_observability_surface(ctx)
    return (0, "") if before == rendered else (1, "observability surface drift detected; regenerate generated docs")


def _check_runbooks_contract(ctx: RunContext) -> tuple[int, str]:
    runbook_dir = ctx.repo_root / "docs" / "operations" / "runbooks"
    required_sections = [
        "Symptoms",
        "Metrics",
        "Commands",
        "Expected outputs",
        "Mitigations",
        "Alerts",
        "Rollback",
        "Postmortem checklist",
    ]
    alerts_contract = _read_json(ctx.repo_root / "ops" / "obs" / "contract" / "alerts-contract.json")
    alert_names = {str(v) for v in alerts_contract.get("required_alerts", []) if isinstance(v, str)}
    metrics_contract = _read_json(ctx.repo_root / "docs" / "contracts" / "METRICS.json")
    metrics = {str(m.get("name")) for m in metrics_contract.get("metrics", []) if isinstance(m, dict) and "name" in m}
    endpoints_contract = _read_json(ctx.repo_root / "docs" / "contracts" / "ENDPOINTS.json")
    endpoint_registry = {
        str(e.get("path")) for e in endpoints_contract.get("endpoints", []) if isinstance(e, dict) and "path" in e
    }
    endpoint_registry.update({"/metrics", "/healthz", "/readyz", "/debug/datasets", "/debug/registry-health"})
    make_targets = {name for name, _, _ in parse_make_targets(ctx.repo_root)}
    errors: list[str] = []
    for path in sorted(runbook_dir.glob("*.md")):
        if path.name == "INDEX.md":
            continue
        text = path.read_text(encoding="utf-8")
        rel = path.relative_to(ctx.repo_root).as_posix()
        for section in required_sections:
            if not re.search(rf"^##\s+{re.escape(section)}\s*$", text, flags=re.MULTILINE):
                errors.append(f"{rel}: missing section '## {section}'")
        for metric in re.findall(r"`(bijux_[a-z0-9_]+)`", text):
            if metric not in metrics:
                errors.append(f"{rel}: unknown metric `{metric}`")
        for endpoint in re.findall(r"(/(?:v1|metrics|healthz|readyz|debug)[a-zA-Z0-9_\-/{}:?=&.]*)", text):
            endpoint_base = endpoint.split("?")[0]
            if endpoint_base not in endpoint_registry:
                errors.append(f"{rel}: unknown endpoint `{endpoint_base}`")
        for cmd in re.findall(r"^\$\s+(.+)$", text, flags=re.MULTILINE):
            if cmd.startswith("make "):
                target = cmd.split()[1]
                if target not in make_targets:
                    errors.append(f"{rel}: unknown make target `{target}`")
        obs_dir = "observability"
        dashboard_pattern = rf"(docs/operations/{obs_dir}/dashboard\.md|\.\./{obs_dir}/dashboard\.md)"
        if not re.search(dashboard_pattern, text):
            errors.append(f"{rel}: missing dashboard link to observability dashboard")
        if not re.search(r"ops-drill-[a-z0-9-]+", text):
            errors.append(f"{rel}: missing drill make target reference (ops-drill-*)")
        listed_alerts = sorted(set(a for a in re.findall(r"`([A-Za-z][A-Za-z0-9]+)`", text) if a in alert_names))
        if not listed_alerts:
            errors.append(f"{rel}: Alerts section must list at least one known alert id")

    map_doc = (ctx.repo_root / "docs/operations/observability/runbook-dashboard-alert-map.md").read_text(encoding="utf-8")
    for alert in sorted(alert_names):
        if alert not in map_doc:
            errors.append(f"runbook-dashboard-alert-map: missing alert `{alert}`")
    for path in sorted(runbook_dir.glob("*.md")):
        if path.name == "INDEX.md":
            continue
        if path.name not in map_doc:
            errors.append(f"runbook-dashboard-alert-map: missing runbook row for `{path.name}`")
    return (0, "") if not errors else (1, "\n".join(errors[:200]))


def _check_ops_readmes_make_only(ctx: RunContext) -> tuple[int, str]:
    script_cmd = re.compile(r"^\s*(\./ops/|bash\s+ops/|sh\s+ops/|python3\s+ops/)")
    make_cmd = re.compile(r"\bmake\s+[a-zA-Z0-9_.-]+")
    errors: list[str] = []
    for md in sorted((ctx.repo_root / "ops").rglob("README.md")):
        text = md.read_text(encoding="utf-8", errors="ignore")
        if not make_cmd.search(text):
            errors.append(f"{md.relative_to(ctx.repo_root)}: missing make target instruction")
        for line_no, line in enumerate(text.splitlines(), start=1):
            if script_cmd.search(line):
                errors.append(f"{md.relative_to(ctx.repo_root)}:{line_no}: raw script run instruction found")
    return (0, "") if not errors else (1, "\n".join(errors[:200]))


def _check_ops_readme_canonical_links(ctx: RunContext) -> tuple[int, str]:
    errors: list[str] = []
    for md in sorted((ctx.repo_root / "ops").rglob("README.md")):
        rel = md.relative_to(ctx.repo_root).as_posix()
        if rel == "ops/README.md":
            continue
        text = md.read_text(encoding="utf-8", errors="ignore")
        if "ops/README.md" not in text and "docs/operations/INDEX.md" not in text:
            errors.append(f"{rel}: missing canonical link to ops/README.md or docs/operations/INDEX.md")
    return (0, "") if not errors else (1, "\n".join(errors[:200]))


def _check_ops_doc_duplication(ctx: RunContext) -> tuple[int, str]:
    ops_docs = ctx.repo_root / "docs" / "operations"
    headings: dict[str, list[str]] = {}
    blocks: dict[str, list[str]] = {}
    common_headings = {
        "what",
        "why",
        "scope",
        "non-goals",
        "contracts",
        "failure modes",
        "how to verify",
        "see also",
        "commands",
        "symptoms",
        "metrics",
        "expected outputs",
        "mitigations",
        "alerts",
        "rollback",
        "postmortem checklist",
        "dashboards",
        "drills",
    }
    for md in sorted(ops_docs.rglob("*.md")):
        rel = md.relative_to(ctx.repo_root).as_posix()
        text = md.read_text(encoding="utf-8", errors="ignore")
        for heading in re.findall(r"^##\s+(.+)$", text, flags=re.MULTILINE):
            headings.setdefault(heading.strip().lower(), []).append(rel)
        for para in re.split(r"\n\s*\n", text):
            normalized = "\n".join(line.strip() for line in para.splitlines() if line.strip())
            if len(normalized) < 220:
                continue
            key = hashlib.sha256(normalized.encode("utf-8")).hexdigest()
            blocks.setdefault(key, []).append(rel)
    errors: list[str] = []
    for heading, files in headings.items():
        if heading in common_headings:
            continue
        if len(set(files)) > 6:
            errors.append(f"heading appears excessively ({len(set(files))} files): '{heading}'")
    for files in blocks.values():
        if len(set(files)) > 1:
            errors.append(f"duplicated long content block across docs: {', '.join(sorted(set(files)))}")
    return (0, "") if not errors else (1, "\n".join(errors[:200]))


def _check_docs_make_only_ops(ctx: RunContext) -> tuple[int, str]:
    docs_root = ctx.repo_root / "docs"
    patterns = [
        re.compile(r"\./ops/[\w./-]+\.sh"),
        re.compile(r"\bops/[\w./-]+run_all\.sh\b"),
        re.compile(r"\bops/[\w./-]+scripts/[\w./-]+\.sh\b"),
    ]
    errors: list[str] = []
    for md in sorted(docs_root.rglob("*.md")):
        text = md.read_text(encoding="utf-8", errors="ignore")
        for line_no, line in enumerate(text.splitlines(), start=1):
            for pattern in patterns:
                if pattern.search(line):
                    errors.append(f"{md.relative_to(ctx.repo_root)}:{line_no}: raw ops script reference found")
                    break
    return (0, "") if not errors else (1, "\n".join(errors[:200]))


def _generate_sli_doc(ctx: RunContext) -> tuple[int, str]:
    payload = _read_json(ctx.repo_root / "configs/ops/slo/slis.v1.json")
    slis = payload.get("slis", [])
    lines = [
        "# SLIs (v1)",
        "",
        "- Generated from `configs/ops/slo/slis.v1.json`.",
        "",
        "| SLI | Goal | Primary Metric | Secondary Metric | Status |",
        "|---|---|---|---|---|",
    ]
    if isinstance(slis, list):
        for sli in slis:
            if not isinstance(sli, dict):
                continue
            secondary = f"`{sli.get('secondary_metric')}`" if sli.get("secondary_metric") else "-"
            lines.append(
                "| {name} | {goal} | `{metric}` | {secondary} | `{status}` |".format(
                    name=sli.get("name", ""),
                    goal=sli.get("goal", ""),
                    metric=sli.get("metric", ""),
                    secondary=secondary,
                    status=sli.get("status", "unknown"),
                )
            )
    lines.extend(
        [
            "",
            "## Endpoint Class Mapping",
            "",
            "- `cheap`: `^/health$`, `^/version$`, `^/metrics$`",
            "- `standard`: `^/v1/genes$`, `^/v1/genes/[^/]+$`",
            "- `heavy`: `^/v1/genes/[^/]+/(diff|region|sequence)$`",
            "",
        ]
    )
    out = ctx.repo_root / "docs/operations/slo/SLIS.md"
    out.write_text("\n".join(lines), encoding="utf-8")
    return 0, f"generated {out.relative_to(ctx.repo_root)}"


def _generate_slos_doc(ctx: RunContext) -> tuple[int, str]:
    payload = _read_json(ctx.repo_root / "configs/ops/slo/slo.v1.json")
    slis = {item.get("id"): item for item in payload.get("slis", []) if isinstance(item, dict)}
    lines = [
        "# SLO Targets (v1)",
        "",
        "- Generated from `configs/ops/slo/slo.v1.json`.",
        "",
        "| SLO ID | SLI | Target | Window | Threshold |",
        "|---|---|---:|---|---|",
    ]
    for slo in payload.get("slos", []):
        if not isinstance(slo, dict):
            continue
        sli_id = slo.get("sli", "")
        sli_name = slis.get(sli_id, {}).get("name", sli_id) if isinstance(sli_id, str) else sli_id
        threshold = "-"
        th = slo.get("threshold")
        if isinstance(th, dict):
            threshold = f"`{th.get('operator')} {th.get('value')} {th.get('unit')}`"
        lines.append(
            f"| `{slo.get('id','')}` | `{sli_name}` | `{slo.get('target','')}` | `{slo.get('window','')}` | {threshold} |"
        )
    lines.extend(
        [
            "",
            "## v1 Pragmatic Targets",
            "",
            "- `/readyz` availability: `99.9%` over `30d`.",
            "- Success: cheap `99.99%`, standard `99.9%`, heavy `99.0%` over `30d`.",
            "- Latency p99 thresholds: cheap `< 50ms`, standard `< 300ms`, heavy `< 2s`.",
            "- Overload cheap survival: `> 99.99%`.",
            "- Shed policy: heavy shedding tolerated; standard shedding bounded.",
            "- Registry freshness: refresh age `< 10m` for `99%` of windows.",
            "- Store objective: p95 latency bounded and error rate `< 0.5%`.",
            "",
        ]
    )
    out = ctx.repo_root / "docs/operations/slo/SLOS.md"
    out.write_text("\n".join(lines), encoding="utf-8")
    return 0, f"generated {out.relative_to(ctx.repo_root)}"


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
        if check.fn is not None:
            code, output = check.fn(ctx)
        elif check.cmd is not None:
            code, output = runner(check.cmd, ctx.repo_root)
        else:
            code, output = 2, "invalid docs check configuration"
        row: dict[str, object] = {
            "id": check.check_id,
            "description": check.description,
            "status": "pass" if code == 0 else "fail",
            "command": " ".join(check.cmd) if check.cmd else "native",
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
                None,
                "Regenerate/align docs public surface JSON and docs references.",
                fn=_check_public_surface_docs,
            ),
            _check(
                "docs-no-internal-target-refs",
                "Validate no internal make target refs",
                None,
                "Replace internal make targets with public targets in docs.",
                fn=_check_docs_make_only,
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
                None,
                "Regenerate docs outputs and commit deterministic updates.",
                fn=_check_docs_freeze_drift,
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
        code, output = _check_public_surface_docs(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        else:
            print("docs public surface check passed")
        return code

    if ns.docs_cmd == "no-internal-target-refs":
        code, output = _check_docs_make_only(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        else:
            print("docs make-only check passed")
        return code

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
        code, output = _check_docs_freeze_drift(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        else:
            print("docs freeze check passed")
        return code

    if ns.docs_cmd == "glossary-check":
        return _run_simple(ctx, ["python3", "scripts/areas/docs/lint_glossary_links.py"], ns.report)

    if ns.docs_cmd == "openapi-examples-check":
        code, output = _check_openapi_examples(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        else:
            print("openapi examples check passed")
        return code

    if ns.docs_cmd == "observability-surface-check":
        code, output = _check_observability_surface_drift(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        else:
            print("observability surface drift check passed")
        return code

    if ns.docs_cmd == "runbooks-contract-check":
        code, output = _check_runbooks_contract(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        else:
            print("runbook contract check passed")
        return code

    if ns.docs_cmd == "ops-readmes-make-only-check":
        code, output = _check_ops_readmes_make_only(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        else:
            print("ops README make-only contract passed")
        return code

    if ns.docs_cmd == "ops-readme-canonical-links-check":
        code, output = _check_ops_readme_canonical_links(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        else:
            print("ops README canonical-link check passed")
        return code

    if ns.docs_cmd == "ops-doc-duplication-check":
        code, output = _check_ops_doc_duplication(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        else:
            print("ops docs duplication check passed")
        return code

    if ns.docs_cmd == "docs-make-only-ops-check":
        code, output = _check_docs_make_only_ops(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        else:
            print("docs make-only ops entrypoint check passed")
        return code

    if ns.docs_cmd == "generate-sli-doc":
        code, output = _generate_sli_doc(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        return code

    if ns.docs_cmd == "generate-slos-doc":
        code, output = _generate_slos_doc(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        return code

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

    if ns.docs_cmd == "spellcheck":
        return _run_simple(ctx, ["python3", "scripts/areas/docs/spellcheck_docs.py", ns.path], ns.report)

    if ns.docs_cmd == "style":
        return _run_simple(ctx, ["python3", "scripts/areas/docs/lint_doc_status.py"], ns.report)

    if ns.docs_cmd == "rewrite-legacy-terms":
        if not ns.apply:
            payload = {
                "schema_version": 1,
                "tool": "atlasctl",
                "status": "pass",
                "note": "no changes applied; pass --apply to rewrite terms",
            }
            print(json.dumps(payload, sort_keys=True) if ns.report == "json" else payload["note"])
            return 0
        return _run_simple(ctx, ["python3", "scripts/areas/docs/rewrite_legacy_terms.py", ns.path], ns.report)

    if ns.docs_cmd == "generate":
        errors: list[str] = []
        for cmd in DOCS_GENERATE_COMMANDS:
            code, output = _run_check(cmd, ctx.repo_root)
            if code != 0:
                errors.append(f"{' '.join(cmd)} -> {output}")
                if ns.fail_fast:
                    break
        payload = {
            "schema_version": 1,
            "tool": "atlasctl",
            "status": "pass" if not errors else "fail",
            "generated_count": len(DOCS_GENERATE_COMMANDS) - len(errors),
            "errors": errors,
        }
        print(json.dumps(payload, sort_keys=True) if ns.report == "json" else json.dumps(payload, indent=2, sort_keys=True))
        return 0 if not errors else 1

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
        ("openapi-examples-check", "validate OpenAPI examples against declared schemas"),
        ("observability-surface-check", "validate observability surface generated docs are in sync"),
        ("runbooks-contract-check", "validate runbook content contract"),
        ("ops-readmes-make-only-check", "validate ops README files use make-only instructions"),
        ("ops-readme-canonical-links-check", "validate canonical links in ops README files"),
        ("ops-doc-duplication-check", "detect duplicate long sections in operations docs"),
        ("docs-make-only-ops-check", "forbid raw ops script references in docs"),
        ("generate-sli-doc", "generate docs/operations/slo/SLIS.md from SLI contract"),
        ("generate-slos-doc", "generate docs/operations/slo/SLOS.md from SLO contract"),
        ("glossary-check", "validate glossary and banned terms policy"),
        ("contracts-index", "validate or generate docs contracts index"),
        ("runbook-map", "validate or generate docs runbook map index"),
        ("evidence-policy-page", "generate docs evidence policy page"),
        ("inventory", "generate docs command inventory page"),
        ("extract-code", "extract code blocks from docs"),
        ("render-diagrams", "render docs diagrams"),
        ("lint-spelling", "run docs spelling checks"),
        ("spellcheck", "run docs spelling checks"),
        ("style", "run docs style checks"),
        ("generate", "run docs generators"),
        ("rewrite-legacy-terms", "explicit legacy-term rewrite command"),
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
        if name == "spellcheck":
            cmd.add_argument("--path", default="docs")
        if name == "rewrite-legacy-terms":
            cmd.add_argument("--path", default="docs")
            cmd.add_argument("--apply", action="store_true")
        if name == "generate":
            cmd.add_argument("--fail-fast", action="store_true")
