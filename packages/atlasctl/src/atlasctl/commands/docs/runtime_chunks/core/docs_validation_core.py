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
