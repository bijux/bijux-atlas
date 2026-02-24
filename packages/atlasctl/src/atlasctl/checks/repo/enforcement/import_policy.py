from __future__ import annotations

import ast
import os
import py_compile
import subprocess
import sys
from pathlib import Path


_SRC_ROOT = Path("packages/atlasctl/src/atlasctl")
_COMMAND_IMPORT_ALLOW_PREFIXES = ("core", "contracts", "checks", "reporting", "adapters", "ops", "registry")
_COMMAND_IMPORT_ALLOW_EXACT = {"errors", "exit_codes", "run_context"}
_CHECK_IMPORT_ALLOW_PREFIXES = ("core", "contracts", "reporting", "adapters", "checks")
_CHECK_IMPORT_ALLOW_EXACT = {"errors", "exit_codes", "run_context"}
_CLI_IMPORT_ALLOW_PREFIXES = ("commands", "core", "cli")
_CLI_IMPORT_ALLOW_EXACT = {"errors", "exit_codes", "network_guard"}
_COLD_IMPORT_BUDGET_MS = 250.0
_CHECKS_TO_COMMANDS_ALLOWLIST = {
    "packages/atlasctl/src/atlasctl/checks/tools/make_enforcement.py",
    "packages/atlasctl/src/atlasctl/checks/layout/ops/validation/check_ops_indexes_generated.py",
    "packages/atlasctl/src/atlasctl/checks/repo/__init__.py",
    "packages/atlasctl/src/atlasctl/checks/tools/reachability.py",
}
_COMMANDS_TO_CHECKS_ALLOWLIST = {
    "packages/atlasctl/src/atlasctl/commands/check/run.py",
    "packages/atlasctl/src/atlasctl/commands/ops/load/contracts/validate_suite_manifest.py",
    "packages/atlasctl/src/atlasctl/commands/ops/load/checks/check_abuse_scenarios_required.py",
    "packages/atlasctl/src/atlasctl/commands/ops/load/checks/check_perf_baselines.py",
    "packages/atlasctl/src/atlasctl/commands/ops/load/checks/check_pinned_queries_lock.py",
    "packages/atlasctl/src/atlasctl/commands/ops/load/checks/check_runbook_suite_names.py",
}
_CHECKS_TO_CLI_ALLOWLIST = {
    "packages/atlasctl/src/atlasctl/checks/repo/native/modules/repo_checks_make_and_layout.py",
}


def _iter_py_files(repo_root: Path) -> list[Path]:
    return sorted((repo_root / _SRC_ROOT).rglob("*.py"))


def _module_prefix(node: ast.ImportFrom | ast.Import) -> str | None:
    if isinstance(node, ast.Import):
        name = node.names[0].name
        if not name.startswith("atlasctl."):
            return None
        return name.split(".", 2)[1]
    if node.level != 0 or not node.module or not node.module.startswith("atlasctl."):
        return None
    return node.module.split(".", 2)[1]


def _first_atlasctl_import(node: ast.ImportFrom | ast.Import) -> tuple[str, int] | None:
    if isinstance(node, ast.Import):
        for alias in node.names:
            if alias.name.startswith("atlasctl."):
                return alias.name, node.lineno
        return None
    if node.level == 0 and node.module and node.module.startswith("atlasctl."):
        return node.module, node.lineno
    return None


def check_internal_import_boundaries(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    for path in _iter_py_files(repo_root):
        rel = path.relative_to(repo_root).as_posix()
        if "/internal/" in rel:
            continue
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=rel)
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                if any(alias.name.startswith("atlasctl.internal") for alias in node.names):
                    offenders.append(rel)
                    break
            elif isinstance(node, ast.ImportFrom):
                if node.level == 0 and node.module and node.module.startswith("atlasctl.internal"):
                    offenders.append(rel)
                    break
    if offenders:
        return 1, [f"forbidden import of atlasctl.internal outside internal/: {rel}" for rel in sorted(set(offenders))]
    return 0, []


def check_no_modern_imports_from_legacy(repo_root: Path) -> tuple[int, list[str]]:
    legacy_prefix = "atlasctl." + "legacy"
    offenders: list[str] = []
    for path in _iter_py_files(repo_root):
        rel = path.relative_to(repo_root).as_posix()
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=rel)
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                if any(alias.name.startswith(legacy_prefix) for alias in node.names):
                    offenders.append(rel)
                    break
            elif isinstance(node, ast.ImportFrom):
                if node.level == 0 and node.module and node.module.startswith(legacy_prefix):
                    offenders.append(rel)
                    break
    if offenders:
        return 1, [f"forbidden import of {legacy_prefix} from modern module: {rel}" for rel in sorted(set(offenders))]
    return 0, []


def check_no_legacy_obs_imports_in_modern(repo_root: Path) -> tuple[int, list[str]]:
    legacy_obs_prefix = "atlasctl." + "legacy" + ".obs"
    offenders: list[str] = []
    for path in _iter_py_files(repo_root):
        rel = path.relative_to(repo_root).as_posix()
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=rel)
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                if any(alias.name.startswith(legacy_obs_prefix) for alias in node.names):
                    offenders.append(rel)
                    break
            elif isinstance(node, ast.ImportFrom):
                if node.level == 0 and node.module and node.module.startswith(legacy_obs_prefix):
                    offenders.append(rel)
                    break
    if offenders:
        return 1, [f"forbidden modern import of {legacy_obs_prefix}: {rel}" for rel in sorted(set(offenders))]
    return 0, []


def check_forbidden_deprecated_namespaces(repo_root: Path) -> tuple[int, list[str]]:
    def _is_deprecated_namespace(name: str) -> bool:
        return (
            name == "atlasctl.check"
            or name.startswith("atlasctl.check.")
            or name == "atlasctl.report"
            or name.startswith("atlasctl.report.")
            or name == "atlasctl.obs"
            or name.startswith("atlasctl.obs.")
        )

    offenders: list[str] = []
    for path in _iter_py_files(repo_root):
        rel = path.relative_to(repo_root).as_posix()
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=rel)
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                names = [alias.name for alias in node.names]
                if any(_is_deprecated_namespace(name) for name in names):
                    offenders.append(rel)
                    break
            elif isinstance(node, ast.ImportFrom):
                if node.level == 0 and node.module and _is_deprecated_namespace(node.module):
                    offenders.append(rel)
                    break
    if offenders:
        return 1, [f"forbidden deprecated namespace import detected: {rel}" for rel in sorted(set(offenders))]
    return 0, []


def check_forbidden_deprecated_namespace_dirs(repo_root: Path) -> tuple[int, list[str]]:
    base = repo_root / _SRC_ROOT
    forbidden = ("check", "report", "obs")
    offenders = [str((base / name).relative_to(repo_root).as_posix()) for name in forbidden if (base / name).exists()]
    if offenders:
        return 1, [f"forbidden deprecated namespace directory present: {path}" for path in sorted(offenders)]
    return 0, []


def check_no_legacy_module_paths(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    root = repo_root / _SRC_ROOT
    for path in sorted(root.rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        if "/legacy/" in rel:
            offenders.append(rel)
    if offenders:
        return 1, [f"legacy module path forbidden pre-1.0: {rel}" for rel in offenders]
    return 0, []


def check_forbidden_core_integration_dir(repo_root: Path) -> tuple[int, list[str]]:
    path = repo_root / _SRC_ROOT / "core" / "integration"
    if path.exists():
        return 1, [f"forbidden deprecated core integration namespace present: {path.relative_to(repo_root).as_posix()}"]
    return 0, []


def check_contract_import_boundaries(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    for path in _iter_py_files(repo_root):
        rel = path.relative_to(repo_root).as_posix()
        if not ("/commands/" in rel or "/checks/" in rel):
            continue
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=rel)
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                names = [alias.name for alias in node.names]
                if any(name.startswith("atlasctl.core.schema") for name in names):
                    offenders.append(rel)
                    break
            elif isinstance(node, ast.ImportFrom):
                if node.level == 0 and node.module and node.module.startswith("atlasctl.core.schema"):
                    offenders.append(rel)
                    break
    if offenders:
        return 1, [f"commands/checks must import schemas from atlasctl.contracts, not core.schema: {rel}" for rel in sorted(set(offenders))]
    return 0, []


def check_runcontext_single_builder(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    core_context = "packages/atlasctl/src/atlasctl/core/context.py"
    for path in _iter_py_files(repo_root):
        rel = path.relative_to(repo_root).as_posix()
        if rel == core_context or "/tests/" in rel:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if "RunContext" + "(" in text:
            offenders.append(rel)
    if offenders:
        return 1, [f"RunContext must be built only in core/context.py via from_args: {rel}" for rel in sorted(set(offenders))]
    return 0, []


def check_command_import_lint(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    for path in sorted((repo_root / _SRC_ROOT / "commands").rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        if rel.endswith("/legacy.py"):
            continue
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=rel)
        bad_edges: set[str] = set()
        for node in ast.walk(tree):
            if not isinstance(node, (ast.Import, ast.ImportFrom)):
                continue
            prefix = _module_prefix(node)
            if prefix is None:
                continue
            imported = _first_atlasctl_import(node)
            if prefix in _COMMAND_IMPORT_ALLOW_EXACT:
                continue
            if prefix in _COMMAND_IMPORT_ALLOW_PREFIXES:
                continue
            if imported and (
                imported[0].startswith("atlasctl.commands._shared")
                or imported[0].startswith("atlasctl.commands.ops._shared")
                or imported[0].startswith("atlasctl.commands.ops.tools")
                or imported[0].startswith("atlasctl.commands.ops.orchestrate._wrappers")
                or imported[0].startswith("atlasctl.commands.ops.e2e.realdata._common")
                or imported[0].startswith("atlasctl.commands.ops.e2e.realdata.generate_snapshots")
                or imported[0].startswith("atlasctl.commands.ops.runtime_modules.layer_contract")
                or imported[0].startswith("atlasctl.commands.ops.k8s.runtime_bridge")
                or imported[0].startswith("atlasctl.commands.ops.stack.faults")
            ):
                continue
            if imported is None:
                continue
            bad_edges.add(f"{rel}:{imported[1]} import-chain commands -> {imported[0]}")
        if bad_edges:
            offenders.extend(sorted(bad_edges))
    return (0 if not offenders else 1), offenders


def check_checks_import_lint(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    for path in sorted((repo_root / _SRC_ROOT / "checks").rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        if "/legacy/" in rel:
            continue
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=rel)
        bad_edges: set[str] = set()
        for node in ast.walk(tree):
            if not isinstance(node, (ast.Import, ast.ImportFrom)):
                continue
            prefix = _module_prefix(node)
            if prefix is None:
                continue
            imported = _first_atlasctl_import(node)
            if prefix in _CHECK_IMPORT_ALLOW_EXACT:
                continue
            if prefix in _CHECK_IMPORT_ALLOW_PREFIXES:
                continue
            if imported and imported[0].startswith("atlasctl.ops.contracts"):
                continue
            if imported and imported[0].startswith("atlasctl.commands.ops.runtime_modules.index_generator"):
                continue
            if imported is None:
                continue
            bad_edges.add(f"{rel}:{imported[1]} import-chain checks -> {imported[0]}")
        if bad_edges:
            offenders.extend(sorted(bad_edges))
    return (0 if not offenders else 1), offenders


def check_core_no_command_imports(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    for path in sorted((repo_root / _SRC_ROOT / "core").rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=rel)
        for node in ast.walk(tree):
            imported = _first_atlasctl_import(node) if isinstance(node, (ast.Import, ast.ImportFrom)) else None
            if imported is None:
                continue
            target = imported[0]
            if target.startswith("atlasctl.commands.") or target == "atlasctl.commands" or target.startswith("atlasctl.cli.") or target == "atlasctl.cli":
                offenders.append(f"{rel}:{imported[1]} import-chain core -> {target}")
    return (0 if not offenders else 1), sorted(set(offenders))


def check_checks_no_cli_imports(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    for path in sorted((repo_root / _SRC_ROOT / "checks").rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        if rel in _CHECKS_TO_CLI_ALLOWLIST:
            continue
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=rel)
        for node in ast.walk(tree):
            imported = _first_atlasctl_import(node) if isinstance(node, (ast.Import, ast.ImportFrom)) else None
            if imported is None:
                continue
            target = imported[0]
            if target == "atlasctl.cli" or target.startswith("atlasctl.cli."):
                offenders.append(f"{rel}:{imported[1]} import-chain checks -> {target}")
    return (0 if not offenders else 1), sorted(set(offenders))


def check_checks_no_command_imports(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    for path in sorted((repo_root / _SRC_ROOT / "checks").rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        if rel in _CHECKS_TO_COMMANDS_ALLOWLIST:
            continue
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=rel)
        for node in ast.walk(tree):
            imported = _first_atlasctl_import(node) if isinstance(node, (ast.Import, ast.ImportFrom)) else None
            if imported is None:
                continue
            target = imported[0]
            if target == "atlasctl.commands" or target.startswith("atlasctl.commands."):
                offenders.append(f"{rel}:{imported[1]} import-chain checks -> {target}")
    return (0 if not offenders else 1), sorted(set(offenders))


def check_commands_no_check_impl_imports(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    for path in sorted((repo_root / _SRC_ROOT / "commands").rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        if rel in _COMMANDS_TO_CHECKS_ALLOWLIST:
            continue
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=rel)
        for node in ast.walk(tree):
            imported = _first_atlasctl_import(node) if isinstance(node, (ast.Import, ast.ImportFrom)) else None
            if imported is None:
                continue
            target = imported[0]
            if target == "atlasctl.checks" or target.startswith("atlasctl.checks."):
                offenders.append(f"{rel}:{imported[1]} import-chain commands -> {target}")
    return (0 if not offenders else 1), sorted(set(offenders))


def check_commands_check_no_domain_imports(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    check_commands_root = repo_root / _SRC_ROOT / "commands" / "check"
    for path in sorted(check_commands_root.rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=rel)
        for node in ast.walk(tree):
            imported = _first_atlasctl_import(node) if isinstance(node, (ast.Import, ast.ImportFrom)) else None
            if imported is None:
                continue
            target = imported[0]
            if target.startswith("atlasctl.checks.domains."):
                offenders.append(f"{rel}:{imported[1]} commands/check must not import domain checks directly: {target}")
    return (0 if not offenders else 1), sorted(set(offenders))


def check_checks_no_commands_check_imports(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    for path in sorted((repo_root / _SRC_ROOT / "checks").rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=rel)
        for node in ast.walk(tree):
            imported = _first_atlasctl_import(node) if isinstance(node, (ast.Import, ast.ImportFrom)) else None
            if imported is None:
                continue
            target = imported[0]
            if target == "atlasctl.commands.check" or target.startswith("atlasctl.commands.check."):
                offenders.append(f"{rel}:{imported[1]} checks must not import commands/check modules: {target}")
    return (0 if not offenders else 1), sorted(set(offenders))


def check_cli_import_scope(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    for path in sorted((repo_root / _SRC_ROOT / "cli").rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=rel)
        for node in ast.walk(tree):
            if not isinstance(node, (ast.Import, ast.ImportFrom)):
                continue
            prefix = _module_prefix(node)
            if prefix is None:
                continue
            if prefix in _CLI_IMPORT_ALLOW_EXACT or prefix in _CLI_IMPORT_ALLOW_PREFIXES:
                continue
            imported = _first_atlasctl_import(node)
            if imported is None:
                continue
            offenders.append(f"{rel}:{imported[1]} import-chain cli -> {imported[0]}")
    return (0 if not offenders else 1), sorted(set(offenders))


def check_cli_render_boundary(repo_root: Path) -> tuple[int, list[str]]:
    cli_root = repo_root / _SRC_ROOT / "cli"
    if not cli_root.exists():
        return 1, [f"missing cli package root: {(_SRC_ROOT / 'cli').as_posix()}"]
    canonical = (cli_root / "render.py").relative_to(repo_root).as_posix()
    offenders: list[str] = []
    for path in sorted(cli_root.rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        if "def render_error(" in text and rel != canonical:
            offenders.append(f"{rel}: render_error must be defined only in {canonical}")
    return (0 if not offenders else 1), offenders


def check_registry_definition_boundary(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    forbidden = ("atlasctl.commands", "atlasctl.cli")
    for path in sorted((repo_root / _SRC_ROOT / "registry").rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=rel)
        for node in ast.walk(tree):
            imported = _first_atlasctl_import(node) if isinstance(node, (ast.Import, ast.ImportFrom)) else None
            if imported is None:
                continue
            target = imported[0]
            if any(target == base or target.startswith(f"{base}.") for base in forbidden):
                offenders.append(f"{rel}:{imported[1]} import-chain registry -> {target}")
    return (0 if not offenders else 1), sorted(set(offenders))


def check_compileall_gate(repo_root: Path) -> tuple[int, list[str]]:
    src_root = repo_root / "packages/atlasctl/src/atlasctl"
    if not src_root.exists():
        return 1, ["missing atlasctl source root for compile check"]
    failures: list[str] = []
    for path in sorted(src_root.rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        try:
            py_compile.compile(str(path), doraise=True)
        except py_compile.PyCompileError as exc:
            message = str(exc).strip() or "compile error"
            failures.append(f"{rel}: {message}")
    return (0 if not failures else 1), failures


def check_import_smoke(repo_root: Path) -> tuple[int, list[str]]:
    env = dict(os.environ)
    src_path = str(repo_root / "packages/atlasctl/src")
    existing = env.get("PYTHONPATH", "")
    env["PYTHONPATH"] = f"{src_path}:{existing}" if existing else src_path
    proc = subprocess.run(
        [sys.executable, "-c", "import atlasctl"],
        cwd=repo_root,
        text=True,
        capture_output=True,
        check=False,
        env=env,
    )
    if proc.returncode == 0:
        return 0, []
    return 1, [proc.stderr.strip() or proc.stdout.strip() or "import atlasctl failed"]


def check_cold_import_budget(repo_root: Path) -> tuple[int, list[str]]:
    env = dict(os.environ)
    src_path = str(repo_root / "packages/atlasctl/src")
    existing = env.get("PYTHONPATH", "")
    env["PYTHONPATH"] = f"{src_path}:{existing}" if existing else src_path
    proc = subprocess.run(
        [
            sys.executable,
            "-c",
            "import time; t=time.perf_counter(); import atlasctl; print((time.perf_counter()-t)*1000)",
        ],
        cwd=repo_root,
        text=True,
        capture_output=True,
        check=False,
        env=env,
    )
    if proc.returncode != 0:
        return 1, [proc.stderr.strip() or proc.stdout.strip() or "cold import budget check failed"]
    try:
        elapsed_ms = float((proc.stdout or "").strip())
    except ValueError:
        return 1, [f"unexpected cold import timing output: {(proc.stdout or '').strip()}"]
    if elapsed_ms <= _COLD_IMPORT_BUDGET_MS:
        return 0, []
    return 1, [f"cold import budget exceeded: {elapsed_ms:.2f}ms > {_COLD_IMPORT_BUDGET_MS:.2f}ms"]
