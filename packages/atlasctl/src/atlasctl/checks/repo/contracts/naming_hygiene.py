from __future__ import annotations

from pathlib import Path
import re


_SRC_ROOT = Path("packages/atlasctl/src/atlasctl")
_CONTRACTS_ALLOWED_MODULES = {
    "__init__.py",
    "catalog.py",
    "checks.py",
    "command.py",
    "generators.py",
    "ids.py",
    "json.py",
    "output.py",
    "output_base.py",
    "validate.py",
    "validate_self.py",
}


def _resolve_src_root(repo_root: Path) -> Path:
    repo_style = repo_root / _SRC_ROOT
    if repo_style.exists():
        return repo_style
    package_style = repo_root / "src/atlasctl"
    if package_style.exists():
        return package_style
    return repo_style


def _find_named_modules(repo_root: Path, filename: str) -> list[Path]:
    root = _resolve_src_root(repo_root)
    if not root.exists():
        return []
    return sorted(path for path in root.rglob(filename))


def check_single_registry_module(repo_root: Path) -> tuple[int, list[str]]:
    paths = _find_named_modules(repo_root, "registry.py")
    expected = "checks/registry.py"
    found = [path.relative_to(_resolve_src_root(repo_root)).as_posix() for path in paths]
    if found == [expected]:
        return 0, []
    return 1, [f"registry.py must exist only at {expected}; found: {', '.join(found) or '<none>'}"]


def check_single_runner_module(repo_root: Path) -> tuple[int, list[str]]:
    paths = _find_named_modules(repo_root, "runner.py")
    expected = "checks/runner.py"
    found = [path.relative_to(_resolve_src_root(repo_root)).as_posix() for path in paths]
    if found == [expected]:
        return 0, []
    return 1, [f"runner.py must exist only at {expected}; found: {', '.join(found) or '<none>'}"]


def check_command_module_cli_intent(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    for rel in _find_named_modules(repo_root, "command.py"):
        text = rel.read_text(encoding="utf-8", errors="ignore")
        # command.py modules are CLI entry shims: parser + run are required.
        if "configure_" not in text or "run_" not in text:
            offenders.append(rel.relative_to(_resolve_src_root(repo_root)).as_posix())
    if offenders:
        return 1, [f"command.py must be CLI entry module (configure_* + run_*): {path}" for path in offenders]
    return 0, []


def check_no_wildcard_exports(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    src_root = _resolve_src_root(repo_root)
    wildcard_pattern = re.compile(r"^\s*(?:from\s+\S+\s+import\s+\*|import\s+\*)", re.MULTILINE)
    for path in sorted(src_root.rglob("*.py")):
        rel = path.relative_to(src_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        if wildcard_pattern.search(text):
            offenders.append(rel)
    if offenders:
        return 1, [f"wildcard import/export is forbidden outside public surface: {path}" for path in offenders]
    return 0, []


def check_public_api_doc_exists(repo_root: Path) -> tuple[int, list[str]]:
    path = repo_root / "packages/atlasctl/docs/PUBLIC_API.md"
    if not path.exists():
        path = repo_root / "docs/PUBLIC_API.md"
    if path.exists():
        return 0, []
    return 1, ["missing packages/atlasctl/docs/PUBLIC_API.md"]


def check_contracts_namespace_purpose(repo_root: Path) -> tuple[int, list[str]]:
    contracts_root = _resolve_src_root(repo_root) / "contracts"
    offenders: list[str] = []
    for path in sorted(contracts_root.glob("*.py")):
        if path.name not in _CONTRACTS_ALLOWED_MODULES:
            offenders.append(path.relative_to(contracts_root.parent).as_posix())
    if offenders:
        return 1, [f"contracts namespace must contain schema/validation modules only: {path}" for path in offenders]
    return 0, []


def check_layout_domain_alias_cleanup(repo_root: Path) -> tuple[int, list[str]]:
    deprecated = _resolve_src_root(repo_root) / "checks/layout/registry.py"
    if deprecated.exists():
        return 1, [f"deprecated layout alias module exists: {deprecated.relative_to(repo_root).as_posix()}"]
    return 0, []
