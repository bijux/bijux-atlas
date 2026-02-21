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
