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
        None,
        "Ensure each docs directory has an index page where required.",
        fn=lambda ctx: _check_index_pages(ctx),
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
        None,
        "Add nav links or remove orphaned docs pages.",
        fn=lambda ctx: _check_no_orphan_docs(ctx),
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


def _tracked_files(repo_root: Path, patterns: list[str] | None = None) -> list[str]:
    cmd = ["git", "ls-files"]
    if patterns:
        cmd.extend(patterns)
    proc = subprocess.run(cmd, cwd=repo_root, text=True, capture_output=True, check=False)
    if proc.returncode != 0:
        return []
    return [line.strip() for line in proc.stdout.splitlines() if line.strip()]


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


def _check_durable_naming(ctx: RunContext) -> tuple[int, str]:
    forbidden_prefix_re = re.compile(
        r"^(phase|task|stage|round|iteration|tmp|placeholder|vnext)([-_0-9]|$)",
        re.IGNORECASE,
    )
    kebab_script = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*\.(sh|py)$")
    kebab_doc = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*\.md$")
    adr_doc = re.compile(r"^ADR-\d{4}-[a-z0-9-]+\.md$")
    scream_doc = re.compile(r"^[A-Z0-9_]+\.md$")
    doc_exact_exceptions = {
        "docs/STYLE.md",
        "docs/contracts/README.md",
        "docs/api/compatibility.md",
        "docs/api/deprecation.md",
        "docs/api/v1-surface.md",
    }
    doc_name_exceptions = {"INDEX.md", "CONCEPT_REGISTRY.md", "DEPTH_POLICY.md", "DEPTH_RUBRIC.md"}
    files = _tracked_files(ctx.repo_root)
    errors: list[str] = []

    case_map: dict[str, list[str]] = {}
    for path in files:
        case_map.setdefault(path.lower(), []).append(path)

    for variants in case_map.values():
        uniq = sorted(set(variants))
        if len(uniq) > 1:
            errors.append(f"case-collision path variants: {uniq}")

    for path in files:
        name = Path(path).name
        stem = Path(path).stem
        if not path.startswith("docs/_drafts/") and forbidden_prefix_re.search(stem):
            errors.append(f"forbidden temporal/task token in name: {path}")
        if path.startswith("docs/") and path.endswith(".md"):
            if (
                path not in doc_exact_exceptions
                and name not in doc_name_exceptions
                and not kebab_doc.match(name)
                and not adr_doc.match(name)
                and not (path.startswith("docs/_generated/contracts/") and scream_doc.match(name))
            ):
                errors.append(f"docs markdown must use kebab-case or approved canonical exception: {path}")
        if path.startswith("scripts/areas/public/") and (path.endswith(".sh") or path.endswith(".py")):
            if not kebab_script.match(name):
                errors.append(f"public scripts must use kebab-case: {path}")
    return (0, "durable naming check passed") if not errors else (1, "\n".join(errors))


def _check_duplicate_topics(ctx: RunContext) -> tuple[int, str]:
    required = [
        "docs/architecture/boundaries.md",
        "docs/architecture/effects.md",
        "docs/architecture/boundary-maps.md",
        "docs/product/immutability-and-aliases.md",
        "docs/contracts/compatibility.md",
        "docs/contracts/plugin/spec.md",
        "docs/contracts/plugin/mode.md",
        "docs/_lint/duplicate-topics.md",
    ]
    pointer_files = [
        "docs/reference/store/immutability-guarantee.md",
        "docs/reference/registry/latest-release-alias-policy.md",
        "docs/reference/compatibility/umbrella-atlas-matrix.md",
        "docs/reference/compatibility/bijux-dna-atlas.md",
    ]
    owner_header_files = [
        "docs/architecture/boundaries.md",
        "docs/architecture/effects.md",
        "docs/architecture/boundary-maps.md",
        "docs/product/immutability-and-aliases.md",
        "docs/contracts/compatibility.md",
        "docs/operations/k8s/INDEX.md",
        "docs/operations/runbooks/INDEX.md",
    ]
    errors: list[str] = []
    for rel in required:
        if not (ctx.repo_root / rel).exists():
            errors.append(f"missing canonical file {rel}")
    for rel in pointer_files:
        path = ctx.repo_root / rel
        if "Canonical page:" not in path.read_text(encoding="utf-8", errors="ignore"):
            errors.append(f"{rel} must be a canonical pointer")
    for rel in owner_header_files:
        text = (ctx.repo_root / rel).read_text(encoding="utf-8", errors="ignore")
        if not re.search(r"^- Owner:", text, re.MULTILINE):
            errors.append(f"missing Owner header in {rel}")
    return (0, "duplicate-topics check passed") if not errors else (1, "\n".join(errors))


def _generate_naming_inventory(ctx: RunContext) -> tuple[int, str]:
    out = ctx.repo_root / "docs" / "_generated" / "naming-inventory.md"
    suites = json.loads((ctx.repo_root / "ops" / "load" / "suites" / "suites.json").read_text(encoding="utf-8")).get("suites", [])
    runbooks_dir = ctx.repo_root / "docs" / "operations" / "runbooks"
    runbook_map = ctx.repo_root / "docs" / "operations" / "observability" / "runbook-dashboard-alert-map.md"
    files = _tracked_files(ctx.repo_root)
    forbidden_tokens = ("phase", "task", "stage", "round", "iteration", "tmp", "placeholder")
    doc_re = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*\.md$")
    script_re = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*\.(sh|py)$")
    docs = [p for p in files if p.startswith("docs/") and p.endswith(".md")]
    scripts = [p for p in files if p.startswith("scripts/") and p.endswith((".sh", ".py"))]
    rust_tests = [p for p in files if p.endswith(".rs") and "/tests/" in p]
    runbooks = sorted(p.name for p in runbooks_dir.glob("*.md") if p.name != "INDEX.md")
    runbook_rows = sum(1 for line in runbook_map.read_text(encoding="utf-8").splitlines() if line.startswith("| `") and line.endswith("|"))
    doc_non_kebab = [
        p
        for p in docs
        if Path(p).name not in {"INDEX.md", "STYLE.md", "README.md", "compatibility.md", "deprecation.md", "v1-surface.md"}
        and not p.startswith("docs/_generated/contracts/")
        and not doc_re.match(Path(p).name)
    ]
    script_non_kebab = [p for p in scripts if not script_re.match(Path(p).name)]
    forbidden = [p for p in files if any(token in Path(p).stem.lower() for token in forbidden_tokens)]
    lines = [
        "# Naming Inventory",
        "",
        "- Owner: `docs-governance`",
        "- Generated by: `atlasctl docs naming-inventory`",
        "",
        "## Summary",
        "",
        f"- Tracked files: `{len(files)}`",
        f"- Docs markdown files: `{len(docs)}`",
        f"- Script files under `scripts/`: `{len(scripts)}`",
        f"- Rust test files: `{len(rust_tests)}`",
        f"- Load suites in `ops/load/suites/suites.json`: `{len(suites)}`",
        f"- Runbooks in `docs/operations/runbooks/`: `{len(runbooks)}`",
        f"- Runbook map rows: `{runbook_rows}`",
        "",
        "## Naming Health",
        "",
        f"- Forbidden-token hits: `{len(forbidden)}`",
        f"- Non-kebab docs outside allowed exceptions: `{len(doc_non_kebab)}`",
        f"- Non-kebab scripts under `scripts/`: `{len(script_non_kebab)}`",
        "",
        "## Load Suites",
        "",
    ]
    for suite in sorted(suites, key=lambda item: item.get("name", "")):
        lines.append(f"- `{suite.get('name','')}`")
    lines.extend(["", "## Runbooks", ""])
    for name in runbooks:
        lines.append(f"- `{name}`")
    lines.extend(["", "## Violations", ""])
    if not forbidden and not doc_non_kebab and not script_non_kebab:
        lines.append("None.")
    else:
        for entry in sorted(set(forbidden + doc_non_kebab + script_non_kebab)):
            lines.append(f"- `{entry}`")
    lines.append("")
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(lines), encoding="utf-8")
    return 0, str(out.relative_to(ctx.repo_root))


def _check_legacy_terms(ctx: RunContext) -> tuple[int, str]:
    allowlist = ctx.repo_root / "scripts/areas/docs/legacy-terms-allowlist.txt"
    if not allowlist.exists():
        return 2, f"missing allowlist: {allowlist.relative_to(ctx.repo_root)}"
    allow = [line.strip() for line in allowlist.read_text(encoding="utf-8").splitlines() if line.strip() and not line.startswith("#")]
    patterns = [
        r"\bphase\s+[0-9ivx]+\b",
        r"\bphase\s+stability\b",
        r"\bphase\s+contract\b",
        r"\b(step|task|stage|iteration|round)\s+[0-9ivx]+\b",
        r"\bvnext\s+placeholder\b",
        r"\btemporary\b",
        r"\bwip\b",
    ]
    checks = [ctx.repo_root / "docs", ctx.repo_root / "ops", ctx.repo_root / "crates"]
    violations: list[str] = []
    for root in checks:
        for path in root.rglob("*.md"):
            rel = path.relative_to(ctx.repo_root).as_posix()
            if rel.startswith("docs/_drafts/") or rel.startswith("docs/_generated/"):
                continue
            text = path.read_text(encoding="utf-8", errors="ignore")
            for idx, line in enumerate(text.splitlines(), start=1):
                low = line.lower()
                if any(re.search(pat, low, re.IGNORECASE) for pat in patterns):
                    if any(a.lower() in low for a in allow):
                        continue
                    violations.append(f"{rel}:{idx}:{line.strip()}")
    return (0, "legacy language gate passed") if not violations else (1, "\n".join(violations))


def _check_observability_docs_checklist(ctx: RunContext) -> tuple[int, str]:
    checklist = ctx.repo_root / "docs/_lint/observability-docs.md"
    obs_dir = ctx.repo_root / "docs/operations/observability"
    required_pages = {
        "INDEX.md",
        "acceptance-gates.md",
        "alerts.md",
        "dashboard.md",
        "profiles.md",
        "slo.md",
        "tracing.md",
        "compatibility.md",
    }
    required_headings = ["## What", "## Why", "## Contracts", "## Failure modes", "## How to verify"]
    errors: list[str] = []
    if not checklist.exists():
        errors.append("missing docs/_lint/observability-docs.md")
    else:
        text = checklist.read_text(encoding="utf-8")
        for page in sorted(required_pages):
            needle = f"- [x] `{page}`"
            if needle not in text:
                errors.append(f"checklist missing completed item: {needle}")
    for page in sorted(required_pages):
        path = obs_dir / page
        if not path.exists():
            errors.append(f"missing observability page: {path.relative_to(ctx.repo_root)}")
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        for heading in required_headings:
            if heading not in text:
                errors.append(f"{path.relative_to(ctx.repo_root)} missing heading: {heading}")
    alerts = (obs_dir / "alerts.md").read_text(encoding="utf-8", errors="ignore") if (obs_dir / "alerts.md").exists() else ""
    if "## Run drills" not in alerts:
        errors.append("docs/operations/observability/alerts.md missing heading: ## Run drills")
    return (0, "observability docs checklist passed") if not errors else (1, "\n".join(errors))


def _check_no_orphan_docs(ctx: RunContext) -> tuple[int, str]:
    docs = ctx.repo_root / "docs"
    mkdocs = (ctx.repo_root / "mkdocs.yml").read_text(encoding="utf-8")
    nav_refs = set(re.findall(r":\s+([A-Za-z0-9_./\\-]+\.md)\s*$", mkdocs, re.MULTILINE))
    index_refs: set[str] = set()
    index_dirs: set[str] = set()
    for idx in docs.rglob("INDEX.md"):
        index_dirs.add(str(idx.parent.relative_to(docs)))
        txt = idx.read_text(encoding="utf-8", errors="ignore")
        for link in re.findall(r"\[[^\]]+\]\(([^)]+\.md)(?:#[^)]+)?\)", txt):
            p = (idx.parent / link).resolve()
            if p.exists() and docs in p.parents:
                index_refs.add(str(p.relative_to(docs)))
    allow_prefixes = ("_generated/", "_drafts/")
    errors: list[str] = []
    for md in docs.rglob("*.md"):
        rel = str(md.relative_to(docs))
        if rel.endswith("INDEX.md") or any(rel.startswith(p) for p in allow_prefixes):
            continue
        parent = str(md.parent.relative_to(docs))
        if rel not in nav_refs and rel not in index_refs and parent not in index_dirs:
            errors.append(rel)
    return (0, "no orphan docs check passed") if not errors else (1, "\n".join(sorted(errors)))


def _check_script_locations(ctx: RunContext) -> tuple[int, str]:
    files = _tracked_files(ctx.repo_root, patterns=["*.sh", "*.py"])
    allowed_ops_markers = ("/scripts/", "/tests/", "/ci/", "/_lib/", "/run/", "/_lint/", "/runner/")
    allowed_ops_prefixes = (
        "ops/_meta/",
        "ops/e2e/realdata/",
        "ops/load/reports/",
        "ops/stack/kind/",
        "ops/stack/minio/",
        "ops/stack/registry/",
        "ops/stack/toxiproxy/",
        "ops/stack/faults/",
        "ops/report/",
        "ops/e2e/smoke/",
        "ops/stack/scripts/",
    )
    errors: list[str] = []
    for rel in files:
        if rel.startswith("scripts/") or rel.startswith("docker/scripts/") or rel.startswith("packages/bijux-atlas-scripts/"):
            continue
        if rel.startswith("ops/"):
            if any(marker in rel for marker in allowed_ops_markers) or any(rel.startswith(prefix) for prefix in allowed_ops_prefixes):
                continue
            errors.append(f"{rel}: ops script path is outside approved automation zones")
            continue
        errors.append(f"{rel}: scripts must live under scripts/, ops/, or packages/bijux-atlas-scripts/")
    return (0, "script location check passed") if not errors else (1, "\n".join(errors))


def _check_runbook_map_registration(ctx: RunContext) -> tuple[int, str]:
    runbook_dir = ctx.repo_root / "docs/operations/runbooks"
    runbook_map = ctx.repo_root / "docs/operations/observability/runbook-dashboard-alert-map.md"
    runbooks = sorted(p.name for p in runbook_dir.glob("*.md") if p.name != "INDEX.md")
    mapped = set(re.findall(r"\|\s*`([^`]+\.md)`\s*\|", runbook_map.read_text(encoding="utf-8", errors="ignore")))
    missing = [name for name in runbooks if name not in mapped]
    if missing:
        return 1, "\n".join(f"{name} missing from {runbook_map.relative_to(ctx.repo_root)}" for name in missing)
    return 0, "runbook map registration check passed"


def _check_contract_doc_pairs(ctx: RunContext) -> tuple[int, str]:
    contracts_dir = ctx.repo_root / "docs/contracts"
    gen_dir = ctx.repo_root / "docs/_generated/contracts"
    handwritten_map = {
        "ERROR_CODES.json": "errors.md",
        "METRICS.json": "metrics.md",
        "TRACE_SPANS.json": "tracing.md",
        "ENDPOINTS.json": "endpoints.md",
        "CONFIG_KEYS.json": "config-keys.md",
        "CHART_VALUES.json": "chart-values.md",
    }
    errors: list[str] = []
    for json_path in sorted(contracts_dir.glob("*.json")):
        generated = gen_dir / f"{json_path.stem}.md"
        hand = contracts_dir / handwritten_map.get(json_path.name, "")
        if generated.exists() or (hand.name and hand.exists()):
            continue
        errors.append(
            f"{json_path.relative_to(ctx.repo_root)} has no matching doc; expected {generated.relative_to(ctx.repo_root)} or mapped docs/contracts/*.md"
        )
    return (0, "contract doc pair check passed") if not errors else (1, "\n".join(errors))


def _check_index_pages(ctx: RunContext) -> tuple[int, str]:
    docs = ctx.repo_root / "docs"
    required_sections = [
        "## What",
        "## Why",
        "## Scope",
        "## Non-goals",
        "## Contracts",
        "## Failure modes",
        "## How to verify",
        "## See also",
    ]
    errors: list[str] = []
    for directory in sorted(p for p in docs.rglob("*") if p.is_dir()):
        rel = directory.relative_to(ctx.repo_root).as_posix()
        if rel == "docs" or rel.startswith("docs/_assets") or rel.startswith("docs/_"):
            continue
        md_files = list(directory.glob("*.md"))
        if not md_files:
            continue
        index = directory / "INDEX.md"
        if not index.exists():
            errors.append(f"missing INDEX.md in {rel}")
            continue
        text = index.read_text(encoding="utf-8", errors="ignore")
        for sec in required_sections:
            if sec not in text:
                errors.append(f"{index.relative_to(ctx.repo_root)} missing section: {sec}")
    return (0, "index pages check passed") if not errors else (1, "\n".join(errors))


def _check_script_headers(ctx: RunContext) -> tuple[int, str]:
    root = ctx.repo_root
    script_paths = sorted(
        p
        for p in (root / "scripts").rglob("*")
        if p.is_file()
        and p.suffix in {".sh", ".py"}
        and (
            p.relative_to(root).as_posix().startswith("scripts/areas/public/")
            or p.relative_to(root).as_posix().startswith("scripts/bin/")
        )
    )
    errors: list[str] = []
    for path in script_paths:
        if "/scripts/areas/_internal/" in path.as_posix():
            continue
        txt = path.read_text(encoding="utf-8", errors="ignore").splitlines()
        first = txt[0] if txt else ""
        rel = path.relative_to(root).as_posix()
        is_public = rel.startswith("scripts/areas/public/")
        is_executable = path.stat().st_mode & 0o111 != 0
        has_shebang = first.startswith("#!")
        if not (is_public or is_executable or has_shebang):
            continue
        head = "\n".join(txt[:12])
        if path.suffix == ".sh" and not (
            head.startswith("#!/usr/bin/env sh")
            or head.startswith("#!/bin/sh")
            or head.startswith("#!/usr/bin/env bash")
            or head.startswith("#!/bin/bash")
            or head.startswith("#!/usr/bin/env python3")
        ):
            errors.append(f"{rel}: missing shebang")
        if path.suffix == ".py" and not head.startswith("#!/usr/bin/env python3"):
            errors.append(f"{rel}: missing shebang")
        legacy_header = "Purpose:" in head and "Inputs:" in head and "Outputs:" in head
        modern_header = all(token in head.lower() for token in ("owner:", "purpose:", "stability:", "called-by:"))
        if not (legacy_header or modern_header):
            errors.append(f"{rel}: missing script header contract")
        if rel.startswith("scripts/areas/public/"):
            required = ("owner:", "purpose:", "stability:", "called-by:")
            missing = [k for k in required if k not in head.lower()]
            if missing:
                errors.append(f"{rel}: missing public header fields ({', '.join(missing)})")
    idx = root / "docs/development/scripts/INDEX.md"
    if idx.exists():
        text = idx.read_text(encoding="utf-8", errors="ignore")
        required_groups = [
            "scripts/areas/docs/",
            "scripts/areas/public/perf/",
            "scripts/areas/public/observability/",
            "scripts/areas/fixtures/",
            "scripts/areas/release/",
            "scripts/areas/layout/",
            "scripts/bin/",
            "scripts/areas/public/",
            "scripts/areas/internal/",
            "scripts/areas/tools/",
        ]
        for group in required_groups:
            if group not in text:
                errors.append(f"{idx.relative_to(root)}: missing script group reference `{group}`")
    else:
        errors.append(f"{idx.relative_to(root)}: missing scripts index")
    return (0, "script header check passed") if not errors else (1, "\n".join(errors))


def _generate_architecture_map(ctx: RunContext) -> tuple[int, str]:
    category_hints = {
        "bijux-atlas-api": "api-surface",
        "bijux-atlas-server": "runtime-server",
        "bijux-atlas-query": "query-engine",
        "bijux-atlas-store": "artifact-store",
        "bijux-atlas-ingest": "ingest-pipeline",
        "bijux-atlas-cli": "cli-ops",
        "bijux-atlas-model": "shared-model",
        "bijux-atlas-core": "shared-core",
        "bijux-atlas-policies": "policy-contracts",
    }
    code, out = _run_check(
        ["cargo", "metadata", "--locked", "--format-version", "1", "--no-deps"],
        ctx.repo_root,
    )
    if code != 0:
        return 1, out
    meta = json.loads(out)
    packages = {
        p.get("name"): p
        for p in meta.get("packages", [])
        if isinstance(p, dict) and isinstance(p.get("name"), str) and p["name"].startswith("bijux-atlas-")
    }
    names = sorted(packages.keys())
    lines = [
        "# Architecture Map",
        "",
        "- Owner: `atlas-platform`",
        "- Stability: `stable`",
        "",
        "Generated crate-level architecture map from workspace metadata.",
        "",
        "## Crate Nodes",
        "",
        "| Crate | Role | Internal Dependencies |",
        "| --- | --- | --- |",
    ]
    for name in names:
        pkg = packages[name]
        deps = sorted(
            d.get("name")
            for d in pkg.get("dependencies", [])
            if isinstance(d, dict) and isinstance(d.get("name"), str) and d["name"].startswith("bijux-atlas-")
        )
        dep_str = ", ".join(f"`{d}`" for d in deps) if deps else "`(none)`"
        role = category_hints.get(name, "unspecified")
        lines.append(f"| `{name}` | `{role}` | {dep_str} |")
    lines += [
        "",
        "## Runtime Direction",
        "",
        "`bijux-atlas-server -> bijux-atlas-query -> bijux-atlas-store -> immutable artifacts`",
        "",
        "## Notes",
        "",
        "- This file is generated; do not hand-edit.",
        "- Regenerate via `atlasctl docs generate-architecture-map`.",
        "",
    ]
    out_path = ctx.repo_root / "docs/architecture/architecture-map.md"
    out_path.write_text("\n".join(lines), encoding="utf-8")
    return 0, f"generated {out_path.relative_to(ctx.repo_root)}"


def _generate_upgrade_guide(ctx: RunContext) -> tuple[int, str]:
    payload = _read_json(ctx.repo_root / "configs/ops/target-renames.json")
    rows = payload.get("renames", [])
    lines = [
        "# Make Target Upgrade Guide",
        "",
        "Use this table to migrate renamed or aliased make targets.",
        "",
        "| Old Target | New Target | Status |",
        "|---|---|---|",
    ]
    if isinstance(rows, list):
        for row in rows:
            if not isinstance(row, dict):
                continue
            lines.append(f"| `{row.get('from','')}` | `{row.get('to','')}` | `{row.get('status','')}` |")
    lines.append("")
    out = ctx.repo_root / "docs/_generated/upgrade-guide.md"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(lines), encoding="utf-8")
    return 0, str(out.relative_to(ctx.repo_root))


def _check_crate_docs_contract(ctx: RunContext) -> tuple[int, str]:
    crates_root = ctx.repo_root / "crates"
    if not crates_root.exists():
        return 1, "crates directory missing"
    crates = sorted([p for p in crates_root.iterdir() if p.is_dir()])
    required_docs = {"INDEX.md", "architecture.md", "effects.md", "public-api.md", "testing.md"}
    contracts_required = {"bijux-atlas-api", "bijux-atlas-server", "bijux-atlas-policies", "bijux-atlas-store"}
    failure_modes_required = {"bijux-atlas-server", "bijux-atlas-store", "bijux-atlas-ingest"}
    required_sections = ["## Purpose", "## Invariants", "## Boundaries", "## Failure modes", "## How to test"]
    placeholder_pat = re.compile(r"\b(TODO|TBD|coming soon)\b", re.IGNORECASE)
    pub_pat = re.compile(r"^\s*pub\s+(?:struct|enum|trait|type)\s+([A-Z][A-Za-z0-9_]*)\b", re.MULTILINE)
    errors: list[str] = []

    for crate in crates:
        name = crate.name
        docs = crate / "docs"
        readme = crate / "README.md"
        if not docs.is_dir():
            errors.append(f"{crate}: missing docs directory")
            continue
        files = {p.name for p in docs.glob("*.md")}
        for req in required_docs:
            if req not in files:
                errors.append(f"{crate}/docs: missing {req}")
        if name in contracts_required and "contracts.md" not in files:
            errors.append(f"{crate}/docs: missing contracts.md (required)")
        if name in failure_modes_required and "failure-modes.md" not in files:
            errors.append(f"{crate}/docs: missing failure-modes.md (required)")

        for forbidden in [
            "HUMAN_MACHINE.md",
            "PUBLIC_SURFACE_CHECKLIST.md",
            "EFFECT_BOUNDARY_MAP.md",
            "PUBLIC_API.md",
            "ARCHITECTURE.md",
            "EFFECTS.md",
        ]:
            if forbidden in files:
                errors.append(f"{crate}/docs: legacy filename forbidden: {forbidden}")

        if "patterns.md" in files:
            text = (docs / "patterns.md").read_text(encoding="utf-8")
            if len(text.strip()) < 120:
                errors.append(f"{crate}/docs/patterns.md: too small; remove or document real patterns")

        major = [docs / "testing.md"]
        if name in contracts_required:
            major.append(docs / "contracts.md")
        if name in failure_modes_required:
            major.append(docs / "failure-modes.md")
        for md in major:
            if not md.exists():
                continue
            txt = md.read_text(encoding="utf-8")
            if not re.search(r"^- Owner:\s*`[^`]+`\s*$", txt, re.MULTILINE):
                errors.append(f"{md}: missing owner header \"- Owner: `...`\"")
            for sec in required_sections:
                if sec not in txt:
                    errors.append(f"{md}: missing section {sec}")
            if md.name == "contracts.md" and "## Versioning" not in txt:
                errors.append(f"{md}: missing section ## Versioning")
            if (txt.count("```") // 2) < 2:
                errors.append(f"{md}: requires at least 2 examples")
            if placeholder_pat.search(txt):
                errors.append(f"{md}: contains placeholder marker TODO/TBD/coming soon")
            if re.search(r"\]\((?:https?://|file://|/)", txt):
                errors.append(f"{md}: contains non-relative internal link")

        if not readme.exists():
            errors.append(f"{crate}: missing README.md")
        else:
            rtxt = readme.read_text(encoding="utf-8")
            for sec in [
                "## Purpose",
                "## Public API",
                "## Boundaries",
                "## Effects",
                "## Telemetry",
                "## Tests",
                "## Benches",
                "## Docs index",
            ]:
                if sec not in rtxt:
                    errors.append(f"{readme}: missing section {sec}")
            for req_link in ["docs/INDEX.md", "docs/public-api.md"]:
                if req_link not in rtxt:
                    errors.append(f"{readme}: missing link {req_link}")
            docs_index_block = re.search(r"## Docs index\n([\s\S]*?)(?:\n## |\Z)", rtxt)
            if not docs_index_block:
                errors.append(f"{readme}: missing docs index block")
            else:
                links = re.findall(r"\[[^\]]+\]\([^\)]+\)", docs_index_block.group(1))
                if len(links) < 5:
                    errors.append(f"{readme}: docs index must list at least 5 important docs")

        idx = docs / "INDEX.md"
        if idx.exists():
            itxt = idx.read_text(encoding="utf-8")
            for req in ["public-api.md", "effects.md", "testing.md"]:
                if req not in itxt:
                    errors.append(f"{idx}: must link {req}")
            if "#how-to-extend" not in itxt and "How to extend" not in itxt:
                errors.append(f"{idx}: must provide How to extend linkage")

        lib = crate / "src" / "lib.rs"
        public_api = docs / "public-api.md"
        if lib.exists() and public_api.exists():
            names = sorted(set(pub_pat.findall(lib.read_text(encoding="utf-8"))))
            ptxt = public_api.read_text(encoding="utf-8")
            if (
                "../../../../docs/_style/stability-levels.md" not in ptxt
                and "../../../docs/_style/stability-levels.md" not in ptxt
            ):
                errors.append(f"{public_api}: missing stability reference link")
            for n in names:
                if n not in ptxt:
                    errors.append(f"{public_api}: missing mention of public type {n}")
    return (0, "crate docs contract OK") if not errors else (1, "\n".join(errors[:300]))


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

    if ns.docs_cmd == "generate-architecture-map":
        code, output = _generate_architecture_map(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        return code

    if ns.docs_cmd == "generate-upgrade-guide":
        code, output = _generate_upgrade_guide(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        return code

    if ns.docs_cmd == "crate-docs-contract-check":
        code, output = _check_crate_docs_contract(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        return code

    if ns.docs_cmd == "durable-naming-check":
        code, output = _check_durable_naming(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "duplicate-topics-check":
        code, output = _check_duplicate_topics(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "naming-inventory":
        code, output = _generate_naming_inventory(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "legacy-terms-check":
        code, output = _check_legacy_terms(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "observability-docs-checklist":
        code, output = _check_observability_docs_checklist(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "no-orphan-docs-check":
        code, output = _check_no_orphan_docs(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "script-locations-check":
        code, output = _check_script_locations(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "runbook-map-registration-check":
        code, output = _check_runbook_map_registration(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "contract-doc-pairs-check":
        code, output = _check_contract_doc_pairs(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "index-pages-check":
        code, output = _check_index_pages(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "script-headers-check":
        code, output = _check_script_headers(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "contracts-index":
        if ns.fix:
            return _run_simple(ctx, ["python3", "scripts/areas/docs/generate_contracts_index_doc.py"], ns.report)
        return _run_simple(ctx, ["python3", "scripts/areas/docs/check_contracts_index_nav.py"], ns.report)

    if ns.docs_cmd == "runbook-map":
        if ns.fix:
            return _run_simple(ctx, ["python3", "scripts/areas/docs/generate_runbook_map_index.py"], ns.report)
        code, output = _check_runbook_map_registration(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        else:
            print(output)
        return code

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
        ("generate-architecture-map", "generate docs/architecture/architecture-map.md"),
        ("generate-upgrade-guide", "generate docs/_generated/upgrade-guide.md"),
        ("crate-docs-contract-check", "validate per-crate docs contract"),
        ("durable-naming-check", "enforce durable naming rules across docs/scripts"),
        ("duplicate-topics-check", "enforce duplicate topics pointer and owner contract"),
        ("naming-inventory", "generate docs/_generated/naming-inventory.md"),
        ("legacy-terms-check", "forbid legacy planning/task wording in docs"),
        ("observability-docs-checklist", "validate observability docs checklist and page sections"),
        ("no-orphan-docs-check", "validate docs are discoverable by nav/index links"),
        ("script-locations-check", "validate script location policy"),
        ("runbook-map-registration-check", "validate runbook map has every runbook"),
        ("contract-doc-pairs-check", "validate JSON contracts have docs pairs"),
        ("index-pages-check", "validate docs/INDEX.md contract"),
        ("script-headers-check", "validate script header and docs script-group contract"),
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
