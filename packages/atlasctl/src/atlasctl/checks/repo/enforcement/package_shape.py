from __future__ import annotations

from pathlib import Path


def check_no_nested_same_name_packages(repo_root: Path) -> tuple[int, list[str]]:
    src_root = repo_root / "packages/atlasctl/src/atlasctl"
    offenders: list[str] = []
    for path in sorted(src_root.rglob("*")):
        if not path.is_dir():
            continue
        parts = path.relative_to(src_root).parts
        for left, right in zip(parts, parts[1:]):
            if left == right:
                offenders.append(path.relative_to(repo_root).as_posix())
                break
    if offenders:
        return 1, [f"nested same-name package segment is forbidden: {item}" for item in offenders]
    return 0, []


def check_layout_domain_readmes(repo_root: Path) -> tuple[int, list[str]]:
    layout_root = repo_root / "packages/atlasctl/src/atlasctl/checks/layout"
    required_domains = (
        "root",
        "artifacts",
        "makefiles",
        "ops",
        "scripts",
        "docs",
        "workflows",
        "contracts",
        "governance",
        "public_surface",
        "hygiene",
        "policies",
        "orphans",
        "scenarios",
        "shell",
    )
    missing: list[str] = []
    for domain in required_domains:
        readme = layout_root / domain / "README.md"
        if not readme.exists():
            missing.append(readme.relative_to(repo_root).as_posix())
    if missing:
        return 1, [f"missing layout domain README: {path}" for path in missing]
    return 0, []


def check_layout_no_legacy_imports(repo_root: Path) -> tuple[int, list[str]]:
    layout_root = repo_root / "packages/atlasctl/src/atlasctl/checks/layout"
    offenders: list[str] = []
    for path in sorted(layout_root.rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        if "/legacy/" in rel:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if "atlasctl.legacy" in text or "from ...legacy" in text or "from ....legacy" in text:
            offenders.append(rel)
    if offenders:
        return 1, [f"layout checks must not import atlasctl.legacy: {path}" for path in offenders]
    return 0, []
