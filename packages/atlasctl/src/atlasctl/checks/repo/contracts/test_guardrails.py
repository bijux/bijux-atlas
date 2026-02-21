from __future__ import annotations

import ast
import re
from pathlib import Path

from ....cli.registry import command_registry


def _tests_root(repo_root: Path) -> Path:
    return repo_root / "packages/atlasctl/tests"


def check_test_ownership_tags(repo_root: Path) -> tuple[int, list[str]]:
    roots = _tests_root(repo_root)
    errors: list[str] = []
    for path in sorted(roots.rglob("test_*.py")):
        rel = path.relative_to(repo_root).as_posix()
        lines = path.read_text(encoding="utf-8", errors="ignore").splitlines()
        tag_line = next((line.strip() for line in lines[:10] if line.strip().startswith("# test-domain:")), "")
        if tag_line:
            continue
        parts = path.relative_to(roots).parts
        if len(parts) >= 2:
            continue
        errors.append(f"test ownership tag missing: {rel}")
    return (0 if not errors else 1), errors


def check_test_duplicate_expectations(repo_root: Path) -> tuple[int, list[str]]:
    roots = _tests_root(repo_root)
    names: dict[str, list[tuple[str, str]]] = {}
    for path in sorted(roots.rglob("test_*.py")):
        rel = path.relative_to(repo_root).as_posix()
        tree = ast.parse(path.read_text(encoding="utf-8", errors="ignore"), filename=rel)
        for node in tree.body:
            if isinstance(node, ast.FunctionDef) and node.name.startswith("test_"):
                body_sig = ast.dump(ast.Module(body=node.body, type_ignores=[]), annotate_fields=False, include_attributes=False)
                names.setdefault(node.name, []).append((rel, body_sig))
    offenders: list[str] = []
    for name, rows in sorted(names.items()):
        if len(rows) <= 1:
            continue
        sigs = {sig for _, sig in rows}
        if len(sigs) > 1:
            offenders.append(f"duplicate test name with conflicting expectations {name}: {[rel for rel, _ in rows]}")
    return (0 if not offenders else 1), offenders


def check_command_test_coverage(repo_root: Path) -> tuple[int, list[str]]:
    tests_text = "\n".join(path.read_text(encoding="utf-8", errors="ignore") for path in _tests_root(repo_root).rglob("test_*.py"))
    docs_text = ""
    docs_cli = repo_root / "docs/_generated/cli.md"
    if docs_cli.exists():
        docs_text = docs_cli.read_text(encoding="utf-8", errors="ignore")
    missing: list[str] = []
    for spec in command_registry():
        if spec.name in {"legacy", "compat"}:
            continue
        token = f"\"{spec.name}\""
        if token not in tests_text and f" {spec.name} " not in tests_text and spec.name not in docs_text:
            missing.append(spec.name)
    return (0 if not missing else 1), [f"command missing explicit test coverage hint: {name}" for name in missing]


def check_check_test_coverage(repo_root: Path) -> tuple[int, list[str]]:
    tests_text = "\n".join(path.read_text(encoding="utf-8", errors="ignore") for path in _tests_root(repo_root).rglob("*.py"))
    goldens_text = "\n".join(path.read_text(encoding="utf-8", errors="ignore") for path in (_tests_root(repo_root) / "goldens").rglob("*") if path.is_file())
    repo_init = (repo_root / "packages/atlasctl/src/atlasctl/checks/repo/__init__.py").read_text(encoding="utf-8", errors="ignore")
    check_ids = sorted(set(re.findall(r'CheckDef\("([^"]+)"', repo_init)))
    missing: list[str] = []
    for marker in check_ids:
        if marker in tests_text or marker in goldens_text:
            continue
        missing.append(marker)
    return (0 if not missing else 1), [f"check missing test/suite marker: {check_id}" for check_id in missing]


def check_legacy_parity_tests_present(repo_root: Path) -> tuple[int, list[str]]:
    legacy_dir = repo_root / "packages/atlasctl/src/atlasctl/legacy"
    if not legacy_dir.exists():
        return 0, []
    legacy_files = [p for p in legacy_dir.rglob("*.py") if "__pycache__" not in p.as_posix()]
    if not legacy_files:
        return 0, []
    tests_root = _tests_root(repo_root)
    parity_tests = sorted(tests_root.rglob("test_legacy*.py"))
    if parity_tests:
        return 0, []
    return 1, ["legacy modules exist but no legacy parity tests were found under packages/atlasctl/tests"]
