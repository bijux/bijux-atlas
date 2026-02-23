from __future__ import annotations

import json
import re
from pathlib import Path

from ....core.process import run_command
from ....core.effects import command_group

_MD_LINK_RE = re.compile(r"\[[^\]]+\]\(([^)]+)\)")
_HEADING_RE = re.compile(r"^(#{1,6})\s+\S")
_LEGACY_CLI_PATTERNS = (
    re.compile(r"\bpython3?\s+-m\s+atlasctl\.cli\b"),
    re.compile(r"\./bin/bijux-atlas\b"),
)


def _package_root(repo_root: Path) -> Path:
    candidate = repo_root / "packages/atlasctl"
    if candidate.exists():
        return candidate
    return repo_root


def _docs_root(repo_root: Path) -> Path:
    return _package_root(repo_root) / "docs"


def check_no_package_root_markdown_except_readme(repo_root: Path) -> tuple[int, list[str]]:
    package_root = _package_root(repo_root)
    offenders = [
        p.name
        for p in sorted(package_root.glob("*.md"))
        if p.name != "README.md"
    ]
    allowed = {"ARCHITECTURE.md"}
    offenders = [name for name in offenders if name not in allowed]
    if offenders:
        return 1, [f"package root markdown forbidden (except README.md and allowlist): {name}" for name in offenders]
    return 0, []


def check_docs_links_exist(repo_root: Path) -> tuple[int, list[str]]:
    docs_root = _docs_root(repo_root)
    errors: list[str] = []
    for md in sorted(docs_root.rglob("*.md")):
        text = md.read_text(encoding="utf-8", errors="ignore")
        for raw_target in _MD_LINK_RE.findall(text):
            target = raw_target.strip()
            if not target or target.startswith("#"):
                continue
            if "://" in target or target.startswith("mailto:"):
                continue
            target_path = target.split("#", 1)[0]
            if not target_path:
                continue
            resolved = (md.parent / target_path).resolve()
            if not resolved.exists():
                rel_md = md.relative_to(repo_root).as_posix()
                errors.append(f"{rel_md}: broken link target `{target}`")
    return (0 if not errors else 1), errors


def check_docs_index_complete(repo_root: Path) -> tuple[int, list[str]]:
    docs_root = _docs_root(repo_root)
    index = docs_root / "index.md"
    if not index.exists():
        return 1, ["missing packages/atlasctl/docs/index.md"]
    index_text = index.read_text(encoding="utf-8", errors="ignore")
    errors: list[str] = []
    for md in sorted(docs_root.rglob("*.md")):
        if md == index:
            continue
        rel = md.relative_to(docs_root).as_posix()
        if rel not in index_text:
            errors.append(f"docs index missing entry for `{rel}`")
    return (0 if not errors else 1), errors


def check_command_group_docs_pages(repo_root: Path) -> tuple[int, list[str]]:
    from ....cli.surface_registry import command_registry

    groups_dir = _docs_root(repo_root) / "commands/groups"
    required_groups = sorted({command_group(spec.name) for spec in command_registry()})
    errors: list[str] = []
    for group in required_groups:
        path = groups_dir / f"{group}.md"
        if not path.exists():
            errors.append(f"missing command-group docs page: {path.relative_to(repo_root).as_posix()}")
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if "## Examples" not in text:
            errors.append(f"{path.relative_to(repo_root).as_posix()}: missing `## Examples` section")
    return (0 if not errors else 1), errors


def check_docs_registry_command_drift(repo_root: Path) -> tuple[int, list[str]]:
    from ....cli.surface_registry import command_registry

    docs_root = _docs_root(repo_root)
    scoped_roots = (docs_root / "commands", docs_root / "control-plane")
    known = {spec.name for spec in command_registry()}
    known.update({"help", "version", "env", "product"})
    token_re = re.compile(r"\batlasctl\s+([a-z][a-z0-9-]*)\b")
    errors: list[str] = []
    files: list[Path] = []
    for root in scoped_roots:
        if root.exists():
            files.extend(sorted(root.rglob("*.md")))
    for md in files:
        rel = md.relative_to(repo_root).as_posix()
        text = md.read_text(encoding="utf-8", errors="ignore")
        for cmd in sorted(set(token_re.findall(text))):
            if cmd in known:
                continue
            errors.append(f"{rel}: unknown command in docs: atlasctl {cmd}")
    return (0 if not errors else 1), errors


def check_stable_command_examples_in_group_docs(repo_root: Path) -> tuple[int, list[str]]:
    from ....cli.surface_registry import command_registry

    groups_dir = _docs_root(repo_root) / "commands/groups"
    errors: list[str] = []
    for spec in command_registry():
        if spec.internal or not spec.stable:
            continue
        group = command_group(spec.name)
        path = groups_dir / f"{group}.md"
        if not path.exists():
            errors.append(f"{spec.name}: missing group docs file {path.relative_to(repo_root).as_posix()}")
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if f"`{spec.name}`" not in text:
            errors.append(f"{path.relative_to(repo_root).as_posix()}: missing command `{spec.name}` entry")
        if f"atlasctl {spec.name}" not in text:
            errors.append(f"{path.relative_to(repo_root).as_posix()}: missing example usage for `{spec.name}`")
    return (0 if not errors else 1), errors


def check_migration_docs_not_stale(repo_root: Path) -> tuple[int, list[str]]:
    migration_root = _docs_root(repo_root) / "migration"
    if not migration_root.exists():
        return 0, []
    banned = (
        "legacy parity shim",
        "parallel legacy path",
        "keep legacy indefinitely",
    )
    errors: list[str] = []
    for md in sorted(migration_root.rglob("*.md")):
        rel = md.relative_to(repo_root).as_posix()
        text = md.read_text(encoding="utf-8", errors="ignore").lower()
        for term in banned:
            if term in text:
                errors.append(f"{rel}: stale migration wording `{term}`")
    return (0 if not errors else 1), errors


def check_docs_check_id_drift(repo_root: Path) -> tuple[int, list[str]]:
    docs_root = _docs_root(repo_root)
    checks_root = _package_root(repo_root) / "src/atlasctl/checks"
    known: set[str] = set()
    pat = re.compile(r'CheckDef\("([^"]+)"')
    for path in sorted(checks_root.rglob("__init__.py")):
        text = path.read_text(encoding="utf-8", errors="ignore")
        known.update(pat.findall(text))
    token_re = re.compile(r"\bcheck:([a-z0-9_.-]+)\b")
    errors: list[str] = []
    for md in sorted(docs_root.rglob("*.md")):
        rel = md.relative_to(repo_root).as_posix()
        text = md.read_text(encoding="utf-8", errors="ignore")
        for check_id in sorted(set(token_re.findall(text))):
            if check_id not in known:
                errors.append(f"{rel}: unknown check id reference `check:{check_id}`")
    return (0 if not errors else 1), errors


def check_docs_nav_references_exist(repo_root: Path) -> tuple[int, list[str]]:
    nav_file = repo_root / "mkdocs.yml"
    docs_root = repo_root / "docs"
    if not nav_file.exists():
        return 0, []
    errors: list[str] = []
    nav_text = nav_file.read_text(encoding="utf-8", errors="ignore")
    for raw in re.findall(r":\s*([A-Za-z0-9_./-]+\.md)\s*$", nav_text, flags=re.MULTILINE):
        if raw.startswith("http"):
            continue
        path = (docs_root / raw).resolve()
        if not path.exists():
            errors.append(f"mkdocs.yml: missing docs nav target `{raw}`")
    return (0 if not errors else 1), sorted(set(errors))


def check_docs_no_orphans(repo_root: Path) -> tuple[int, list[str]]:
    docs_root = _docs_root(repo_root)
    allow = {
        "README.md",
        "index.md",
        "PUBLIC_API.md",
    }
    allow_prefixes = (
        "_generated/",
        "_meta/",
        "_drafts/",
    )
    referenced: set[str] = set()
    for md in sorted(docs_root.rglob("*.md")):
        rel = md.relative_to(docs_root).as_posix()
        text = md.read_text(encoding="utf-8", errors="ignore")
        for raw_target in _MD_LINK_RE.findall(text):
            target = raw_target.strip()
            if not target or target.startswith("#") or "://" in target or target.startswith("mailto:"):
                continue
            target_path = target.split("#", 1)[0]
            if not target_path:
                continue
            resolved = (md.parent / target_path).resolve()
            try:
                rel_target = resolved.relative_to(docs_root.resolve()).as_posix()
            except ValueError:
                continue
            if rel_target.endswith(".md"):
                referenced.add(rel_target)
    index = docs_root / "index.md"
    if index.exists():
        text = index.read_text(encoding="utf-8", errors="ignore")
        for md in sorted(docs_root.rglob("*.md")):
            rel = md.relative_to(docs_root).as_posix()
            if rel in text:
                referenced.add(rel)
    errors: list[str] = []
    for md in sorted(docs_root.rglob("*.md")):
        rel = md.relative_to(docs_root).as_posix()
        if rel in allow:
            continue
        if rel.startswith(allow_prefixes):
            continue
        if rel not in referenced:
            errors.append(f"orphan docs file not linked from docs graph: {rel}")
    return (0 if not errors else 1), errors


def check_docs_no_legacy_cli_invocation(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    docs_roots = [
        repo_root / "docs",
        _docs_root(repo_root),
    ]
    seen: set[str] = set()
    for root in docs_roots:
        if not root.exists():
            continue
        for md in sorted(root.rglob("*.md")):
            rel = md.relative_to(repo_root).as_posix()
            if rel in seen:
                continue
            seen.add(rel)
            text = md.read_text(encoding="utf-8", errors="ignore")
            for lineno, line in enumerate(text.splitlines(), start=1):
                for pattern in _LEGACY_CLI_PATTERNS:
                    if pattern.search(line):
                        errors.append(f"{rel}:{lineno}: use `./bin/atlasctl` instead of legacy invocation")
                        break
    return (0 if not errors else 1), errors


def _render_commands_index(repo_root: Path) -> str:
    from ....cli.surface_registry import command_registry

    groups = sorted({command_group(spec.name) for spec in command_registry()})
    lines = [
        "# Atlasctl Commands",
        "",
        "Generated from CLI registry (`src/atlasctl/cli/surface_registry.py`).",
        "",
        "## Command Groups",
        "",
    ]
    for group in groups:
        lines.append(f"- [{group.capitalize()}](groups/{group}.md)")
    lines.extend(["", "## Stable Commands", ""])
    for spec in sorted(command_registry(), key=lambda item: item.name):
        if spec.internal:
            continue
        lines.append(f"- `{spec.name}`")
    lines.extend(
        [
            "",
            "Internal commands are hidden from default help output.",
            "Use `atlasctl help --include-internal --json` to inspect them explicitly.",
            "",
        ]
    )
    return "\n".join(lines)


def _render_checks_index() -> str:
    checks_root = Path(__file__).resolve().parents[1]
    domains: dict[str, int] = {}
    pat = re.compile(r'CheckDef\("([^"]+)",\s*"([^"]+)"')
    for path in sorted(checks_root.rglob("__init__.py")):
        text = path.read_text(encoding="utf-8", errors="ignore")
        for _, domain in pat.findall(text):
            domains[domain] = domains.get(domain, 0) + 1
    lines = [
        "# Check Domains",
        "",
        "Generated from check registry (`src/atlasctl/checks/registry.py`).",
        "",
        "## Domains",
        "",
    ]
    for domain, count in sorted(domains.items()):
        lines.append(f"- `{domain}` ({count})")
    lines.extend(["", "Use `atlasctl check list --json` for machine-readable inventory.", ""])
    return "\n".join(lines)


def _render_suites_index(repo_root: Path) -> str:
    from ....suite.command import load_suites
    from ....suite.manifests import load_first_class_suites

    default_suite, suites = load_suites(repo_root)
    first_class = load_first_class_suites()
    lines = [
        "# Suites",
        "",
        "Generated from suite registries (`pyproject.toml` + `src/atlasctl/registry/suites_catalog.json`).",
        "",
        f"- Default suite: `{default_suite}`",
        "",
        "## Configured Suites",
        "",
    ]
    for name in sorted(suites):
        spec = suites[name]
        lines.append(f"- `{name}`: includes={list(spec.includes)} items={len(spec.items)} complete={spec.complete}")
    lines.extend(["", "## First-Class Suites", ""])
    for name in sorted(first_class):
        spec = first_class[name]
        lines.append(f"- `{name}`: checks={len(spec.check_ids)} markers={list(spec.markers)}")
    lines.append("")
    return "\n".join(lines)


def check_docs_registry_indexes(repo_root: Path) -> tuple[int, list[str]]:
    docs_root = _docs_root(repo_root)
    expected = {
        "commands/index.md": _render_commands_index(repo_root).strip(),
        "checks/index.md": _render_checks_index().strip(),
        "control-plane/suites.md": _render_suites_index(repo_root).strip(),
    }
    errors: list[str] = []
    for rel, want in expected.items():
        path = docs_root / rel
        if not path.exists():
            errors.append(f"missing generated docs index: {path.relative_to(repo_root).as_posix()}")
            continue
        got = path.read_text(encoding="utf-8", errors="ignore").strip()
        if got != want:
            errors.append(f"docs registry index drift: {path.relative_to(repo_root).as_posix()} (run `atlasctl docs generate-registry-indexes --report text`)")
    return (0 if not errors else 1), errors


def check_docs_ci_lane_mapping(repo_root: Path) -> tuple[int, list[str]]:
    doc = _docs_root(repo_root) / "control-plane" / "ci-lane-mapping.md"
    if not doc.exists():
        return 1, [f"missing CI lane mapping doc: {doc.relative_to(repo_root).as_posix()}"]
    text = doc.read_text(encoding="utf-8", errors="ignore")
    required_doc_markers = [
        "`ci`",
        "`control-plane-conformance`",
        "`suite-product`",
        "`suite-ops-fast`",
        "`bypass-burn-down`",
    ]
    errors = [f"CI lane mapping doc missing marker {marker}" for marker in required_doc_markers if marker not in text]
    ci_workflow = repo_root / ".github" / "workflows" / "ci.yml"
    if ci_workflow.exists():
        wf = ci_workflow.read_text(encoding="utf-8", errors="ignore")
        for job in ("ci:", "control-plane-conformance:", "suite-product:", "suite-ops-fast:"):
            if job not in wf:
                errors.append(f".github/workflows/ci.yml missing expected job `{job[:-1]}`")
    bypass_wf = repo_root / ".github" / "workflows" / "bypass-burn-down.yml"
    if not bypass_wf.exists():
        errors.append("missing .github/workflows/bypass-burn-down.yml referenced by CI lane mapping")
    return (0 if not errors else 1), errors


def check_docs_new_command_workflow(repo_root: Path) -> tuple[int, list[str]]:
    proc = run_command(
        ["git", "diff", "--name-only", "HEAD~1", "HEAD"],
        cwd=repo_root,
    )
    changed = [line.strip() for line in proc.stdout.splitlines() if line.strip()] if proc.code == 0 else []
    if "packages/atlasctl/src/atlasctl/cli/surface_registry.py" not in changed:
        return 0, []
    required = [
        "packages/atlasctl/docs/commands/index.md",
        "packages/atlasctl/pyproject.toml",
    ]
    errors = [f"new command workflow requires updating `{path}`" for path in required if path not in changed]
    test_touched = any(path.startswith("packages/atlasctl/tests/") for path in changed)
    if not test_touched:
        errors.append("new command workflow requires at least one test update under packages/atlasctl/tests/")
    return (0 if not errors else 1), errors


def check_docs_ownership_metadata(repo_root: Path) -> tuple[int, list[str]]:
    docs_root = _docs_root(repo_root)
    meta = docs_root / "_meta" / "owners.json"
    if not meta.exists():
        return 1, [f"missing docs ownership metadata: {meta.relative_to(repo_root).as_posix()}"]
    payload = json.loads(meta.read_text(encoding="utf-8"))
    owners = payload.get("owners", {})
    if not isinstance(owners, dict):
        return 1, [f"{meta.relative_to(repo_root).as_posix()}: owners must be an object"]
    major: set[str] = set()
    for path in docs_root.iterdir():
        if path.name.startswith("_") or path.name == "index.md":
            continue
        if path.is_dir():
            major.add(path.name)
    errors = [f"missing docs owner mapping for area `{area}`" for area in sorted(major) if area not in owners]
    return (0 if not errors else 1), errors


def check_docs_lint_style(repo_root: Path) -> tuple[int, list[str]]:
    docs_root = _docs_root(repo_root)
    errors: list[str] = []
    for md in sorted(docs_root.rglob("*.md")):
        rel = md.relative_to(repo_root).as_posix()
        if "/_generated/" in rel:
            continue
        lines = md.read_text(encoding="utf-8", errors="ignore").splitlines()
        for idx, line in enumerate(lines, 1):
            raw = line.rstrip("\n")
            if len(raw) > 160 and not raw.startswith("|"):
                errors.append(f"{rel}:{idx}: line length exceeds 160 chars")
            low = raw.lower()
            if "todo" in low or "tbd" in low:
                errors.append(f"{rel}:{idx}: TODO/TBD placeholders are forbidden")
        for idx, line in enumerate(lines, 1):
            if line.startswith("#") and not _HEADING_RE.match(line):
                errors.append(f"{rel}:{idx}: invalid heading format")
    return (0 if not errors else 1), errors


def check_docs_no_placeholder_release_docs(repo_root: Path) -> tuple[int, list[str]]:
    docs_root = _docs_root(repo_root)
    errors: list[str] = []
    for md in sorted(docs_root.rglob("*.md")):
        rel = md.relative_to(repo_root).as_posix().lower()
        if "/_drafts/" in rel:
            continue
        if "placeholder" in rel or "/tmp" in rel:
            errors.append(f"placeholder doc path forbidden for release: {rel}")
    return (0 if not errors else 1), errors
