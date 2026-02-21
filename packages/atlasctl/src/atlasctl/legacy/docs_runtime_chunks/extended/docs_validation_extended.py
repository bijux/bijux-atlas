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
