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
