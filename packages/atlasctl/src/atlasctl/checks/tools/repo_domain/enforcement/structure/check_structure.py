from __future__ import annotations

import ast
from pathlib import Path

_SRC_ROOT = Path("packages/atlasctl/src/atlasctl")
_CHECKS_ROOT = _SRC_ROOT / "checks"
_CHECK_IMPL_TRANSITION_ALLOWLIST = (
    "packages/atlasctl/src/atlasctl/load/checks/",
    "packages/atlasctl/src/atlasctl/commands/ops/load/checks/",
    "packages/atlasctl/src/atlasctl/commands/ops/observability/",
    "packages/atlasctl/src/atlasctl/observability/",
)
_MAIN_ENTRY = Path("packages/atlasctl/src/atlasctl/__main__.py")
_CLI_MAIN = Path("packages/atlasctl/src/atlasctl/cli/main.py")
_CLI_MAIN_MAX_LOC = 300


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


def check_main_entrypoint_calls_app_main(repo_root: Path) -> tuple[int, list[str]]:
    path = repo_root / _MAIN_ENTRY
    if not path.exists():
        return 1, [f"missing main entrypoint: {_MAIN_ENTRY.as_posix()}"]
    text = path.read_text(encoding="utf-8")
    required = (
        "from .app.main import main",
        "if __name__ == \"__main__\":",
        "raise SystemExit(main())",
    )
    errors = [f"main entrypoint missing required line: {line}" for line in required if line not in text]
    return (0 if not errors else 1), errors


def check_cli_main_loc_budget(repo_root: Path) -> tuple[int, list[str]]:
    path = repo_root / _CLI_MAIN
    if not path.exists():
        return 1, [f"missing cli main module: {_CLI_MAIN.as_posix()}"]
    loc = len(path.read_text(encoding="utf-8").splitlines())
    if loc > _CLI_MAIN_MAX_LOC:
        return 1, [f"cli main LOC budget exceeded: {loc} > {_CLI_MAIN_MAX_LOC}"]
    return 0, []


def check_registry_generated_readonly(repo_root: Path) -> tuple[int, list[str]]:
    try:
        from atlasctl.checks.registry_legacy.ssot import generate_registry_json
    except Exception as exc:  # noqa: BLE001
        return 1, [f"unable to load registry generator: {exc}"]
    try:
        _out, changed = generate_registry_json(repo_root, check_only=True)
    except Exception as exc:  # noqa: BLE001
        return 1, [f"registry generation check failed: {exc}"]
    if changed:
        return 1, ["REGISTRY.generated.json drift detected; run ./bin/atlasctl dev regen-registry"]
    return 0, []
