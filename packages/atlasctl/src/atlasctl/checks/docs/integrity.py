from __future__ import annotations

import re
from pathlib import Path

from ...cli.surface_registry import command_registry
from ...core.effects import command_group

_MD_LINK_RE = re.compile(r"\[[^\]]+\]\(([^)]+)\)")


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
    if offenders:
        return 1, [f"package root markdown forbidden (except README.md): {name}" for name in offenders]
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
    docs_root = _docs_root(repo_root)
    scoped_roots = (docs_root / "commands", docs_root / "control-plane")
    known = {spec.name for spec in command_registry()}
    known.update({"help", "version", "env"})
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
