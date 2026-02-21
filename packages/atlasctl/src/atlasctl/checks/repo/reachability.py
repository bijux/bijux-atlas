from __future__ import annotations

from pathlib import Path


ALLOWED_UNLISTED = {"__init__.py", "legacy_native.py"}


def check_repo_check_modules_registered(repo_root: Path) -> tuple[int, list[str]]:
    checks_dir = repo_root / "packages/atlasctl/src/atlasctl/checks/repo"
    init_path = checks_dir / "__init__.py"
    text = init_path.read_text(encoding="utf-8")
    errors: list[str] = []
    for path in sorted(checks_dir.glob("*.py")):
        if path.name in ALLOWED_UNLISTED:
            continue
        module_text = path.read_text(encoding="utf-8", errors="ignore")
        if "def check_" not in module_text:
            continue
        module_name = path.stem
        if f".{module_name} import " not in text:
            errors.append(f"unregistered repo check module: {path.relative_to(repo_root)}")
    return (0 if not errors else 1), errors


def check_no_legacy_importers(repo_root: Path) -> tuple[int, list[str]]:
    src_root = repo_root / "packages/atlasctl/src"
    offenders: list[str] = []
    for path in sorted(src_root.rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        if "atlasctl.legacy" in text:
            offenders.append(rel)
    if offenders:
        return 1, [f"legacy reachability violation (importer exists): {rel}" for rel in offenders]
    return 0, []
