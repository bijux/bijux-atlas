from __future__ import annotations

import json
from pathlib import Path

from ...commands.policies.runtime.dead_modules import analyze_dead_modules


ALLOWED_UNLISTED = {"__init__.py", "legacy_native.py", "native_lint.py"}
_DEAD_MODULE_ALLOWLIST = Path("configs/policy/dead-modules-allowlist.json")


def check_repo_check_modules_registered(repo_root: Path) -> tuple[int, list[str]]:
    checks_dir = repo_root / "packages/atlasctl/src/atlasctl/checks/repo"
    if not checks_dir.exists():
        # Repo checks were consolidated under checks/tools/repo_domain.
        return 0, []
    init_path = checks_dir / "__init__.py"
    if not init_path.exists():
        return 1, [f"missing registry module: {init_path.relative_to(repo_root).as_posix()}"]
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
    legacy_ns = "atlasctl." + "legacy"
    offenders: list[str] = []
    for path in sorted(src_root.rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        if legacy_ns in text:
            offenders.append(rel)
    if offenders:
        return 1, [f"legacy reachability violation (importer exists): {rel}" for rel in offenders]
    return 0, []


def check_dead_modules_report_runs(repo_root: Path) -> tuple[int, list[str]]:
    payload = analyze_dead_modules(repo_root)
    if not isinstance(payload, dict):
        return 1, ["dead modules analysis returned non-object payload"]
    if "candidate_count" not in payload or "candidates" not in payload:
        return 1, ["dead modules analysis payload missing candidate_count/candidates"]
    return 0, []


def check_dead_module_reachability_allowlist(repo_root: Path) -> tuple[int, list[str]]:
    allowlist_path = repo_root / _DEAD_MODULE_ALLOWLIST
    if not allowlist_path.exists():
        return 1, [f"missing dead modules allowlist: {_DEAD_MODULE_ALLOWLIST.as_posix()}"]
    payload = json.loads(allowlist_path.read_text(encoding="utf-8"))
    allowed = {str(item.get("path", "")).strip() for item in payload.get("allow", []) if isinstance(item, dict)}
    candidates = {str(item.get("path", "")).strip() for item in analyze_dead_modules(repo_root).get("candidates", []) if isinstance(item, dict)}
    errors: list[str] = []
    for path in sorted(candidates - allowed):
        errors.append(f"dead module candidate missing allowlist entry: {path}")
    for path in sorted(allowed - candidates):
        errors.append(f"stale dead module allowlist entry: {path}")
    return (0 if not errors else 1), errors
