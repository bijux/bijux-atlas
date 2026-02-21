from __future__ import annotations

import ast
import json
import re
from pathlib import Path

from ....cli.registry import command_registry


def _tests_root(repo_root: Path) -> Path:
    return repo_root / "packages/atlasctl/tests"


def _suite_markers_path(repo_root: Path) -> Path:
    return _tests_root(repo_root) / "goldens" / "check-suite-coverage.markers.txt"


def _load_suite_markers(repo_root: Path) -> list[str]:
    path = _suite_markers_path(repo_root)
    if not path.exists():
        return []
    lines = path.read_text(encoding="utf-8", errors="ignore").splitlines()
    return [line.strip() for line in lines if line.strip() and not line.strip().startswith("#")]


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
    markers = set(_load_suite_markers(repo_root))
    repo_init = (repo_root / "packages/atlasctl/src/atlasctl/checks/repo/__init__.py").read_text(encoding="utf-8", errors="ignore")
    check_ids = sorted(set(re.findall(r'CheckDef\("([^"]+)"', repo_init)))
    missing: list[str] = []
    for marker in check_ids:
        if marker in tests_text or marker in markers:
            continue
        missing.append(marker)
    return (0 if not missing else 1), [f"check missing test/suite marker: {check_id}" for check_id in missing]


def check_suite_marker_rules(repo_root: Path) -> tuple[int, list[str]]:
    path = _suite_markers_path(repo_root)
    if not path.exists():
        return 1, [f"missing suite coverage markers file: {path.relative_to(repo_root).as_posix()}"]
    markers = _load_suite_markers(repo_root)
    if not markers:
        return 1, ["suite coverage markers file is empty"]
    errors: list[str] = []
    if markers != sorted(markers):
        errors.append("suite coverage markers must be sorted lexicographically")
    dupes = sorted({marker for marker in markers if markers.count(marker) > 1})
    for marker in dupes:
        errors.append(f"duplicate suite coverage marker: {marker}")
    repo_init = (repo_root / "packages/atlasctl/src/atlasctl/checks/repo/__init__.py").read_text(encoding="utf-8", errors="ignore")
    check_ids = set(re.findall(r'CheckDef\("([^"]+)"', repo_init))
    unknown = sorted(marker for marker in set(markers) if marker not in check_ids)
    for marker in unknown:
        errors.append(f"unknown suite coverage marker: {marker}")
    return (0 if not errors else 1), errors


def check_json_goldens_validate_schema(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    for path in sorted(_tests_root(repo_root).rglob("test_*.py")):
        text = path.read_text(encoding="utf-8", errors="ignore")
        if ".json.golden" not in text:
            continue
        if "# schema-validate-exempt" in text:
            continue
        if "_golden(" not in text and "golden =" not in text:
            continue
        if "validate(" not in text:
            offenders.append(path.relative_to(repo_root).as_posix())
    return (0 if not offenders else 1), [f"json golden test missing schema validate() call: {rel}" for rel in offenders]


def check_no_unjustified_skips(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for path in sorted(_tests_root(repo_root).rglob("test_*.py")):
        rel = path.relative_to(repo_root).as_posix()
        lines = path.read_text(encoding="utf-8", errors="ignore").splitlines()
        text = "\n".join(lines)
        if "pytest.mark.skip" not in text and "pytest.skip(" not in text:
            continue
        for idx, line in enumerate(lines, start=1):
            if "pytest.mark.skip" not in line and "pytest.skip(" not in line:
                continue
            window = lines[max(0, idx - 4) : idx + 2]
            joined = "\n".join(window)
            if "skip-justification:" not in joined or "skip-expires:" not in joined:
                errors.append(f"{rel}:{idx}: skip requires skip-justification and skip-expires markers")
    return (0 if not errors else 1), errors


def check_no_conflicting_json_goldens(repo_root: Path) -> tuple[int, list[str]]:
    golden_dir = _tests_root(repo_root) / "goldens"
    schema_to_files: dict[str, list[str]] = {}
    for golden in sorted(golden_dir.glob("*.json.golden")):
        text = golden.read_text(encoding="utf-8", errors="ignore").strip()
        if not text:
            continue
        try:
            payload = json.loads(text)
        except Exception:
            continue
        schema_name = str(payload.get("schema_name", "")).strip()
        if not schema_name:
            continue
        schema_to_files.setdefault(schema_name, []).append(golden.name)
    errors: list[str] = []
    for schema_name, files in sorted(schema_to_files.items()):
        if len(files) > 1:
            errors.append(f"conflicting goldens for schema {schema_name}: {sorted(files)}")
    return (0 if not errors else 1), errors


def check_duplicate_contract_assertions(repo_root: Path) -> tuple[int, list[str]]:
    contract_assertions: dict[tuple[str, str], list[str]] = {}
    for path in sorted(_tests_root(repo_root).rglob("test_*.py")):
        rel = path.relative_to(repo_root).as_posix()
        tree = ast.parse(path.read_text(encoding="utf-8", errors="ignore"), filename=rel)
        for node in tree.body:
            if not isinstance(node, ast.FunctionDef) or not node.name.startswith("test_"):
                continue
            schemas: set[str] = set()
            for sub in ast.walk(node):
                if isinstance(sub, ast.Call) and isinstance(sub.func, ast.Name) and sub.func.id == "validate":
                    if sub.args and isinstance(sub.args[0], ast.Constant) and isinstance(sub.args[0].value, str):
                        schemas.add(sub.args[0].value)
            for schema in sorted(schemas):
                key = (rel, schema)
                contract_assertions.setdefault(key, []).append(node.name)
    errors: list[str] = []
    for (rel, schema), tests in sorted(contract_assertions.items()):
        if len(tests) > 1:
            errors.append(f"{rel}: duplicate assertions for contract `{schema}` in tests {sorted(tests)}")
    return (0 if not errors else 1), errors
