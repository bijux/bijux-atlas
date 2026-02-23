from __future__ import annotations

import ast
from pathlib import Path

try:
    import tomllib  # type: ignore[attr-defined]
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib  # type: ignore[no-redef]

_SRC_ROOT = Path("packages/atlasctl/src/atlasctl")
_CHECKS_ROOT = _SRC_ROOT / "checks"
_REGISTRY_TOML = _CHECKS_ROOT / "REGISTRY.toml"
_CHECK_IMPL_TRANSITION_ALLOWLIST = (
    "packages/atlasctl/src/atlasctl/load/checks/",
    "packages/atlasctl/src/atlasctl/commands/ops/load/checks/",
    "packages/atlasctl/src/atlasctl/commands/ops/observability/",
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
    shell_checks = sorted(path.relative_to(repo_root).as_posix() for path in (repo_root / _CHECKS_ROOT).rglob("*.sh"))
    if shell_checks:
        offenders.extend(f"shell check must be migrated to python: {rel}" for rel in shell_checks)
        offenders.append("migration completeness failed: .sh checks remain under packages/atlasctl/src/atlasctl/checks")
    registry_path = repo_root / _REGISTRY_TOML
    if registry_path.exists():
        payload = tomllib.loads(registry_path.read_text(encoding="utf-8"))
        seen: set[str] = set()
        duplicates: set[str] = set()
        for row in payload.get("checks", []):
            if not isinstance(row, dict):
                continue
            check_id = str(row.get("id", "")).strip()
            if not check_id:
                continue
            if check_id in seen:
                duplicates.add(check_id)
            seen.add(check_id)
        if duplicates:
            offenders.extend(f"duplicate check id in registry: {check_id}" for check_id in sorted(duplicates))
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
