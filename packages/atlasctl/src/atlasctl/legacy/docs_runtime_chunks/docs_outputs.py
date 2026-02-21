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
