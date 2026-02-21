from __future__ import annotations

import re
from pathlib import Path


_MD_LINK_RE = re.compile(r"\[[^\]]+\]\(([^)]+)\)")


def check_no_package_root_markdown_except_readme(repo_root: Path) -> tuple[int, list[str]]:
    package_root = repo_root / "packages/atlasctl"
    offenders = [
        p.name
        for p in sorted(package_root.glob("*.md"))
        if p.name != "README.md"
    ]
    if offenders:
        return 1, [f"package root markdown forbidden (except README.md): {name}" for name in offenders]
    return 0, []


def check_docs_links_exist(repo_root: Path) -> tuple[int, list[str]]:
    docs_root = repo_root / "packages/atlasctl/docs"
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
    docs_root = repo_root / "packages/atlasctl/docs"
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
