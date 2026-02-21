from __future__ import annotations

import ast
from pathlib import Path

_SRC_ROOT = Path("packages/atlasctl/src/atlasctl")
_CHECKS_ROOT = _SRC_ROOT / "checks"
_CHECK_IMPL_TRANSITION_ALLOWLIST = (
    "packages/atlasctl/src/atlasctl/load/checks/",
    "packages/atlasctl/src/atlasctl/observability/",
)


def _iter_check_impl_files(repo_root: Path) -> list[Path]:
    return sorted((repo_root / _SRC_ROOT).rglob("check_*.py"))


def check_checks_canonical_location(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    for path in _iter_check_impl_files(repo_root):
        rel = path.relative_to(repo_root).as_posix()
        if rel.startswith(_CHECKS_ROOT.as_posix() + "/"):
            continue
        if any(rel.startswith(prefix) for prefix in _CHECK_IMPL_TRANSITION_ALLOWLIST):
            continue
        offenders.append(rel)
    if offenders:
        return 1, [f"check implementation must live under {_CHECKS_ROOT.as_posix()}/: {rel}" for rel in offenders]
    return 0, []


def check_check_impl_no_cli_imports(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    for path in sorted((repo_root / _CHECKS_ROOT).rglob("check_*.py")):
        rel = path.relative_to(repo_root).as_posix()
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=rel)
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                if any(alias.name == "atlasctl.cli" or alias.name.startswith("atlasctl.cli.") for alias in node.names):
                    offenders.append(rel)
                    break
            elif isinstance(node, ast.ImportFrom):
                if node.level == 0 and node.module and (node.module == "atlasctl.cli" or node.module.startswith("atlasctl.cli.")):
                    offenders.append(rel)
                    break
    if offenders:
        return 1, [f"check implementation must not import CLI modules directly: {rel}" for rel in sorted(set(offenders))]
    return 0, []
