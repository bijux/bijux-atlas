from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import subprocess
from dataclasses import dataclass
from datetime import datetime, timezone
from html.parser import HTMLParser
from pathlib import Path
from typing import Callable

import yaml

from ...core.context import RunContext
from ...core.fs import ensure_evidence_path
from ...make.target_graph import parse_make_targets


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
        None,
        "Align terminology and units references with docs SSOT conventions.",
        fn=lambda ctx: _check_terminology_units_ssot(ctx),
    ),
    _check(
        "docs-status-lint",
        "Validate document status contract",
        None,
        "Fix missing/invalid status frontmatter values.",
        fn=lambda ctx: _lint_doc_status(ctx),
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
        None,
        "Normalize page titles to the required style.",
        fn=lambda ctx: _check_title_case(ctx),
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
    ["python3", "-m", "atlasctl.cli", "docs", "generate-crates-map", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-architecture-map", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-k8s-values-doc", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "concept-graph-generate", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-openapi-docs", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-observability-surface", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-ops-badge", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-ops-schema-docs", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-ops-surface", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-ops-contracts-doc", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-make-targets-catalog", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-config-keys-doc", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-env-vars-doc", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "contracts-index", "--fix", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-chart-contract-index", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-k8s-install-matrix", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-make-targets-inventory", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "generate-scripts-graph", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "ops", "k8s-surface-generate", "--report", "text"],
    ["python3", "-m", "atlasctl.cli", "docs", "runbook-map", "--fix", "--report", "text"],
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
        ["python3", "-m", "atlasctl.cli", "contracts", "generate", "--generators", "artifacts"],
        ["python3", "-m", "atlasctl.cli", "docs", "generate-chart-contract-index", "--report", "text"],
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
    allowlist = ctx.repo_root / "configs/docs/legacy-terms-allowlist.txt"
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
        if (
            rel.startswith("scripts/")
            or rel.startswith("docker/scripts/")
            or rel.startswith("packages/atlasctl/")
            or rel.startswith("packages/bijux-atlas-py/")
        ):
            continue
        if rel.startswith("ops/"):
            if any(marker in rel for marker in allowed_ops_markers) or any(rel.startswith(prefix) for prefix in allowed_ops_prefixes):
                continue
            errors.append(f"{rel}: ops script path is outside approved automation zones")
            continue
        errors.append(f"{rel}: scripts must live under scripts/, ops/, or packages/atlasctl/")
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


def _check_observability_acceptance_checklist(ctx: RunContext) -> tuple[int, str]:
    path = ctx.repo_root / "docs/operations/observability/acceptance-checklist.md"
    if not path.exists():
        return 1, f"missing checklist: {path.relative_to(ctx.repo_root)}"
    text = path.read_text(encoding="utf-8", errors="ignore")
    required = [
        "## Required Checks",
        "## Release Notes",
        "make telemetry-verify",
        "make observability-pack-drills",
    ]
    missing = [item for item in required if item not in text]
    if missing:
        return 1, "acceptance checklist missing required entries:\n" + "\n".join(f"- {item}" for item in missing)
    return 0, "observability acceptance checklist contract passed"


def _generate_contracts_index_doc(ctx: RunContext) -> tuple[int, str]:
    out = ctx.repo_root / "docs/_generated/contracts-index.md"
    scan_roots = [ctx.repo_root / "ops", ctx.repo_root / "configs", ctx.repo_root / "makefiles", ctx.repo_root / "docker"]
    contracts: list[Path] = []
    schemas: list[Path] = []
    for root in scan_roots:
        contracts.extend(root.glob("**/CONTRACT.md"))
        schemas.extend(root.glob("**/*.schema.json"))
    contracts = sorted(contracts)
    schemas = sorted(schemas)
    lines = [
        "# Contracts And Schemas Index (Generated)",
        "",
        "Generated from repository files. Do not edit manually.",
        "",
        f"- CONTRACT files: `{len(contracts)}`",
        f"- Schema files: `{len(schemas)}`",
        "",
        "## CONTRACT.md Files",
        "",
    ]
    lines.extend(f"- `{p.relative_to(ctx.repo_root).as_posix()}`" for p in contracts)
    lines += ["", "## Schema Files", ""]
    lines.extend(f"- `{p.relative_to(ctx.repo_root).as_posix()}`" for p in schemas)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0, str(out.relative_to(ctx.repo_root))


def _check_contracts_index_nav(ctx: RunContext) -> tuple[int, str]:
    mkdocs = (ctx.repo_root / "mkdocs.yml").read_text(encoding="utf-8", errors="ignore")
    index = ctx.repo_root / "docs/_generated/contracts-index.md"
    scan_roots = [ctx.repo_root / "ops", ctx.repo_root / "configs", ctx.repo_root / "makefiles", ctx.repo_root / "docker"]
    errors: list[str] = []
    if "_generated/contracts-index.md" not in mkdocs:
        errors.append("mkdocs.yml is missing nav entry for docs/_generated/contracts-index.md")
    if not index.exists():
        errors.append("missing docs/_generated/contracts-index.md; run docs generator")
    else:
        index_text = index.read_text(encoding="utf-8", errors="ignore")
        contracts: list[Path] = []
        for root in scan_roots:
            contracts.extend(root.glob("**/CONTRACT.md"))
        for path in sorted(contracts):
            rel = path.relative_to(ctx.repo_root).as_posix()
            if f"`{rel}`" not in index_text:
                errors.append(f"contracts index missing `{rel}`")
    return (0, "contracts index/nav check passed") if not errors else (1, "\n".join(errors))


def _generate_runbook_map_index(ctx: RunContext) -> tuple[int, str]:
    runbook_dir = ctx.repo_root / "docs/operations/runbooks"
    map_doc = ctx.repo_root / "docs/operations/observability/runbook-dashboard-alert-map.md"
    out = ctx.repo_root / "docs/_generated/runbook-map-index.md"
    runbooks = sorted(p.name for p in runbook_dir.glob("*.md") if p.name != "INDEX.md")
    mapped = set(re.findall(r"\|\s*`([^`]+\.md)`\s*\|", map_doc.read_text(encoding="utf-8", errors="ignore")))
    lines = [
        "# Runbook Map Index (Generated)",
        "",
        "Generated from runbooks and observability runbook map.",
        "",
        f"- Total runbooks: `{len(runbooks)}`",
        f"- Mapped runbooks: `{len([r for r in runbooks if r in mapped])}`",
        "",
        "| Runbook | In map |",
        "|---|---|",
    ]
    for name in runbooks:
        lines.append(f"| `{name}` | {'yes' if name in mapped else 'no'} |")
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0, str(out.relative_to(ctx.repo_root))


def _check_concept_registry(ctx: RunContext) -> tuple[int, str]:
    docs = ctx.repo_root / "docs"
    registry = docs / "_style/concepts.yml"
    id_pat = re.compile(r"^Concept ID:\s*`?([a-z0-9.-]+)`?\s*$", re.MULTILINE)
    ids_pat = re.compile(r"^Concept IDs:\s*`?([a-z0-9.,\s-]+)`?\s*$", re.MULTILINE)

    def extract_ids(text: str) -> list[str]:
        ids: list[str] = []
        for m in id_pat.finditer(text):
            ids.append(m.group(1).strip())
        for m in ids_pat.finditer(text):
            ids.extend([p.strip() for p in m.group(1).split(",") if p.strip()])
        unique: list[str] = []
        for concept_id in ids:
            if concept_id not in unique:
                unique.append(concept_id)
        return unique

    data = yaml.safe_load(registry.read_text(encoding="utf-8")) if registry.exists() else {}
    concepts = data.get("concepts", []) if isinstance(data, dict) else []
    errors: list[str] = []
    if not concepts:
        errors.append(f"{registry.relative_to(ctx.repo_root)}: missing concepts list")
    registry_ids: set[str] = set()
    canonical_by_id: dict[str, str] = {}
    pointers_by_id: dict[str, list[str]] = {}
    for item in concepts:
        concept_id = item.get("id")
        canonical = item.get("canonical")
        pointers = item.get("pointers", [])
        if not concept_id or not canonical:
            errors.append("concept entry missing id or canonical")
            continue
        if concept_id in registry_ids:
            errors.append(f"duplicate concept id in registry: {concept_id}")
        registry_ids.add(concept_id)
        canonical_by_id[concept_id] = canonical
        pointers_by_id[concept_id] = pointers

    canonical_claims: dict[str, list[str]] = {k: [] for k in registry_ids}
    for concept_id, canonical in canonical_by_id.items():
        path = ctx.repo_root / canonical
        if not path.exists():
            errors.append(f"{concept_id}: missing canonical file {canonical}")
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        ids = extract_ids(text)
        if concept_id not in ids and not canonical.startswith("docs/contracts/"):
            errors.append(f"{canonical}: missing declaration for {concept_id}")
        if "Canonical page:" in text:
            errors.append(f"{canonical}: canonical page must not be a pointer")
        canonical_claims[concept_id].append(canonical)
        for pointer in pointers_by_id.get(concept_id, []):
            ppath = ctx.repo_root / pointer
            if not ppath.exists():
                errors.append(f"{concept_id}: missing pointer file {pointer}")
                continue
            ptxt = ppath.read_text(encoding="utf-8", errors="ignore")
            pids = extract_ids(ptxt)
            if concept_id not in pids:
                errors.append(f"{pointer}: missing declaration for {concept_id}")
            if "Canonical page:" not in ptxt:
                errors.append(f"{pointer}: pointer missing `Canonical page:` line")
            if canonical not in ptxt:
                errors.append(f"{pointer}: pointer must link to {canonical}")

    for md in docs.rglob("*.md"):
        text = md.read_text(encoding="utf-8", errors="ignore")
        ids = extract_ids(text)
        rel = md.relative_to(ctx.repo_root).as_posix()
        for concept_id in ids:
            if concept_id not in registry_ids:
                errors.append(f"{rel}: concept `{concept_id}` not declared in docs/_style/concepts.yml")
            elif "Canonical page:" not in text:
                canonical_claims[concept_id].append(rel)

    for concept_id, files in canonical_claims.items():
        unique = sorted(set(files))
        if len(unique) != 1:
            errors.append(f"{concept_id}: expected one canonical page, got {unique}")
            continue
        expected = canonical_by_id[concept_id]
        if unique[0] != expected:
            errors.append(f"{concept_id}: canonical mismatch, expected {expected}, found {unique[0]}")

    return (0, "concept registry check passed") if not errors else (1, "\n".join(errors))


def _check_adr_headers(ctx: RunContext) -> tuple[int, str]:
    errors: list[str] = []
    acronyms = {"ADR", "API", "SSOT", "CLI", "CI", "SQL", "SQLITE", "K8S"}
    for path in sorted((ctx.repo_root / "docs/adrs").glob("ADR-*.md")):
        if path.name == "INDEX.md":
            continue
        m = re.match(r"ADR-(\d{4})-([a-z0-9-]+)\.md$", path.name)
        rel = path.relative_to(ctx.repo_root).as_posix()
        if not m:
            errors.append(f"invalid ADR filename: {rel}")
            continue
        num = m.group(1)
        lines = path.read_text(encoding="utf-8", errors="ignore").splitlines()
        first = lines[0].strip() if lines else ""
        prefix = f"# ADR-{num}: "
        if not first.startswith(prefix):
            errors.append(f"header mismatch in {rel}: missing `{prefix}` prefix")
            continue
        title = first[len(prefix) :].strip()
        if not title:
            errors.append(f"header mismatch in {rel}: missing ADR title text")
            continue
        for word in re.findall(r"[A-Za-z0-9]+", title):
            if word.upper() in acronyms:
                continue
            if not word[0].isupper():
                errors.append(f"header mismatch in {rel}: non-title-case word `{word}`")
                break
    return (0, "ADR header check passed") if not errors else (1, "\n".join(errors))


def _check_broken_examples(ctx: RunContext) -> tuple[int, str]:
    docs = ctx.repo_root / "docs"
    codeblock = re.compile(r"```(?:bash|sh)\n(.*?)```", re.DOTALL)
    cmdline = re.compile(r"^\$\s+(.+)$", re.MULTILINE)
    make_db = subprocess.run(["make", "-qp"], cwd=ctx.repo_root, text=True, capture_output=True, check=False).stdout
    make_targets = set(re.findall(r"^([A-Za-z0-9_.%/+\-]+):", make_db, flags=re.MULTILINE))
    errors: list[str] = []
    for md in docs.rglob("*.md"):
        text = md.read_text(encoding="utf-8", errors="ignore")
        rel = md.relative_to(ctx.repo_root).as_posix()
        for block in codeblock.findall(text):
            for cmd in cmdline.findall(block):
                parts = cmd.strip().split()
                while parts and re.match(r"^[A-Z_][A-Z0-9_]*=.*$", parts[0]):
                    parts = parts[1:]
                if not parts:
                    continue
                tok = parts[0]
                if tok == "make":
                    if len(parts) < 2 or parts[1] not in make_targets:
                        errors.append(f"{rel}: unknown make target in example `{cmd}`")
                    continue
                if tok.startswith("./"):
                    path = (ctx.repo_root / tok).resolve()
                    if not path.exists() or not path.is_file() or not (path.stat().st_mode & 0o111):
                        errors.append(f"{rel}: non-executable script path `{tok}`")
                    continue
                if tok in {"curl", "kubectl", "k6", "cargo", "rg", "python3", "helm", "jq", "cat"}:
                    continue
                errors.append(f"{rel}: command not backed by script path or allowed tool `{cmd}`")
    return (0, "broken examples check passed") if not errors else (1, "\n".join(errors))


def _check_doc_filename_style(ctx: RunContext) -> tuple[int, str]:
    docs = ctx.repo_root / "docs"
    kebab = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*\.md$")
    index = re.compile(r"^INDEX\.md$")
    adr = re.compile(r"^ADR-\d{4}-[a-z0-9-]+\.md$")
    scream = re.compile(r"^[A-Z0-9_]+\.md$")
    exceptions = {"docs/STYLE.md", "docs/contracts/README.md"}

    def allowed(path: Path) -> bool:
        rel = path.relative_to(ctx.repo_root).as_posix()
        if rel in exceptions:
            return True
        name = path.name
        if kebab.match(name) or index.match(name) or adr.match(name):
            return True
        if rel.startswith("docs/_style/") and scream.match(name):
            return True
        if rel.startswith("docs/_generated/contracts/") and scream.match(name):
            return True
        if rel.startswith("docs/operations/slo/") and scream.match(name):
            return True
        return False

    bad = [p.relative_to(ctx.repo_root).as_posix() for p in sorted(docs.rglob("*.md")) if not allowed(p)]
    return (0, "doc filename style check passed") if not bad else (1, "\n".join(bad))


def _check_no_placeholders(ctx: RunContext) -> tuple[int, str]:
    docs = ctx.repo_root / "docs"
    pat = re.compile(r"\b(TODO|TBD|placeholder|coming soon)\b", re.IGNORECASE)
    violations: list[str] = []
    for md in sorted(docs.rglob("*.md")):
        rel = md.relative_to(ctx.repo_root).as_posix()
        if rel.startswith("docs/_drafts/"):
            continue
        for i, line in enumerate(md.read_text(encoding="utf-8", errors="ignore").splitlines(), start=1):
            if pat.search(line):
                violations.append(f"{rel}:{i}: placeholder marker")
    return (0, "docs placeholder check passed") if not violations else (1, "\n".join(violations[:200]))


def _check_no_legacy_root_paths(ctx: RunContext) -> tuple[int, str]:
    docs = ctx.repo_root / "docs"
    patterns = [
        re.compile(r"(^|[`\\s])\\.?/charts/"),
        re.compile(r"(^|[`\\s])\\.?/e2e/"),
        re.compile(r"(^|[`\\s])\\.?/load/"),
        re.compile(r"(^|[`\\s])\\.?/observability/"),
        re.compile(r"(^|[`\\s])\\.?/datasets/"),
        re.compile(r"(^|[`\\s])\\.?/fixtures/"),
    ]
    exceptions = {"docs/operations/migration-note.md"}
    violations: list[str] = []
    for path in sorted(docs.rglob("*.md")):
        rel = path.relative_to(ctx.repo_root).as_posix()
        if rel in exceptions:
            continue
        for lineno, line in enumerate(path.read_text(encoding="utf-8", errors="ignore").splitlines(), start=1):
            if any(pat.search(line) for pat in patterns):
                violations.append(f"{rel}:{lineno}: legacy root path reference")
    return (0, "legacy root path docs check passed") if not violations else (1, "\n".join(violations[:200]))


def _check_mkdocs_site_links(ctx: RunContext, site_dir: str) -> tuple[int, str]:
    class LinkParser(HTMLParser):
        def __init__(self) -> None:
            super().__init__()
            self.links: list[str] = []

        def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
            if tag != "a":
                return
            href = dict(attrs).get("href")
            if href:
                self.links.append(href)

    site = (ctx.repo_root / site_dir).resolve() if not Path(site_dir).is_absolute() else Path(site_dir)
    if not site.exists():
        return 2, f"site dir missing: {site_dir}"
    errors: list[str] = []
    for html in site.rglob("*.html"):
        if html.name == "404.html":
            continue
        parser = LinkParser()
        parser.feed(html.read_text(encoding="utf-8", errors="ignore"))
        for href in parser.links:
            if href.startswith(("http://", "https://", "mailto:", "#")):
                continue
            if href.startswith("/"):
                target_root = href.split("#", 1)[0].lstrip("/")
                resolved = (site / target_root).resolve()
                if resolved.is_dir():
                    resolved = resolved / "index.html"
                elif resolved.suffix == "":
                    resolved = resolved.with_suffix(".html")
                if not resolved.exists():
                    errors.append(f"{html.relative_to(ctx.repo_root)}: broken site-root link -> {href}")
                continue
            target = href.split("#", 1)[0]
            if not target:
                continue
            resolved = (html.parent / target).resolve()
            if resolved.is_dir():
                resolved = resolved / "index.html"
            if not resolved.exists():
                errors.append(f"{html.relative_to(ctx.repo_root)}: broken link -> {href}")
    return (0, "mkdocs output link-check passed") if not errors else (1, "\n".join(errors))


def _check_nav_order(ctx: RunContext) -> tuple[int, str]:
    mkdocs = (ctx.repo_root / "mkdocs.yml").read_text(encoding="utf-8", errors="ignore")
    expected = [
        "Product",
        "Quickstart",
        "Reference",
        "Contracts",
        "API",
        "Operations",
        "Development",
        "Architecture",
        "Science",
        "Generated",
    ]
    nav_start = mkdocs.find("\nnav:\n")
    if nav_start == -1:
        return 1, "nav ordering check failed: missing nav section"
    nav_text = mkdocs[nav_start:]
    found = re.findall(r"^  - ([A-Za-z]+):\s*$", nav_text, flags=re.M)
    if found[: len(expected)] != expected:
        return 1, f"nav ordering check failed\nexpected: {expected}\nfound:    {found[:len(expected)]}"
    return 0, "nav ordering check passed"


def _check_docs_deterministic(ctx: RunContext) -> tuple[int, str]:
    mkdocs = (ctx.repo_root / "mkdocs.yml").read_text(encoding="utf-8", errors="ignore")
    docs_mk = (ctx.repo_root / "makefiles/docs.mk").read_text(encoding="utf-8", errors="ignore")
    errors: list[str] = []
    if "enable_creation_date: false" not in mkdocs:
        errors.append("mkdocs.yml must set `enable_creation_date: false`")
    if "fallback_to_build_date: false" not in mkdocs:
        errors.append("mkdocs.yml must set `fallback_to_build_date: false`")
    if "SOURCE_DATE_EPOCH=" not in docs_mk:
        errors.append("makefiles/docs.mk must set SOURCE_DATE_EPOCH for mkdocs build")
    return (0, "docs determinism check passed") if not errors else (1, "\n".join(errors))


def _check_docs_make_targets_exist(ctx: RunContext) -> tuple[int, str]:
    line_cmd_re = re.compile(r"^\s*(?:\$|#)?\s*(?:[A-Za-z_][A-Za-z0-9_]*=[^\s]+\s+)*make\s+([A-Za-z0-9_./-]+)")
    inline_cmd_re = re.compile(r"`(?:[A-Za-z_][A-Za-z0-9_]*=[^\s`]+\s+)*make\s+([A-Za-z0-9_./-]+)`")
    docs = ctx.repo_root / "docs"
    proc = subprocess.run(["make", "-qp"], cwd=ctx.repo_root, text=True, capture_output=True, check=False)
    targets: set[str] = set()
    for line in proc.stdout.splitlines():
        if ":" not in line or line.startswith("\t") or line.startswith("#"):
            continue
        candidate = line.split(":", 1)[0].strip()
        if not candidate:
            continue
        if any(ch in candidate for ch in " %$()"):
            continue
        targets.add(candidate)
    missing: list[str] = []
    for path in sorted(docs.rglob("*.md")):
        rel = path.relative_to(ctx.repo_root).as_posix()
        for lineno, line in enumerate(path.read_text(encoding="utf-8", errors="ignore").splitlines(), start=1):
            matches = list(line_cmd_re.finditer(line)) + list(inline_cmd_re.finditer(line))
            for match in matches:
                target = match.group(1)
                if target not in targets:
                    missing.append(f"{rel}:{lineno}: unknown make target `{target}`")
    return (0, "docs make-target existence check passed") if not missing else (1, "\n".join(missing[:200]))


def _check_critical_make_targets_referenced(ctx: RunContext) -> tuple[int, str]:
    critical = [
        "docs",
        "contracts",
        "ci",
        "local",
        "local-full",
        "ops-up",
        "ops-deploy",
        "ops-smoke",
        "ops-k8s-tests",
        "ops-load-smoke",
        "ops-observability-validate",
        "ops-full",
    ]
    docs = ctx.repo_root / "docs"
    text = "\n".join(p.read_text(encoding="utf-8", errors="ignore") for p in sorted(docs.rglob("*.md")))
    missing = [t for t in critical if f"`{t}`" not in text and f"make {t}" not in text]
    return (0, "critical make target docs coverage passed") if not missing else (1, "\n".join(f"missing docs reference for `{t}`" for t in missing))


def _check_make_targets_documented(ctx: RunContext) -> tuple[int, str]:
    surface_path = ctx.repo_root / "docs/development/makefiles/surface.md"
    targets_path = ctx.repo_root / "docs/development/make-targets.md"
    surface_doc = surface_path.read_text(encoding="utf-8", errors="ignore") if surface_path.exists() else ""
    targets_doc = targets_path.read_text(encoding="utf-8", errors="ignore") if targets_path.exists() else ""
    help_out = subprocess.run(["make", "help"], cwd=ctx.repo_root, text=True, capture_output=True, check=False).stdout.strip().splitlines()
    if len(help_out) < 3:
        return 1, "make target docs check failed: unexpected `make help` output"
    missing: list[str] = []
    for line in help_out:
        if not line.startswith("    "):
            continue
        token = line.strip().split()[0]
        if token.startswith("[") and token.endswith("]"):
            continue
        if f"`{token}`" not in surface_doc and f"`{token}`" not in targets_doc:
            missing.append(token)
    return (0, "make target docs check passed") if not missing else (1, "\n".join(missing))


def _check_make_targets_drift(ctx: RunContext) -> tuple[int, str]:
    paths = [ctx.repo_root / "docs/development/make-targets.md", ctx.repo_root / "docs/development/make-targets-inventory.md"]

    def digest(path: Path) -> str:
        return hashlib.sha256(path.read_bytes()).hexdigest() if path.exists() else ""

    before = {str(p): digest(p) for p in paths}
    code, out = _run_check(["python3", "-m", "atlasctl.cli", "docs", "generate", "--report", "text"], ctx.repo_root)
    if code != 0:
        return 1, out
    after = {str(p): digest(p) for p in paths}
    if before != after:
        return 1, "make-target docs drift detected; regenerate and commit"
    return 0, "make-target docs drift check passed"


def _check_docker_entrypoints(ctx: RunContext) -> tuple[int, str]:
    violations: list[str] = []
    pattern = re.compile(r"(^|\n)\s*\$\s*docker\s+build\b")
    for md in (ctx.repo_root / "docs").rglob("*.md"):
        text = md.read_text(encoding="utf-8", errors="ignore")
        for match in pattern.finditer(text):
            line = text.count("\n", 0, match.start()) + 1
            violations.append(f"{md.relative_to(ctx.repo_root)}:{line}")
    if violations:
        return 1, "docs must use make docker-build instead of direct docker build:\n" + "\n".join(
            f"- {item}" for item in violations
        )
    return 0, "docker entrypoint docs check passed"


def _check_example_configs(ctx: RunContext) -> tuple[int, str]:
    example = json.loads((ctx.repo_root / "docs" / "examples" / "policy-config.example.json").read_text(encoding="utf-8"))
    schema = json.loads((ctx.repo_root / "docs" / "contracts" / "POLICY_SCHEMA.json").read_text(encoding="utf-8"))
    required = set(schema.get("required", []))
    missing = sorted(required - set(example.keys()))
    extra = sorted(set(example.keys()) - set(schema.get("properties", {}).keys()))
    if missing:
        return 1, f"example config validation failed: missing keys {missing}"
    if extra:
        return 1, f"example config validation failed: unknown keys {extra}"
    return 0, "example config validation passed"


def _check_full_stack_page(ctx: RunContext) -> tuple[int, str]:
    page = ctx.repo_root / "docs" / "operations" / "full-stack-local.md"
    text = page.read_text(encoding="utf-8")
    lines = [line for line in text.splitlines() if line.strip()]
    if len(lines) > 80:
        return 1, "full-stack page exceeds one-page policy (>80 non-empty lines)"
    required = "make ops-up ops-deploy ops-warm ops-smoke"
    if required not in text:
        return 1, "full-stack page missing canonical command sequence"
    mk = (ctx.repo_root / "makefiles" / "ops.mk").read_text(encoding="utf-8")
    for target in ["ops-up", "ops-deploy", "ops-warm", "ops-smoke"]:
        if not re.search(rf"^{target}:", mk, flags=re.MULTILINE):
            return 1, f"missing target in ops.mk: {target}"
    return 0, "full stack page check passed"


def _check_k8s_docs_contract(ctx: RunContext) -> tuple[int, str]:
    k8s_dir = ctx.repo_root / "docs" / "operations" / "k8s"
    values = json.loads((ctx.repo_root / "docs" / "contracts" / "CHART_VALUES.json").read_text(encoding="utf-8"))
    keys = set(values.get("top_level_keys", []))
    errors: list[str] = []
    for path in sorted(k8s_dir.glob("*.md")):
        if path.name == "INDEX.md":
            continue
        text = path.read_text(encoding="utf-8")
        refs = [ref for ref in re.findall(r"`values\.([a-z][a-zA-Z0-9_\-]*)`", text) if ref != "yaml"]
        if not refs:
            errors.append(f"{path}: missing values.<key> references")
            continue
        for ref in refs:
            if ref not in keys:
                errors.append(f"{path}: unknown chart values key `{ref}`")
    return (0, "k8s docs contract check passed") if not errors else (
        1,
        "k8s docs contract check failed:\n" + "\n".join(f"- {e}" for e in errors),
    )


def _check_load_docs_contract(ctx: RunContext) -> tuple[int, str]:
    load_doc_dir = ctx.repo_root / "docs" / "operations" / "load"
    scenario_dir = ctx.repo_root / "ops" / "load" / "scenarios"
    if not scenario_dir.exists():
        scenario_dir = ctx.repo_root / "ops" / "e2e" / "k6" / "scenarios"
    if not scenario_dir.exists():
        scenario_dir = ctx.repo_root / "e2e" / "k6" / "scenarios"
    scenarios = {p.name for p in scenario_dir.glob("*.json")}
    suites_manifest = ctx.repo_root / "ops" / "load" / "suites" / "suites.json"
    suite_ids: set[str] = set()
    if suites_manifest.exists():
        data = json.loads(suites_manifest.read_text(encoding="utf-8"))
        suite_ids = {item["name"] for item in data.get("suites", []) if isinstance(item, dict) and "name" in item}
    errors: list[str] = []
    for path in sorted(load_doc_dir.glob("*.md")):
        text = path.read_text(encoding="utf-8")
        refs = [ref for ref in re.findall(r"`([a-zA-Z0-9_\-]+\.json)`", text) if ref != "suites.json"]
        suite_refs = [ref for ref in re.findall(r"`([a-z0-9][a-z0-9\-]+)`", text) if ref in suite_ids]
        if not refs and not suite_refs:
            errors.append(f"{path}: no load suite or k6 scenario references found")
            continue
        for ref in refs:
            if ref not in scenarios:
                errors.append(f"{path}: unknown k6 scenario `{ref}`")
    return (0, "load docs contract check passed") if not errors else (
        1,
        "load docs contract check failed:\n" + "\n".join(f"- {e}" for e in errors),
    )


def _parse_make_help_sections(text: str) -> dict[str, list[str]]:
    sections: dict[str, list[str]] = {}
    current: str | None = None
    for line in text.splitlines():
        if line.endswith(":") and not line.startswith("  "):
            current = line[:-1]
            sections[current] = []
            continue
        if current and line.startswith("  "):
            sections[current].append(line.strip().split()[0])
    return sections


def _parse_make_targets_doc_sections(text: str) -> dict[str, list[str]]:
    sections: dict[str, list[str]] = {}
    current: str | None = None
    for line in text.splitlines():
        if line.startswith("## "):
            current = line[3:].strip().lower()
            sections[current] = []
            continue
        if current:
            match = re.match(r"^- `([^`]+)`$", line.strip())
            if match:
                sections[current].append(match.group(1))
    return sections


def _check_make_help_drift(ctx: RunContext) -> tuple[int, str]:
    doc = ctx.repo_root / "docs" / "development" / "make-targets.md"
    help_out = subprocess.check_output(["make", "help"], cwd=ctx.repo_root, text=True)
    help_sections = _parse_make_help_sections(help_out)
    doc_sections = _parse_make_targets_doc_sections(doc.read_text(encoding="utf-8"))
    normalized_help = {k.lower(): v for k, v in help_sections.items()}
    if normalized_help != doc_sections:
        return 1, "make help drift detected vs docs/development/make-targets.md"
    return 0, "make help drift check passed"


def _check_no_removed_make_targets(ctx: RunContext) -> tuple[int, str]:
    scan_dirs = [ctx.repo_root / "docs", ctx.repo_root / "makefiles" / "README.md"]
    removed = {"docker", "chart"}
    patterns = [re.compile(rf"\bmake\s+{re.escape(target)}(?=\s|$|`|,)") for target in sorted(removed)]
    violations: list[str] = []
    files: list[Path] = []
    for item in scan_dirs:
        if item.is_file():
            files.append(item)
        elif item.is_dir():
            files.extend(sorted(item.rglob("*.md")))
    for path in files:
        rel = path.relative_to(ctx.repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        for idx, line in enumerate(text.splitlines(), start=1):
            if "make " not in line:
                continue
            for pattern in patterns:
                if pattern.search(line):
                    violations.append(f"{rel}:{idx}: removed public target reference: {pattern.pattern}")
                    break
    return (0, "removed make target docs check passed") if not violations else (
        1,
        "removed make target docs check failed:\n" + "\n".join(f"- {v}" for v in violations),
    )


def _check_ops_docs_make_targets(ctx: RunContext) -> tuple[int, str]:
    ops_docs = ctx.repo_root / "docs" / "operations"
    help_out = subprocess.check_output(["make", "help"], cwd=ctx.repo_root, text=True)
    targets: list[str] = []
    for line in help_out.splitlines():
        if line.startswith("  "):
            targets.extend(line.strip().split())
    ops_targets = [t for t in sorted(set(targets)) if t.startswith("ops-") or t.startswith("e2e-") or t == "observability-check"]
    if not ops_targets:
        return 1, "ops docs make-target contract failed: no ops targets discovered in `make help`"
    target_pattern = re.compile(r"`(" + "|".join(re.escape(t) for t in ops_targets) + r")`")
    make_cmd_pattern = re.compile(r"\bmake\s+(" + "|".join(re.escape(t) for t in ops_targets) + r")\b")
    errors: list[str] = []
    area_has_target: dict[Path, bool] = {}
    area_index: dict[Path, Path] = {}
    for md in sorted(ops_docs.rglob("*.md")):
        text = md.read_text(encoding="utf-8", errors="ignore")
        area = md.parent
        area_has_target.setdefault(area, False)
        if target_pattern.search(text) or make_cmd_pattern.search(text):
            area_has_target[area] = True
        if md.name == "INDEX.md":
            area_index[area] = md
        if re.search(r"(^|\\s)\\./(ops|scripts)/", text):
            errors.append(f"{md.relative_to(ctx.repo_root)}: direct script path reference found; use make target")
    for area, has_target in sorted(area_has_target.items()):
        if has_target:
            continue
        index = area_index.get(area)
        if index is not None:
            errors.append(f"{index.relative_to(ctx.repo_root)}: missing ops make target reference for area")
        else:
            errors.append(f"{area.relative_to(ctx.repo_root)}: missing INDEX.md with ops make target reference")
    return (0, "ops docs make-target contract passed") if not errors else (
        1,
        "ops docs make-target contract failed:\n" + "\n".join(f"- {e}" for e in errors),
    )


def _check_ops_observability_links(ctx: RunContext) -> tuple[int, str]:
    doc_dir = ctx.repo_root / "docs" / "operations" / "observability"
    link_re = re.compile(r"\[[^\]]+\]\(([^)]+)\)")
    errors: list[str] = []
    for md in sorted(doc_dir.glob("*.md")):
        text = md.read_text(encoding="utf-8", errors="ignore")
        for link in link_re.findall(text):
            if link.startswith(("http://", "https://", "mailto:", "#")):
                continue
            target = link.split("#", 1)[0]
            if not target:
                continue
            if not (md.parent / target).resolve().exists():
                errors.append(f"{md.relative_to(ctx.repo_root)} -> missing link target: {link}")
    return (0, "ops observability link-check passed") if not errors else (
        1,
        "ops observability link-check failed:\n" + "\n".join(f"- {e}" for e in errors),
    )


def _check_public_targets_docs_sections(ctx: RunContext) -> tuple[int, str]:
    targets_path = ctx.repo_root / "makefiles" / "targets.json"
    doc = ctx.repo_root / "docs" / "_generated" / "make-targets.md"
    mkdocs = ctx.repo_root / "mkdocs.yml"
    errors: list[str] = []
    if "_generated/make-targets.md" not in mkdocs.read_text(encoding="utf-8", errors="ignore"):
        errors.append("mkdocs.yml missing nav entry for docs/_generated/make-targets.md")
    targets = json.loads(targets_path.read_text(encoding="utf-8")).get("targets", [])
    doc_text = doc.read_text(encoding="utf-8", errors="ignore")
    for target in targets:
        name = target.get("name", "") if isinstance(target, dict) else ""
        if name and f"`{name}`" not in doc_text:
            errors.append(f"docs/_generated/make-targets.md missing `{name}`")
    return (0, "public target docs section check passed") if not errors else (
        1,
        "public target docs section check failed:\n" + "\n".join(f"- {e}" for e in errors),
    )


def _check_suite_id_docs(ctx: RunContext) -> tuple[int, str]:
    targets = [
        ctx.repo_root / "docs" / "operations" / "k8s" / "k8s-test-contract.md",
        ctx.repo_root / "docs" / "operations" / "load" / "INDEX.md",
        ctx.repo_root / "docs" / "operations" / "load" / "k6.md",
        ctx.repo_root / "docs" / "operations" / "load" / "suites.md",
    ]
    bad_patterns = [
        re.compile(r"\btest_[a-z0-9_]+\.sh\b"),
        re.compile(
            r"\b(?:mixed|spike|cold-start|stampede|store-outage-under-spike|pod-churn|cheap-only-survival|response-size-abuse|multi-release|sharded-fanout|diff-heavy|mixed-gene-sequence|soak-30m|redis-optional|catalog-federated|multi-dataset-hotset|large-dataset-simulation|load-under-rollout|load-under-rollback)\.json\b"
        ),
    ]
    errors: list[str] = []
    for path in targets:
        rel = path.relative_to(ctx.repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        for pattern in bad_patterns:
            for match in pattern.findall(text):
                errors.append(f"{rel}: reference suite ID instead of file `{match}`")
    return (0, "suite-id docs check passed") if not errors else (
        1,
        "suite-id docs check failed:\n" + "\n".join(f"- {e}" for e in errors),
    )


def _check_configmap_env_docs(ctx: RunContext) -> tuple[int, str]:
    template = ctx.repo_root / "ops" / "k8s" / "charts" / "bijux-atlas" / "templates" / "configmap.yaml"
    config_doc = ctx.repo_root / "docs" / "operations" / "config.md"
    values_doc = ctx.repo_root / "docs" / "operations" / "k8s" / "values.md"
    tmpl_text = template.read_text(encoding="utf-8")
    doc_text = config_doc.read_text(encoding="utf-8")
    values_doc_text = values_doc.read_text(encoding="utf-8")
    cfg_keys = sorted(set(re.findall(r"^\s+(ATLAS_[A-Z0-9_]+):", tmpl_text, flags=re.MULTILINE)))
    top_level_values = sorted(set(re.findall(r"\.Values\.([a-zA-Z0-9_]+)", tmpl_text)))
    errors: list[str] = []
    for key in cfg_keys:
        if f"`{key}`" not in doc_text:
            errors.append(f"missing key in docs/operations/config.md: {key}")
    for top in top_level_values:
        if f"`values.{top}`" not in values_doc_text:
            errors.append(f"missing values reference in docs/operations/k8s/values.md: values.{top}")
    return (0, "configmap env docs check passed") if not errors else (
        1,
        "configmap env docs check failed:\n" + "\n".join(f"- {e}" for e in errors),
    )


def _check_generated_contract_docs(ctx: RunContext) -> tuple[int, str]:
    code, out = _run_check(["./bin/bijux-atlas", "contracts", "generate", "--generators", "artifacts"], ctx.repo_root)
    if code != 0:
        return 1, out
    code, out = _run_check(
        ["python3", "-m", "atlasctl.cli", "docs", "contracts-index", "--fix", "--report", "text"],
        ctx.repo_root,
    )
    if code != 0:
        return 1, out
    targets = [
        "docs/_generated/contracts",
        "docs/contracts/errors.md",
        "docs/contracts/metrics.md",
        "docs/contracts/tracing.md",
        "docs/contracts/endpoints.md",
        "docs/contracts/config-keys.md",
        "docs/contracts/chart-values.md",
    ]
    proc = subprocess.run(
        ["git", "diff", "--", *targets],
        cwd=ctx.repo_root,
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.stdout.strip() or proc.stderr.strip():
        return 1, "generated contract docs drift detected"
    return 0, "generated contract docs check passed"


def _lint_depth(ctx: RunContext) -> tuple[int, str]:
    docs = ctx.repo_root / "docs"
    artifacts = ctx.repo_root / "artifacts" / "docs"
    report_path = artifacts / "depth-report.md"
    budget_config = ctx.repo_root / "configs" / "docs" / "depth-budget.json"
    required_std = ["what", "why", "contracts", "failure modes", "how to verify"]
    required_runbook = ["symptoms", "metrics", "commands", "mitigations", "rollback"]
    forbidden_terms = ["simple", "just", "obvious", "etc"]
    skip_prefixes = ("_generated/", "_assets/", "_style/")
    major_arch_docs = {
        "architecture/boundaries.md",
        "architecture/effects.md",
        "architecture/boundary-maps.md",
        "architecture/crate-boundary-dependency-graph.md",
    }

    def rel(path: Path) -> str:
        return path.relative_to(docs).as_posix()

    def should_skip(path: Path) -> bool:
        rp = rel(path)
        return any(rp.startswith(prefix) for prefix in skip_prefixes)

    def extract_headings(text: str) -> set[str]:
        return {line[3:].strip().lower() for line in text.splitlines() if line.startswith("## ")}

    def has_verify_command_block(text: str) -> bool:
        lowered = text.lower()
        idx = lowered.find("## how to verify")
        if idx == -1:
            return False
        tail = text[idx:]
        return "```" in tail and bool(re.search(r"\b(make|cargo|python3|scripts/)\b", tail))

    def has_diagram(text: str) -> bool:
        if "```mermaid" in text:
            return True
        return bool(re.search(r"!\[[^\]]*\]\(([^)]+_assets/diagrams/[^)]+)\)", text))

    findings: list[tuple[str, str]] = []
    checked = 0
    threshold = 350
    if budget_config.exists():
        cfg = json.loads(budget_config.read_text(encoding="utf-8"))
        threshold = int(cfg.get("max_findings", threshold))

    for path in sorted(docs.rglob("*.md")):
        if should_skip(path):
            continue
        content = path.read_text(encoding="utf-8")
        headings = extract_headings(content)
        rp = rel(path)
        checked += 1
        is_index = path.name == "INDEX.md"
        is_runbook = rp.startswith("operations/runbooks/")
        is_major = rp.startswith(("reference/", "contracts/", "operations/")) and not is_index
        is_arch = rp in major_arch_docs
        if is_runbook:
            for section in required_runbook:
                if section not in headings:
                    findings.append((path.relative_to(ctx.repo_root).as_posix(), f"missing runbook section: {section}"))
        elif is_major:
            for section in required_std:
                if section not in headings:
                    findings.append((path.relative_to(ctx.repo_root).as_posix(), f"missing required section: {section}"))
        if is_major and (content.count("```") // 2) < 1:
            findings.append((path.relative_to(ctx.repo_root).as_posix(), "requires at least 1 fenced code example"))
        if is_major and not has_verify_command_block(content):
            findings.append((path.relative_to(ctx.repo_root).as_posix(), "verify section must include runnable command block"))
        lowered = content.lower()
        for term in forbidden_terms:
            if re.search(rf"\b{re.escape(term)}\b", lowered):
                findings.append((path.relative_to(ctx.repo_root).as_posix(), f"contains forbidden handwavy term: {term}"))
        if is_arch and not has_diagram(content):
            findings.append((path.relative_to(ctx.repo_root).as_posix(), "architecture doc requires at least one diagram (mermaid or image)"))

    artifacts.mkdir(parents=True, exist_ok=True)
    lines = [
        "# Docs Depth Report",
        "",
        f"- Checked files: {checked}",
        f"- Findings: {len(findings)}",
        f"- Failure threshold: {threshold}",
        "",
    ]
    if findings:
        lines += ["## Findings", ""]
        lines.extend(f"- `{path}`: {msg}" for path, msg in findings)
    else:
        lines.append("No findings.")
    report_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    if len(findings) > threshold:
        return 1, f"depth lint failed with {len(findings)} finding(s) (threshold={threshold}); see {report_path.relative_to(ctx.repo_root)}"
    return 0, f"depth lint passed with {len(findings)} finding(s) (threshold={threshold})"


def _lint_doc_contracts(ctx: RunContext) -> tuple[int, str]:
    docs = ctx.repo_root / "docs"
    excluded = {
        "docs/contracts/INDEX.md",
        "docs/contracts/artifacts/sqlite-migration-strategy.md",
        "docs/contracts/artifacts/sqlite-schema-contract.md",
        "docs/contracts/fasta-derived-metrics.md",
        "docs/contracts/gff3-acceptance.md",
        "docs/contracts/normalized-format.md",
        "docs/contracts/qc.md",
    }
    required_headings = ["what", "why", "scope", "non-goals", "contracts", "failure modes", "how to verify", "see also"]
    banned_marketing = re.compile(r"\b(elite|reference-grade|world-class|best-in-class)\b", re.IGNORECASE)
    banned_vague = re.compile(r"\b(should|might|could|maybe|perhaps)\b", re.IGNORECASE)
    banned_inclusive = re.compile(r"\b(whitelist|blacklist|master|slave)\b", re.IGNORECASE)
    heading_re = re.compile(r"^##\s+(.+?)\s*$", re.MULTILINE)
    codeblock_re = re.compile(r"```(?:bash|sh)\n(.*?)```", re.DOTALL)
    link_re = re.compile(r"\[[^\]]+\]\([^\)]+\)")

    def section_body(text: str, heading: str) -> str:
        pattern = re.compile(rf"^##\s+{re.escape(heading)}\s*$", re.IGNORECASE | re.MULTILINE)
        match = pattern.search(text)
        if not match:
            return ""
        start = match.end()
        next_heading = re.compile(r"^##\s+", re.MULTILINE).search(text, start)
        end = next_heading.start() if next_heading else len(text)
        return text[start:end].strip()

    errors: list[str] = []
    targets = sorted(
        file
        for file in (docs / "contracts").rglob("*.md")
        if file.relative_to(ctx.repo_root).as_posix() not in excluded
    )
    for file in targets:
        rel = file.relative_to(ctx.repo_root).as_posix()
        text = file.read_text(encoding="utf-8")
        headings = {h.strip().lower() for h in heading_re.findall(text)}
        allow_marketing = rel == "docs/product/reference-grade-checklist.md"
        for req in required_headings:
            if req not in headings:
                errors.append(f"{rel}: missing required heading '## {req.title()}'")
        if not allow_marketing and banned_marketing.search(text):
            errors.append(f"{rel}: contains banned marketing adjectives")
        if banned_vague.search(text):
            errors.append(f"{rel}: contains vague verbs (should/might/could/maybe/perhaps)")
        if banned_inclusive.search(text):
            errors.append(f"{rel}: contains non-inclusive terminology")
        examples = section_body(text, "Examples")
        if not examples:
            errors.append(f"{rel}: missing required heading '## Examples'")
        else:
            if "```" not in examples:
                errors.append(f"{rel}: examples section must include fenced code block")
            has_expected = "expected output" in examples.lower()
            for block in codeblock_re.findall(examples):
                lines = [line.strip() for line in block.splitlines() if line.strip()]
                if lines and not all(line.startswith("$") for line in lines):
                    errors.append(f"{rel}: shell code blocks must include full commands prefixed with '$'")
                    break
            if codeblock_re.search(examples) and not has_expected:
                errors.append(f"{rel}: command snippets require an 'Expected output' description")
        see_also = section_body(text, "See also")
        links = link_re.findall(see_also)
        if not (3 <= len(links) <= 8):
            errors.append(f"{rel}: 'See also' must contain 3-8 links")
        if "terms-glossary.md" not in see_also:
            errors.append(f"{rel}: 'See also' must include glossary link")
        if "- Owner:" not in text:
            errors.append(f"{rel}: missing owner header ('- Owner:')")
    return (0, "doc contracts lint passed") if not errors else (1, "\n".join(errors))


def _generate_concept_graph(ctx: RunContext) -> tuple[int, str]:
    registry = ctx.repo_root / "docs/_style/concepts.yml"
    out = ctx.repo_root / "docs/_generated/concepts.md"
    data = yaml.safe_load(registry.read_text(encoding="utf-8")) if registry.exists() else {}
    concepts = data.get("concepts", []) if isinstance(data, dict) else []
    lines = [
        "# Concept Graph",
        "",
        "- Owner: `docs-governance`",
        "",
        "## What",
        "",
        "Generated mapping of concept IDs to canonical and pointer pages.",
        "",
        "## Why",
        "",
        "Provides a deterministic lookup for concept ownership.",
        "",
        "## Scope",
        "",
        "Concept registry entries from `docs/_style/concepts.yml`.",
        "",
        "## Non-goals",
        "",
        "No semantic interpretation beyond declared links.",
        "",
        "## Contracts",
        "",
        "- Exactly one canonical page per concept.",
        "- Pointer pages must reference canonical page.",
        "",
        "## Failure modes",
        "",
        "Registry drift causes stale concept ownership.",
        "",
        "## How to verify",
        "",
        "```bash",
        "$ atlasctl docs concept-registry-check --report text",
        "$ make docs",
        "```",
        "",
        "Expected output: concept checks pass.",
        "",
        "## Concepts",
        "",
    ]
    for item in concepts:
        concept_id = item["id"]
        canonical = item["canonical"].replace("docs/", "")
        pointers = [p.replace("docs/", "") for p in item.get("pointers", [])]
        lines.append(f"### `{concept_id}`")
        lines.append("")
        lines.append(f"- Canonical: [{canonical}](../{canonical})")
        if pointers:
            for pointer in pointers:
                lines.append(f"- Pointer: [{pointer}](../{pointer})")
        else:
            lines.append("- Pointer: none")
        lines.append("")
    lines.extend(
        [
            "## See also",
            "",
            "- [Concept Registry](../_style/CONCEPT_REGISTRY.md)",
            "- [Concept IDs](../_style/concept-ids.md)",
            "- [Docs Home](../index.md)",
            "",
        ]
    )
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(lines), encoding="utf-8")
    return 0, f"generated {out.relative_to(ctx.repo_root)}"


def _render_diagrams(ctx: RunContext) -> tuple[int, str]:
    diagram_dir = ctx.repo_root / "docs/_assets/diagrams"
    rendered = 0
    messages: list[str] = []

    mmdc = shutil.which("mmdc")
    if mmdc:
        for src in sorted(diagram_dir.rglob("*.mmd")):
            out = src.with_suffix(".svg")
            proc = subprocess.run([mmdc, "-i", str(src), "-o", str(out)], cwd=ctx.repo_root, text=True, capture_output=True, check=False)
            if proc.returncode == 0:
                rendered += 1
            else:
                messages.append((proc.stdout + proc.stderr).strip() or f"mmdc failed for {src.relative_to(ctx.repo_root)}")
    else:
        messages.append("mmdc not found; skipping Mermaid rendering")

    plantuml = shutil.which("plantuml")
    if plantuml:
        for src in sorted(diagram_dir.rglob("*.puml")):
            proc = subprocess.run([plantuml, "-tsvg", str(src)], cwd=ctx.repo_root, text=True, capture_output=True, check=False)
            if proc.returncode == 0:
                rendered += 1
            else:
                messages.append((proc.stdout + proc.stderr).strip() or f"plantuml failed for {src.relative_to(ctx.repo_root)}")
    else:
        messages.append("plantuml not found; skipping PlantUML rendering")

    if rendered == 0:
        return 0, "diagram render check completed (no renderer available or no sources)"
    return 0, f"rendered {rendered} diagram(s)" + (f"\n{chr(10).join(messages)}" if messages else "")


def _rewrite_legacy_terms(ctx: RunContext, path_arg: str, apply: bool) -> tuple[int, str]:
    replacements: list[tuple[re.Pattern[str], str]] = [
        (re.compile(r"\bphase\s+([0-9]+)\b", re.IGNORECASE), "stability level: provisional"),
        (re.compile(r"\bstep\s+([0-9]+)\b", re.IGNORECASE), "checkpoint"),
        (re.compile(r"\bstage\s+([0-9]+)\b", re.IGNORECASE), "boundary"),
        (re.compile(r"\btask\s+([0-9]+)\b", re.IGNORECASE), "requirement"),
        (re.compile(r"\biteration\s+([0-9]+)\b", re.IGNORECASE), "revision"),
        (re.compile(r"\bround\s+([0-9]+)\b", re.IGNORECASE), "review cycle"),
        (re.compile(r"\bWIP\b", re.IGNORECASE), "draft"),
        (re.compile(r"\btemporary\b", re.IGNORECASE), "provisional"),
        (re.compile(r"vnext\s+placeholder", re.IGNORECASE), "future extension (documented non-goal)"),
    ]
    path = (ctx.repo_root / path_arg).resolve() if not Path(path_arg).is_absolute() else Path(path_arg)
    if not path.exists():
        return 2, f"missing: {path_arg}"
    text = path.read_text(encoding="utf-8", errors="ignore")
    out = text
    for pattern, replacement in replacements:
        out = pattern.sub(replacement, out)
    if out == text:
        return 0, "no legacy term replacements needed"
    if apply:
        path.write_text(out, encoding="utf-8")
        return 0, f"rewrote {path.relative_to(ctx.repo_root)}"
    diffs: list[str] = [f"--- {path.relative_to(ctx.repo_root)}"]
    for idx, (old, new) in enumerate(zip(text.splitlines(), out.splitlines()), start=1):
        if old != new:
            diffs.append(f"L{idx}: - {old}")
            diffs.append(f"L{idx}: + {new}")
    return 1, "\n".join(diffs)


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
                        "scripts/areas/public/perf/",
            "scripts/areas/public/observability/",
            "packages/atlasctl/src/atlasctl/checks/layout/",
            "scripts/areas/public/",
        ]
        for group in required_groups:
            if group not in text:
                errors.append(f"{idx.relative_to(root)}: missing script group reference `{group}`")
    else:
        errors.append(f"{idx.relative_to(root)}: missing scripts index")
    return (0, "script header check passed") if not errors else (1, "\n".join(errors))


def _check_terminology_units_ssot(ctx: RunContext) -> tuple[int, str]:
    docs = ctx.repo_root / "docs"
    errors: list[str] = []
    term_bans = {
        r"\bgenome build\b": "assembly",
        r"\bwhitelist\b": "allowlist",
        r"\bblacklist\b": "denylist",
    }
    units_pat = re.compile(
        r"\b(coordinate|span|size|latency|timeout)\b[^\n]{0,40}\b(?<![pP.])(\d{2,})\b(?!\s*(bp|bytes|seconds|ms|s))(?!\.)",
        re.IGNORECASE,
    )
    ssot_ban = re.compile(r"docs/contracts/(ERROR_CODES|METRICS|TRACE_SPANS|ENDPOINTS|CONFIG_KEYS|CHART_VALUES)\.json")
    for path in docs.rglob("*.md"):
        text = path.read_text(encoding="utf-8", errors="ignore")
        if path.name == "terms-glossary.md":
            continue
        rel = path.relative_to(ctx.repo_root).as_posix()
        for pat, repl in term_bans.items():
            if re.search(pat, text, flags=re.IGNORECASE):
                errors.append(f"{rel}: terminology violation; use `{repl}`")
        if {"reference", "product", "operations"} & set(path.parts) and units_pat.search(text):
            errors.append(f"{rel}: possible missing unit annotation (bp/bytes/seconds)")
        if "contracts" not in path.parts and ssot_ban.search(text):
            errors.append(f"{rel}: reference docs/contracts/*.md instead of raw registry json")
    return (0, "terminology/units/ssot check passed") if not errors else (1, "\n".join(errors))


def _lint_doc_status(ctx: RunContext) -> tuple[int, str]:
    docs = ctx.repo_root / "docs"
    out = docs / "_generated/doc-status.md"
    allowed = {"active", "frozen", "draft"}
    drafted: list[str] = []
    invalid: list[str] = []
    rows: list[tuple[str, str]] = []

    def _read_status(path: Path) -> str | None:
        text = path.read_text(encoding="utf-8", errors="ignore")
        if not text.startswith("---\n"):
            return None
        end = text.find("\n---\n", 4)
        if end == -1:
            return None
        frontmatter = text[4:end]
        for line in frontmatter.splitlines():
            m = re.match(r"status:\s*([a-zA-Z-]+)\s*$", line.strip())
            if m:
                return m.group(1).lower()
        return None

    def _badge(status: str) -> str:
        mapping = {
            "active": "![active](https://img.shields.io/badge/status-active-brightgreen)",
            "frozen": "![frozen](https://img.shields.io/badge/status-frozen-blue)",
            "draft": "![draft](https://img.shields.io/badge/status-draft-lightgrey)",
        }
        return mapping[status]

    for path in sorted(docs.rglob("*.md")):
        rel = path.relative_to(docs).as_posix()
        if rel.startswith("_generated/"):
            continue
        status = _read_status(path)
        if status is None:
            continue
        if status not in allowed:
            invalid.append(f"{rel}: {status}")
            continue
        rows.append((rel, status))
        if status == "draft":
            drafted.append(rel)

    lines = ["# Document Status", "", "## What", "Status summary generated from document frontmatter.", "", "## Contracts", "- Allowed statuses: `active`, `frozen`, `draft`.", "- `draft` is forbidden on default branch.", "", "## Pages", "", "| Page | Status |", "|---|---|"]
    for rel, status in rows:
        lines.append(f"| `{rel}` | {_badge(status)} `{status}` |")
    if not rows:
        lines.append("| (none) | n/a |")
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    if invalid:
        return 1, "invalid doc status values:\n" + "\n".join(f"- {i}" for i in invalid)
    if drafted:
        return 1, "draft docs are not allowed:\n" + "\n".join(f"- {i}" for i in drafted)
    return 0, "doc status lint passed"


def _check_title_case(ctx: RunContext) -> tuple[int, str]:
    allow = re.compile(r"API|SSOT|ADR|K8s|k6|v1|CI|CLI|JSON|YAML|HMAC|SSRF|SLO|SDK|DNA|ETag|URL|GC|EMBL")
    errors: list[str] = []
    for path in sorted((ctx.repo_root / "docs").rglob("*.md")):
        lines = path.read_text(encoding="utf-8", errors="ignore").splitlines()
        title = lines[0][2:] if lines and lines[0].startswith("# ") else ""
        if not title:
            continue
        if re.search(r"[A-Z]{4,}", title) and not allow.search(title):
            errors.append(f"{path.relative_to(ctx.repo_root).as_posix()}: {title}")
    return (0, "title case check passed") if not errors else (1, "\n".join(errors))


def _glossary_check(ctx: RunContext) -> tuple[int, str]:
    glossary = ctx.repo_root / "docs/_style/terms-glossary.md"
    text = glossary.read_text(encoding="utf-8", errors="ignore")
    terms: list[str] = []
    for line in text.splitlines():
        m = re.match(r"- `([^`]+)`:", line.strip())
        if m:
            terms.append(m.group(1))
    corpus = []
    for path in (ctx.repo_root / "docs").rglob("*.md"):
        if path == glossary:
            continue
        corpus.append(path.read_text(encoding="utf-8", errors="ignore"))
    full = "\n".join(corpus)
    missing = [term for term in terms if re.search(rf"\b{re.escape(term)}\b", full) is None]
    return (0, "glossary link lint passed") if not missing else (1, "glossary link lint failed; missing term usage:\n" + "\n".join(f"- {m}" for m in missing))


def _extract_code(ctx: RunContext) -> tuple[int, str]:
    fence_re = re.compile(r"```(?:bash|sh)\n(.*?)```", re.DOTALL)
    docs = ctx.repo_root / "docs"
    out = ctx.repo_root / "artifacts/docs-snippets"
    out.mkdir(parents=True, exist_ok=True)
    for old in out.glob("*.sh"):
        old.unlink()
    manifest: list[dict[str, object]] = []
    idx = 0
    for path in sorted(docs.rglob("*.md")):
        text = path.read_text(encoding="utf-8", errors="ignore")
        for block in fence_re.findall(text):
            lines = [ln.rstrip("\n") for ln in block.splitlines()]
            cleaned = [ln for ln in lines if ln.strip()]
            if not cleaned or cleaned[0].strip() != "# blessed-snippet":
                continue
            allow_network = any(ln.strip() == "# allow-network" for ln in cleaned[1:3])
            body = [ln for ln in cleaned[1:] if ln.strip() not in {"# allow-network"}]
            idx += 1
            script = out / f"snippet-{idx:03d}.sh"
            script.write_text("#!/usr/bin/env sh\nset -eu\n" + "\n".join(body) + "\n", encoding="utf-8")
            script.chmod(0o755)
            manifest.append({"id": idx, "source": str(path.relative_to(ctx.repo_root)), "path": str(script.relative_to(ctx.repo_root)), "allow_network": allow_network})
    (out / "manifest.json").write_text(json.dumps({"snippets": manifest}, indent=2) + "\n", encoding="utf-8")
    return 0, f"extracted {len(manifest)} blessed snippet(s) to {out.relative_to(ctx.repo_root)}"


def _run_blessed_snippets(ctx: RunContext) -> tuple[int, str]:
    manifest = ctx.repo_root / "artifacts/docs-snippets/manifest.json"
    network_tokens = ("curl ", "wget ", "http://", "https://", "nc ")
    if not manifest.exists():
        return 1, "snippet runner: manifest not found; run atlasctl docs extract-code first"
    data = json.loads(manifest.read_text(encoding="utf-8"))
    snippets = data.get("snippets", [])
    failures: list[str] = []
    for item in snippets:
        if not isinstance(item, dict):
            continue
        script_path = ctx.repo_root / str(item.get("path", ""))
        body = script_path.read_text(encoding="utf-8", errors="ignore")
        if not bool(item.get("allow_network", False)):
            lowered = body.lower()
            if any(token in lowered for token in network_tokens):
                failures.append(f"{item.get('path','')}: network command found without # allow-network")
                continue
        proc = subprocess.run(["sh", str(script_path)], cwd=ctx.repo_root, capture_output=True, text=True, check=False)
        if proc.returncode != 0:
            failures.append(f"{item.get('path','')}: exit={proc.returncode}\n{proc.stderr.strip()}")
    if failures:
        return 1, "snippet execution failed:\n" + "\n".join(f"- {f}" for f in failures)
    return 0, f"snippet execution passed ({len(snippets)} snippet(s))"


def _spellcheck(ctx: RunContext, path_arg: str) -> tuple[int, str]:
    exe = shutil.which("codespell")
    if not exe:
        return 2, "codespell not found in PATH"
    root = ctx.repo_root / path_arg
    targets = [root / "index.md", root / "_style"]
    cmd = [exe, "--quiet-level", "2", "--skip", "*.json,*.png,*.jpg,*.svg"]
    ignore_words = ctx.repo_root / "configs/docs/codespell-ignore-words.txt"
    if ignore_words.exists():
        cmd.extend(["--ignore-words", str(ignore_words)])
    for target in targets:
        if target.exists():
            cmd.append(str(target))
    proc = subprocess.run(cmd, cwd=ctx.repo_root, text=True, capture_output=True, check=False)
    if proc.returncode != 0:
        return proc.returncode, (proc.stdout + proc.stderr).strip() or "spellcheck failed"
    return 0, "spellcheck passed"


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


def _generate_crates_map(ctx: RunContext) -> tuple[int, str]:
    cargo = ctx.repo_root / "Cargo.toml"
    out = ctx.repo_root / "docs" / "development" / "crates-map.md"
    purpose_hints = {
        "bijux-atlas-core": "deterministic primitives, canonicalization, error types",
        "bijux-atlas-model": "domain and artifact data types",
        "bijux-atlas-policies": "runtime policy schema and validation",
        "bijux-atlas-store": "artifact backends and integrity boundaries",
        "bijux-atlas-ingest": "deterministic ingest pipeline to artifacts",
        "bijux-atlas-query": "query planning, limits, and pagination",
        "bijux-atlas-api": "wire contracts and request/response schemas",
        "bijux-atlas-server": "runtime orchestration and effectful serving",
        "bijux-atlas-cli": "plugin CLI and operational commands",
    }
    text = cargo.read_text(encoding="utf-8")
    members_match = re.search(r"members\s*=\s*\[(.*?)\]", text, re.S)
    if not members_match:
        return 1, "workspace members not found in Cargo.toml"
    crates = sorted({Path(item).name for item in re.findall(r'"([^"]+)"', members_match.group(1)) if item.startswith("crates/")})
    lines = [
        "# Crates Map",
        "",
        "- Owner: `docs-governance`",
        "",
        "## What",
        "",
        "Generated map of workspace crates and primary purpose.",
        "",
        "## Why",
        "",
        "Provides a stable navigation index for crate responsibilities.",
        "",
        "## Scope",
        "",
        "Covers workspace crates from `Cargo.toml` members under `crates/`.",
        "",
        "## Non-goals",
        "",
        "Does not replace crate-level architecture and API docs.",
        "",
        "## Contracts",
    ]
    for crate in crates:
        lines.append(f"- `{crate}`: {purpose_hints.get(crate, 'crate responsibility documented in crate docs')}.")
    lines.extend(
        [
            "",
            "## Failure modes",
            "",
            "Stale maps can hide ownership drift and boundary violations.",
            "",
            "## How to verify",
            "",
            "```bash",
            "$ atlasctl docs generate-crates-map",
            " docs crate-docs-contract-check",
            "```",
            "",
            "Expected output: crates map is regenerated and crate docs contract passes.",
            "",
            "## See also",
            "",
            "- [Crate Layout Contract](../architecture/crate-layout-contract.md)",
            "- [Crate Boundary Graph](../architecture/crate-boundary-dependency-graph.md)",
            "- [Terms Glossary](../_style/terms-glossary.md)",
            "",
        ]
    )
    out.write_text("\n".join(lines), encoding="utf-8")
    return 0, f"generated {out.relative_to(ctx.repo_root)}"


def _generate_make_targets_catalog(ctx: RunContext) -> tuple[int, str]:
    ssot = json.loads((ctx.repo_root / "configs" / "make" / "public-targets.json").read_text(encoding="utf-8"))
    owners = json.loads((ctx.repo_root / "makefiles" / "ownership.json").read_text(encoding="utf-8"))
    out_json = ctx.repo_root / "makefiles" / "targets.json"
    out_md = ctx.repo_root / "docs" / "_generated" / "make-targets.md"
    entries: list[dict[str, object]] = []
    for item in sorted(ssot.get("public_targets", []), key=lambda entry: entry["name"]):
        name = item["name"]
        meta = owners.get(name, {})
        lanes = item.get("lanes", [])
        entries.append(
            {
                "name": name,
                "description": item.get("description", ""),
                "owner": meta.get("owner", ""),
                "area": item.get("area", meta.get("area", "")),
                "lane": lanes[0] if lanes else "",
                "lanes": lanes,
            }
        )
    payload = {
        "schema_version": 1,
        "source": {"ssot": "configs/make/public-targets.json", "ownership": "makefiles/ownership.json"},
        "targets": entries,
    }
    out_json.parent.mkdir(parents=True, exist_ok=True)
    out_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    lines = [
        "# Make Targets",
        "",
        "Generated from `makefiles/targets.json`. Do not edit manually.",
        "",
        "| target | description | owner | area | lane |",
        "|---|---|---|---|---|",
    ]
    for target in entries:
        desc = str(target["description"]).replace("|", "/")
        lines.append(
            f"| `{target['name']}` | {desc} | `{target['owner']}` | `{target['area']}` | `{target['lane']}` |"
        )
    out_md.parent.mkdir(parents=True, exist_ok=True)
    out_md.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0, f"{out_json.relative_to(ctx.repo_root)}\n{out_md.relative_to(ctx.repo_root)}"


def _generate_env_vars_doc(ctx: RunContext) -> tuple[int, str]:
    contract = ctx.repo_root / "configs" / "contracts" / "env.schema.json"
    out = ctx.repo_root / "docs" / "_generated" / "env-vars.md"
    payload = json.loads(contract.read_text(encoding="utf-8"))
    keys = sorted(payload.get("allowed_env", []))
    lines = [
        "# Env Vars (Generated)",
        "",
        "Generated from `configs/contracts/env.schema.json`. Do not edit manually.",
        "",
        f"- Count: `{len(keys)}`",
        f"- Enforced prefixes: `{', '.join(payload.get('enforced_prefixes', []))}`",
        f"- Dev escape hatch: `{payload.get('dev_mode_allow_unknown_env', '')}`",
        "",
        "## Allowed Env Vars",
        "",
    ]
    lines.extend(f"- `{key}`" for key in keys)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0, str(out.relative_to(ctx.repo_root))


def _generate_config_keys_doc(ctx: RunContext) -> tuple[int, str]:
    registry = ctx.repo_root / "configs" / "config-key-registry.md"
    out = ctx.repo_root / "docs" / "_generated" / "config-keys.md"
    keys: list[str] = []
    for line in registry.read_text(encoding="utf-8").splitlines():
        item = line.strip()
        if item.startswith("- `") and item.endswith("`"):
            keys.append(item[3:-1])
    lines = [
        "# Config Keys (Generated)",
        "",
        "Generated from `configs/config-key-registry.md`. Do not edit manually.",
        "",
        f"- Count: `{len(keys)}`",
        "",
        "## Keys",
        "",
    ]
    lines.extend(f"- `{key}`" for key in keys)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0, str(out.relative_to(ctx.repo_root))


def _generate_layer_contract_doc(ctx: RunContext) -> tuple[int, str]:
    inp = ctx.repo_root / "ops" / "_meta" / "layer-contract.json"
    out = ctx.repo_root / "docs" / "_generated" / "layer-contract.md"
    contract = json.loads(inp.read_text(encoding="utf-8"))
    lines = [
        "# Layer Contract",
        "",
        f"- Contract version: `{contract['contract_version']}`",
        f"- Compatibility policy: {contract['compatibility']['policy']}",
        "",
        "## Namespaces",
    ]
    for key, val in contract["namespaces"].items():
        lines.append(f"- `{key}`: `{val}`")
    lines.extend(["", "## Services"])
    for key, val in contract["services"].items():
        lines.append(f"- `{key}`: service `{val['service_name']}`, selector `{json.dumps(val['selector'], sort_keys=True)}`")
    lines.extend(["", "## Ports"])
    for key, val in contract["ports"].items():
        lines.append(f"- `{key}`: `{json.dumps(val, sort_keys=True)}`")
    lines.extend(["", "## Labels", "- Required labels:"])
    for item in contract["labels"]["required"]:
        lines.append(f"- `{item}`")
    lines.extend(["", "## Release Metadata"])
    lines.append(f"- Required fields: `{', '.join(contract['release_metadata']['required_fields'])}`")
    lines.append(f"- Defaults: `{json.dumps(contract['release_metadata']['defaults'], sort_keys=True)}`")
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0, f"wrote {out.relative_to(ctx.repo_root)}"


def _generate_makefiles_surface(ctx: RunContext) -> tuple[int, str]:
    surface = json.loads((ctx.repo_root / "configs" / "ops" / "public-surface.json").read_text(encoding="utf-8"))
    out = ctx.repo_root / "docs" / "development" / "makefiles" / "surface.md"
    lines = [
        "# Makefiles Public Surface",
        "",
        "Generated from `configs/ops/public-surface.json`. Do not edit manually.",
        "",
        "## Core Gates",
    ]
    lines.extend(f"- `make {target}`" for target in surface.get("core_targets", []))
    lines.extend(["", "## Public Targets"])
    lines.extend(f"- `make {target}`" for target in surface.get("make_targets", []))
    lines.extend(["", "## Public Ops Run Commands"])
    lines.extend(f"- `./{cmd}`" for cmd in surface.get("ops_run_commands", []))
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0, str(out.relative_to(ctx.repo_root))


def _generate_observability_surface(ctx: RunContext) -> tuple[int, str]:
    out = ctx.repo_root / "docs" / "_generated" / "observability-surface.md"
    rendered = _render_observability_surface(ctx)
    out.write_text(rendered + "\n", encoding="utf-8")
    return 0, f"wrote {out.relative_to(ctx.repo_root)}"


def _generate_ops_contracts_doc(ctx: RunContext) -> tuple[int, str]:
    out = ctx.repo_root / "docs" / "_generated" / "ops-contracts.md"
    contracts = json.loads((ctx.repo_root / "ops" / "_meta" / "contracts.json").read_text(encoding="utf-8"))
    schemas = sorted((ctx.repo_root / "ops" / "_schemas").rglob("*.json"))
    lines = ["# Ops Contracts", "", "Generated from ops contracts and schemas.", "", "## Contract Files", ""]
    for item in contracts.get("contracts", []):
        lines.append(f"- `{item['path']}` (version `{item['version']}`)")
    lines.extend(["", "## Schemas", ""])
    lines.extend(f"- `{schema.relative_to(ctx.repo_root).as_posix()}`" for schema in schemas)
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0, str(out.relative_to(ctx.repo_root))


def _generate_ops_schema_docs(ctx: RunContext) -> tuple[int, str]:
    schemas = sorted((ctx.repo_root / "ops" / "_schemas").rglob("*.json"))
    out = ctx.repo_root / "docs" / "_generated" / "ops-schemas.md"
    lines = ["# Ops Schemas", "", "Generated from `ops/_schemas`. Do not edit manually.", ""]
    for path in schemas:
        rel = path.relative_to(ctx.repo_root).as_posix()
        try:
            payload = json.loads(path.read_text(encoding="utf-8"))
        except Exception:
            payload = {}
        required = payload.get("required", []) if isinstance(payload, dict) else []
        lines.append(f"## `{rel}`")
        lines.append("")
        if required:
            lines.append("Required keys:")
            lines.extend(f"- `{key}`" for key in required)
        else:
            lines.append("Required keys: none")
        lines.append("")
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0, str(out.relative_to(ctx.repo_root))


def _generate_ops_surface(ctx: RunContext) -> tuple[int, str]:
    surface = json.loads((ctx.repo_root / "ops" / "_meta" / "surface.json").read_text(encoding="utf-8"))
    suites = json.loads((ctx.repo_root / "ops" / "e2e" / "suites" / "suites.json").read_text(encoding="utf-8"))
    out = ctx.repo_root / "docs" / "_generated" / "ops-surface.md"
    lines = ["# Ops Surface", "", "Generated from ops manifests.", "", "## Stable Entrypoints", ""]
    lines.extend(f"- `make {target}`" for target in surface.get("entrypoints", []))
    lines.extend(["", "## E2E Suites", ""])
    for suite in suites.get("suites", []):
        capabilities = ",".join(suite.get("required_capabilities", []))
        lines.append(f"- `{suite['id']}`: capabilities=`{capabilities}`")
        for scenario in suite.get("scenarios", []):
            budget = scenario.get("budget", {})
            lines.append(
                f"- scenario `{scenario['id']}`: runner=`{scenario['runner']}`, budget(time_s={budget.get('expected_time_seconds')}, qps={budget.get('expected_qps')})"
            )
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0, str(out.relative_to(ctx.repo_root))


def _generate_repo_surface(ctx: RunContext) -> tuple[int, str]:
    exclude_dirs = {".git", ".github", ".cargo", "target", ".idea", "node_modules"}
    out = ctx.repo_root / "docs" / "_generated" / "repo-surface.md"
    surface = json.loads((ctx.repo_root / "configs" / "ops" / "public-surface.json").read_text(encoding="utf-8"))
    top_dirs = sorted(
        path.name
        for path in ctx.repo_root.iterdir()
        if path.is_dir() and path.name not in exclude_dirs and not path.name.startswith(".")
    )
    lines = ["# Repository Surface", "", "## Top-level Areas"]
    lines.extend(f"- `{name}/`" for name in top_dirs)
    lines.extend(["", "## Public Make Targets"])
    lines.extend(f"- `make {target}`" for target in surface.get("make_targets", []))
    lines.extend(["", "## Public Ops Run Commands"])
    lines.extend(f"- `./{cmd}`" for cmd in surface.get("ops_run_commands", []))
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0, f"wrote {out.relative_to(ctx.repo_root)}"


def _generate_ops_badge(ctx: RunContext) -> tuple[int, str]:
    scorecard = ctx.repo_root / "ops" / "_generated_committed" / "scorecard.json"
    out = ctx.repo_root / "docs" / "_generated" / "ops-badge.md"
    status = "unknown"
    score = 0
    if scorecard.exists():
        payload = json.loads(scorecard.read_text(encoding="utf-8"))
        status = str(payload.get("status", "unknown"))
        score = int(payload.get("score", 0))
    color = "red"
    if status == "pass":
        color = "brightgreen"
    elif status == "unknown":
        color = "lightgrey"
    lines = [
        "# Ops Badge",
        "",
        f"![ops confidence](https://img.shields.io/badge/ops%20confidence-{status}%20({score}%25)-{color})",
        "",
        f"Source: `{scorecard.relative_to(ctx.repo_root)}`",
        "",
    ]
    out.write_text("\n".join(lines), encoding="utf-8")
    return 0, f"wrote {out.relative_to(ctx.repo_root)}"


def _generate_k8s_values_doc(ctx: RunContext) -> tuple[int, str]:
    src = ctx.repo_root / "docs" / "contracts" / "CHART_VALUES.json"
    out = ctx.repo_root / "docs" / "operations" / "k8s" / "values.md"
    data = json.loads(src.read_text(encoding="utf-8"))
    keys = data.get("top_level_keys", [])
    lines = [
        "# Kubernetes Values",
        "",
        "- Owner: `bijux-atlas-operations`",
        "",
        "## What",
        "",
        "Generated summary of Helm top-level values from the chart-values contract.",
        "",
        "## Why",
        "",
        "Keeps operations docs aligned to chart values SSOT.",
        "",
        "## Scope",
        "",
        "Top-level chart values keys only.",
        "",
        "## Non-goals",
        "",
        "Does not redefine schema semantics beyond contract references.",
        "",
        "## Contracts",
    ]
    lines.extend(f"- `values.{key}`" for key in keys)
    lines.extend(
        [
            "",
            "## Failure modes",
            "",
            "Missing or stale keys can break deployments and profile docs.",
            "",
            "## How to verify",
            "",
            "```bash",
            "$ make ops-values-validate",
            "```",
            "",
            "Expected output: generated values doc and chart contract check pass.",
            "",
            "## See also",
            "",
            "- [Chart Values Contract](../../contracts/chart-values.md)",
            "- [Helm Chart Contract](chart.md)",
            "- [K8s Index](INDEX.md)",
            "- `ops-values-validate`",
            "",
        ]
    )
    out.write_text("\n".join(lines), encoding="utf-8")
    return 0, f"generated {out.relative_to(ctx.repo_root)}"


def _generate_openapi_docs(ctx: RunContext) -> tuple[int, str]:
    src_dir = ctx.repo_root / "configs" / "openapi" / "v1"
    out_dir = ctx.repo_root / "docs" / "_generated" / "openapi"
    out_dir.mkdir(parents=True, exist_ok=True)
    generated = src_dir / "openapi.generated.json"
    snapshot = src_dir / "openapi.snapshot.json"
    if not generated.exists():
        return 1, f"missing {generated.relative_to(ctx.repo_root)}"
    if not snapshot.exists():
        return 1, f"missing {snapshot.relative_to(ctx.repo_root)}"
    spec = json.loads(generated.read_text(encoding="utf-8"))
    paths = sorted(spec.get("paths", {}).keys())
    (out_dir / "openapi.generated.json").write_text(generated.read_text(encoding="utf-8"))
    (out_dir / "openapi.snapshot.json").write_text(snapshot.read_text(encoding="utf-8"))
    index = [
        "# OpenAPI Artifacts",
        "",
        "Generated from `configs/openapi/v1/`.",
        "",
        "- Canonical source: `configs/openapi/v1/openapi.generated.json`",
        "- Snapshot: `configs/openapi/v1/openapi.snapshot.json`",
        "",
        "## Paths",
        "",
    ]
    index.extend(f"- `{path}`" for path in paths)
    (out_dir / "INDEX.md").write_text("\n".join(index) + "\n", encoding="utf-8")
    return 0, "generated docs/_generated/openapi"


def _generate_chart_contract_index(ctx: RunContext) -> tuple[int, str]:
    manifest = ctx.repo_root / "ops" / "k8s" / "tests" / "manifest.json"
    out = ctx.repo_root / "docs" / "_generated" / "contracts" / "chart-contract-index.md"
    doc = json.loads(manifest.read_text(encoding="utf-8"))
    tests: list[dict[str, object]] = []
    for test in doc.get("tests", []):
        groups = set(test.get("groups", []))
        if "chart-contract" not in groups:
            continue
        script = test["script"]
        if not script.startswith("checks/"):
            continue
        tests.append(
            {
                "script": script,
                "owner": test.get("owner", "unknown"),
                "failure": ", ".join(test.get("expected_failure_modes", [])) or "n/a",
                "timeout": test.get("timeout_seconds", "n/a"),
            }
        )
    tests.sort(key=lambda item: item["script"])
    lines = [
        "# Chart Contract Index",
        "",
        "Generated from `ops/k8s/tests/manifest.json` entries tagged with `chart-contract`.",
        "",
        "| Contract Test | Owner | Timeout (s) | Failure Modes |",
        "| --- | --- | ---: | --- |",
    ]
    for test in tests:
        lines.append(f"| `{test['script']}` | `{test['owner']}` | {test['timeout']} | `{test['failure']}` |")
    lines.extend(["", "## Regenerate", "", "```bash", "atlasctl docs generate-chart-contract-index", "```", ""])
    out.write_text("\n".join(lines), encoding="utf-8")
    return 0, f"generated {out.relative_to(ctx.repo_root)} ({len(tests)} contracts)"


def _generate_k8s_install_matrix(ctx: RunContext) -> tuple[int, str]:
    src = ctx.repo_root / "artifacts" / "ops" / "k8s-install-matrix.json"
    out = ctx.repo_root / "docs" / "operations" / "k8s" / "release-install-matrix.md"
    if not src.exists():
        data = {"generated_at": "unknown", "profiles": [], "tests": []}
    else:
        data = json.loads(src.read_text(encoding="utf-8"))
    lines = [
        "# Release Install Matrix",
        "",
        "- Owner: `bijux-atlas-operations`",
        "",
        "## What",
        "",
        "Generated matrix of k8s install/test profiles from CI summaries.",
        "",
        "## Why",
        "",
        "Provides a stable compatibility view across supported chart profiles.",
        "",
        "## Contracts",
        "",
        f"Generated at: `{data.get('generated_at', 'unknown')}`",
        "",
        "Profiles:",
    ]
    lines.extend(f"- `{profile}`" for profile in data.get("profiles", []))
    lines.extend(["", "Verified test groups:"])
    lines.extend(f"- `{test}`" for test in data.get("tests", []))
    lines.extend(
        [
            "",
            "## Failure modes",
            "",
            "Missing profile/test entries indicate CI generation drift or skipped suites.",
            "",
            "## How to verify",
            "",
            "```bash",
            "$ ops/k8s/ci/install-matrix.sh",
            "$ make docs",
            "```",
            "",
            "Expected output: matrix doc updated and docs checks pass.",
            "",
            "## See also",
            "",
            "- [K8s Test Contract](k8s-test-contract.md)",
            "- [Helm Chart Contract](chart.md)",
            "- `ops-k8s-tests`",
        ]
    )
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0, f"wrote {out.relative_to(ctx.repo_root)}"


def _generate_make_targets_inventory(ctx: RunContext) -> tuple[int, str]:
    out_main = ctx.repo_root / "docs" / "development" / "make-targets.md"
    out_compat = ctx.repo_root / "docs" / "development" / "make-targets-inventory.md"
    help_text = subprocess.check_output(["python3", "-m", "atlasctl.cli", "make", "help"], cwd=ctx.repo_root, text=True)
    sections = _parse_make_help_sections(help_text)
    lines: list[str] = [
        "# Make Targets Inventory",
        "",
        "- Owner: `docs-governance`",
        "",
        "Generated from `make help`. Do not edit manually.",
        "",
    ]
    for section, targets in sections.items():
        lines.append(f"## {section}")
        lines.append("")
        lines.extend(f"- `{target}`" for target in targets)
        lines.append("")
    rendered = "\n".join(lines)
    out_main.write_text(rendered, encoding="utf-8")
    out_compat.write_text(rendered, encoding="utf-8")
    return 0, str(out_main.relative_to(ctx.repo_root))


def _generate_scripts_graph(ctx: RunContext) -> tuple[int, str]:
    mk_files = [ctx.repo_root / "Makefile", *sorted((ctx.repo_root / "makefiles").glob("*.mk"))]
    out = ctx.repo_root / "docs" / "development" / "scripts-graph.md"
    target_re = re.compile(r"^([a-zA-Z0-9_.-]+):(?:\s|$)")
    script_re = re.compile(r"(?:\./|python3\s+|python\s+)(scripts/(?:public|internal)/[^\s\"']+)")
    rows: list[tuple[str, str]] = []
    for mk in mk_files:
        current = ""
        for line in mk.read_text(encoding="utf-8").splitlines():
            if line.startswith("\t"):
                for match in script_re.finditer(line):
                    rows.append((current, match.group(1).rstrip(";")))
                continue
            m = target_re.match(line)
            if m and not line.startswith("."):
                current = m.group(1)
    rows = sorted(set((target, script) for target, script in rows if target and script))
    lines = [
        "# Scripts Graph",
        "",
        "Generated file. Do not edit by hand.",
        "",
        "| Make Target | Script |",
        "|---|---|",
    ]
    lines.extend(f"| `{target}` | `{script}` |" for target, script in rows)
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0, f"wrote {out.relative_to(ctx.repo_root)}"


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


def _check_markdown_links(ctx: RunContext) -> tuple[int, str]:
    import os
    import re

    root = ctx.repo_root
    exclude_parts = {".git", "artifacts", "target", ".cargo"}
    link_re = re.compile(r"\[[^\]]+\]\(([^)]+)\)")
    md_files: list[Path] = []

    for dirpath, dirnames, filenames in os.walk(root, topdown=True, followlinks=False):
        dirnames[:] = [d for d in dirnames if d not in exclude_parts and not Path(dirpath, d).is_symlink()]
        for filename in filenames:
            if not filename.endswith(".md"):
                continue
            path = Path(dirpath) / filename
            if exclude_parts.intersection(path.parts):
                continue
            md_files.append(path)

    errors: list[str] = []
    for md in md_files:
        text = md.read_text(encoding="utf-8")
        for target in link_re.findall(text):
            if target.startswith(("http://", "https://", "mailto:", "#")):
                continue
            rel = target.split("#", 1)[0]
            if not rel:
                continue
            resolved = (md.parent / rel).resolve()
            if not resolved.exists():
                errors.append(f"{md.relative_to(root)}: missing link target {target}")

    if errors:
        return 1, "docs markdown link-check failed:\n" + "\n".join(f"- {e}" for e in errors[:200])
    return 0, f"markdown links OK ({len(md_files)} files)"


def run_docs_command(ctx: RunContext, ns: argparse.Namespace) -> int:
    if ns.docs_cmd == "check":
        checks = DOCS_LINT_CHECKS + [
            _check(
                "docs-link-check",
                "Validate markdown links",
                None,
                "Fix broken internal links and anchors.",
                fn=_check_markdown_links,
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
                ["python3", "packages/atlasctl/src/atlasctl/checks/layout/check_ops_external_entrypoints.py"],
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
            code, output = _rewrite_legacy_terms(ctx, "docs", apply=True)
            if code != 0:
                if output:
                    print(output)
                return code
        return _run_docs_checks(ctx, DOCS_LINT_CHECKS, ns.report, ns.fail_fast, ns.emit_artifacts)

    if ns.docs_cmd == "link-check":
        code, output = _check_markdown_links(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        else:
            print("docs link-check passed")
        return code

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
        return _run_simple(ctx, ["python3", "packages/atlasctl/src/atlasctl/checks/layout/check_ops_external_entrypoints.py"], ns.report)

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
        code, output = _glossary_check(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

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

    if ns.docs_cmd == "generate-crates-map":
        code, output = _generate_crates_map(ctx)
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

    if ns.docs_cmd == "generate-make-targets-catalog":
        code, output = _generate_make_targets_catalog(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        return code

    if ns.docs_cmd == "generate-env-vars-doc":
        code, output = _generate_env_vars_doc(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        return code

    if ns.docs_cmd == "generate-config-keys-doc":
        code, output = _generate_config_keys_doc(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        return code

    if ns.docs_cmd == "generate-layer-contract-doc":
        code, output = _generate_layer_contract_doc(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        return code

    if ns.docs_cmd == "generate-makefiles-surface":
        code, output = _generate_makefiles_surface(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        return code

    if ns.docs_cmd == "generate-observability-surface":
        code, output = _generate_observability_surface(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        return code

    if ns.docs_cmd == "generate-ops-contracts-doc":
        code, output = _generate_ops_contracts_doc(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        return code

    if ns.docs_cmd == "generate-ops-schema-docs":
        code, output = _generate_ops_schema_docs(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        return code

    if ns.docs_cmd == "generate-ops-surface":
        code, output = _generate_ops_surface(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        return code

    if ns.docs_cmd == "generate-repo-surface":
        code, output = _generate_repo_surface(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        return code

    if ns.docs_cmd == "generate-ops-badge":
        code, output = _generate_ops_badge(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        return code

    if ns.docs_cmd == "generate-k8s-values-doc":
        code, output = _generate_k8s_values_doc(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        return code

    if ns.docs_cmd == "generate-openapi-docs":
        code, output = _generate_openapi_docs(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        return code

    if ns.docs_cmd == "generate-chart-contract-index":
        code, output = _generate_chart_contract_index(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        return code

    if ns.docs_cmd == "generate-k8s-install-matrix":
        code, output = _generate_k8s_install_matrix(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        return code

    if ns.docs_cmd == "generate-make-targets-inventory":
        code, output = _generate_make_targets_inventory(ctx)
        if ns.report == "json":
            print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True))
        elif output:
            print(output)
        return code

    if ns.docs_cmd == "generate-scripts-graph":
        code, output = _generate_scripts_graph(ctx)
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

    if ns.docs_cmd == "observability-acceptance-checklist":
        code, output = _check_observability_acceptance_checklist(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "script-headers-check":
        code, output = _check_script_headers(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "concept-registry-check":
        code, output = _check_concept_registry(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "concept-graph-generate":
        code, output = _generate_concept_graph(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "adr-headers-check":
        code, output = _check_adr_headers(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "broken-examples-check":
        code, output = _check_broken_examples(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "doc-filename-style-check":
        code, output = _check_doc_filename_style(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "no-placeholders-check":
        code, output = _check_no_placeholders(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "no-legacy-root-paths-check":
        code, output = _check_no_legacy_root_paths(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "mkdocs-site-links-check":
        code, output = _check_mkdocs_site_links(ctx, ns.site_dir)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "nav-order-check":
        code, output = _check_nav_order(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "docs-deterministic-check":
        code, output = _check_docs_deterministic(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "docs-make-targets-exist-check":
        code, output = _check_docs_make_targets_exist(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "critical-make-targets-referenced-check":
        code, output = _check_critical_make_targets_referenced(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "make-targets-documented-check":
        code, output = _check_make_targets_documented(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "make-targets-drift-check":
        code, output = _check_make_targets_drift(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "docker-entrypoints-check":
        code, output = _check_docker_entrypoints(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "example-configs-check":
        code, output = _check_example_configs(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "full-stack-page-check":
        code, output = _check_full_stack_page(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "k8s-docs-contract-check":
        code, output = _check_k8s_docs_contract(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "load-docs-contract-check":
        code, output = _check_load_docs_contract(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "make-help-drift-check":
        code, output = _check_make_help_drift(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "no-removed-make-targets-check":
        code, output = _check_no_removed_make_targets(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "ops-docs-make-targets-check":
        code, output = _check_ops_docs_make_targets(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "ops-observability-links-check":
        code, output = _check_ops_observability_links(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "public-targets-docs-sections-check":
        code, output = _check_public_targets_docs_sections(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "suite-id-docs-check":
        code, output = _check_suite_id_docs(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "configmap-env-docs-check":
        code, output = _check_configmap_env_docs(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "generated-contract-docs-check":
        code, output = _check_generated_contract_docs(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "lint-depth":
        code, output = _lint_depth(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "lint-doc-contracts":
        code, output = _lint_doc_contracts(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "contracts-index":
        if ns.fix:
            code, output = _generate_contracts_index_doc(ctx)
        else:
            code, output = _check_contracts_index_nav(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "runbook-map":
        if ns.fix:
            code, output = _generate_runbook_map_index(ctx)
        else:
            code, output = _check_runbook_map_registration(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
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
        code, output = _extract_code(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "run-blessed-snippets":
        code, output = _run_blessed_snippets(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "render-diagrams":
        code, output = _render_diagrams(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "lint-spelling":
        code, output = _spellcheck(ctx, ns.path)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "spellcheck":
        code, output = _spellcheck(ctx, ns.path)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "style":
        code, output = _lint_doc_status(ctx)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

    if ns.docs_cmd == "rewrite-legacy-terms":
        code, output = _rewrite_legacy_terms(ctx, ns.path, ns.apply)
        print(json.dumps({"schema_version": 1, "status": "pass" if code == 0 else "fail", "output": output}, sort_keys=True) if ns.report == "json" else output)
        return code

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
        ("generate-crates-map", "generate docs/development/crates-map.md"),
        ("generate-upgrade-guide", "generate docs/_generated/upgrade-guide.md"),
        ("generate-make-targets-catalog", "generate makefiles/targets.json and docs/_generated/make-targets.md"),
        ("generate-env-vars-doc", "generate docs/_generated/env-vars.md"),
        ("generate-config-keys-doc", "generate docs/_generated/config-keys.md"),
        ("generate-layer-contract-doc", "generate docs/_generated/layer-contract.md"),
        ("generate-makefiles-surface", "generate docs/development/makefiles/surface.md"),
        ("generate-observability-surface", "generate docs/_generated/observability-surface.md"),
        ("generate-ops-contracts-doc", "generate docs/_generated/ops-contracts.md"),
        ("generate-ops-schema-docs", "generate docs/_generated/ops-schemas.md"),
        ("generate-ops-surface", "generate docs/_generated/ops-surface.md"),
        ("generate-repo-surface", "generate docs/_generated/repo-surface.md"),
        ("generate-ops-badge", "generate docs/_generated/ops-badge.md"),
        ("generate-k8s-values-doc", "generate docs/operations/k8s/values.md"),
        ("generate-openapi-docs", "generate docs/_generated/openapi/* artifacts"),
        ("generate-chart-contract-index", "generate docs/_generated/contracts/chart-contract-index.md"),
        ("generate-k8s-install-matrix", "generate docs/operations/k8s/release-install-matrix.md"),
        ("generate-make-targets-inventory", "generate docs/development/make-targets*.md"),
        ("generate-scripts-graph", "generate docs/development/scripts-graph.md"),
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
        ("observability-acceptance-checklist", "validate observability acceptance checklist contract"),
        ("script-headers-check", "validate script header and docs script-group contract"),
        ("concept-registry-check", "validate docs concept registry and canonical ownership"),
        ("concept-graph-generate", "generate docs/_generated/concepts.md from concept registry"),
        ("adr-headers-check", "validate ADR naming and title/header contract"),
        ("broken-examples-check", "validate docs shell examples against make targets and tools"),
        ("doc-filename-style-check", "validate docs filename style policy"),
        ("no-placeholders-check", "forbid TODO/TBD placeholders outside drafts"),
        ("no-legacy-root-paths-check", "forbid legacy root ops paths in docs"),
        ("mkdocs-site-links-check", "validate rendered mkdocs site internal links"),
        ("nav-order-check", "validate top-level mkdocs nav ordering"),
        ("docs-deterministic-check", "validate docs determinism settings"),
        ("docs-make-targets-exist-check", "validate make targets referenced in docs exist"),
        ("critical-make-targets-referenced-check", "validate critical make targets are referenced in docs"),
        ("make-targets-documented-check", "validate public make targets have docs coverage"),
        ("make-targets-drift-check", "validate docs make-targets catalog is in sync"),
        ("docker-entrypoints-check", "validate docs use make docker entrypoints"),
        ("example-configs-check", "validate example config docs contract"),
        ("full-stack-page-check", "validate full-stack operations page contract"),
        ("k8s-docs-contract-check", "validate k8s docs values key contract"),
        ("load-docs-contract-check", "validate load docs suite/scenario contract"),
        ("make-help-drift-check", "validate make help output matches docs/development/make-targets.md"),
        ("no-removed-make-targets-check", "forbid references to removed public make targets"),
        ("ops-docs-make-targets-check", "validate operations docs reference make targets"),
        ("ops-observability-links-check", "validate observability docs local links resolve"),
        ("public-targets-docs-sections-check", "validate every public target appears in generated docs"),
        ("suite-id-docs-check", "forbid file-name references where suite IDs are required"),
        ("configmap-env-docs-check", "validate ATLAS_* configmap keys are documented"),
        ("generated-contract-docs-check", "validate generated contract markdown is drift-free"),
        ("lint-depth", "enforce docs depth rubric"),
        ("lint-doc-contracts", "enforce docs/contracts markdown contract"),
        ("glossary-check", "validate glossary and banned terms policy"),
        ("contracts-index", "validate or generate docs contracts index"),
        ("runbook-map", "validate or generate docs runbook map index"),
        ("evidence-policy-page", "generate docs evidence policy page"),
        ("inventory", "generate docs command inventory page"),
        ("extract-code", "extract code blocks from docs"),
        ("run-blessed-snippets", "run extracted blessed docs snippets with network guardrails"),
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
        if name == "mkdocs-site-links-check":
            cmd.add_argument("--site-dir", default="artifacts/docs/site")
        if name == "rewrite-legacy-terms":
            cmd.add_argument("--path", default="docs")
            cmd.add_argument("--apply", action="store_true")
        if name == "generate":
            cmd.add_argument("--fail-fast", action="store_true")
