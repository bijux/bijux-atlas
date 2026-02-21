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
