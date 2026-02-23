#!/usr/bin/env python3
from __future__ import annotations

import ast
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
OPS_CMDS = ROOT / "packages/atlasctl/src/atlasctl/commands/ops"
ALLOWED_PREFIXES = ("atlasctl.commands.ops", "atlasctl.core", "atlasctl.contracts", "atlasctl.registry", "atlasctl.reporting", "atlasctl.commands._shared", "atlasctl.ops")
FORBIDDEN_PREFIXES = ("atlasctl.checks.layout", "atlasctl.checks.domains.policies", "atlasctl.commands.policies")
TEMP_ALLOWED_IMPORTS: dict[str, tuple[str, ...]] = {
    # Transitional shim modules still wrapping atlasctl-native ops checks.
    "packages/atlasctl/src/atlasctl/commands/ops/load/checks/check_abuse_scenarios_required.py": (
        "atlasctl.checks.domains.ops.contracts",
    ),
    "packages/atlasctl/src/atlasctl/commands/ops/load/checks/check_perf_baselines.py": (
        "atlasctl.checks.domains.ops.contracts",
    ),
    "packages/atlasctl/src/atlasctl/commands/ops/load/checks/check_pinned_queries_lock.py": (
        "atlasctl.checks.domains.ops.contracts",
    ),
    "packages/atlasctl/src/atlasctl/commands/ops/load/checks/check_runbook_suite_names.py": (
        "atlasctl.checks.domains.ops.contracts",
    ),
    "packages/atlasctl/src/atlasctl/commands/ops/load/contracts/validate_suite_manifest.py": (
        "atlasctl.checks.domains.ops.contracts",
    ),
}


def _imports(path: Path) -> list[str]:
    tree = ast.parse(path.read_text(encoding='utf-8', errors='ignore'))
    out: list[str] = []
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            out.extend(a.name for a in node.names)
        elif isinstance(node, ast.ImportFrom):
            if node.level and not node.module:
                continue
            if node.level:
                # relative imports stay within commands.ops package and are allowed
                continue
            if node.module:
                out.append(node.module)
    return out


def main() -> int:
    errs: list[str] = []
    for path in OPS_CMDS.rglob('*.py'):
        rel = path.relative_to(ROOT).as_posix()
        temp_allowed = TEMP_ALLOWED_IMPORTS.get(rel, ())
        for imp in _imports(path):
            if any(imp.startswith(p) for p in FORBIDDEN_PREFIXES):
                errs.append(f"{rel}: forbidden ops command import `{imp}`")
                continue
            if any(imp.startswith(p) for p in temp_allowed):
                continue
            if imp.startswith('atlasctl.') and not any(imp.startswith(p) for p in ALLOWED_PREFIXES):
                errs.append(f"{rel}: non-whitelisted atlasctl import `{imp}`")
    if errs:
        print('\n'.join(sorted(set(errs))))
        return 1
    print('ops command import policy OK')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
