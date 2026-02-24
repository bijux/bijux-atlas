from __future__ import annotations

import ast
import hashlib
import json
import os
import re
from datetime import date
from pathlib import Path
from typing import Iterable

from ...core.process import run_command
from ..model import CheckCategory, CheckDef
from ..tools.root_policy import load_root_policy


REGISTRY_PATH = "packages/atlasctl/src/atlasctl/checks/REGISTRY.generated.json"
REQUIRED_OWNERS = "configs/make/ownership.json"
REQUIRED_DOCS = {"docs/checks/registry.md", "docs/INDEX.md"}
REQUIRED_GOLDENS = {
    "packages/atlasctl/tests/goldens/check/check-list.json.golden",
    "packages/atlasctl/tests/goldens/check/checks-tree.json.golden",
    "packages/atlasctl/tests/goldens/check/checks-owners.json.golden",
}


def _changed_files(repo_root: Path) -> set[str]:
    proc = run_command(
        ["git", "diff", "--name-only", "HEAD"],
        cwd=repo_root,
    )
    if proc.code != 0:
        return set()
    return {line.strip() for line in (proc.stdout or "").splitlines() if line.strip()}


def _gate(repo_root: Path, *, required: set[str], label: str) -> tuple[int, list[str]]:
    if str(os.environ.get("CI", "")).lower() not in {"1", "true", "yes"}:
        return 0, []
    changed = _changed_files(repo_root)
    if REGISTRY_PATH not in changed:
        return 0, []
    if any(path in changed for path in required):
        return 0, []
    needed = ", ".join(sorted(required))
    return 1, [f"registry changed; require {label} update: {needed}"]


def check_registry_change_requires_owner_update(repo_root: Path) -> tuple[int, list[str]]:
    return _gate(repo_root, required={REQUIRED_OWNERS}, label="owners")


def check_registry_change_requires_docs_update(repo_root: Path) -> tuple[int, list[str]]:
    return _gate(repo_root, required=REQUIRED_DOCS, label="docs index")


def check_registry_change_requires_golden_update(repo_root: Path) -> tuple[int, list[str]]:
    return _gate(repo_root, required=REQUIRED_GOLDENS, label="goldens")

_CANONICAL_RE = re.compile(r"^checks_[a-z0-9]+_[a-z0-9]+_[a-z0-9_]+$")
_CATALOG_PATH = Path("packages/atlasctl/src/atlasctl/registry/checks_catalog.json")
_DOCS_META_PATH = Path("packages/atlasctl/docs/_meta/checks-registry.txt")
_COUNT_BUDGET_PATH = Path("configs/policy/checks-count-budget.json")
_SHAPE_BUDGET_PATH = Path("configs/policy/checks-shape-budget.json")
_FORBIDDEN_ADJECTIVES_CONFIG = Path("configs/policy/forbidden-adjectives.json")
_REQUIRED_PROFILE_TAGS = ("ci", "dev", "internal", "lint", "slow", "fast")


def check_registry_integrity(repo_root: Path) -> tuple[int, list[str]]:
    from ..registry import generate_registry_json

    try:
        _out, changed = generate_registry_json(repo_root, check_only=True)
    except Exception as exc:
        return 1, [f"checks registry integrity failed: {exc}"]
    if changed:
        return 1, ["checks registry generated json drift detected; run `./bin/atlasctl gen checks-registry`"]
    return 0, []


def _catalog_rows(repo_root: Path) -> list[dict[str, object]]:
    payload = json.loads((repo_root / _CATALOG_PATH).read_text(encoding="utf-8"))
    rows = payload.get("checks", [])
    return rows if isinstance(rows, list) else []


def _entries(repo_root: Path):
    from ..registry import load_registry_entries

    return load_registry_entries(repo_root)


def _forbidden_terms(repo_root: Path) -> tuple[str, ...]:
    path = repo_root / _FORBIDDEN_ADJECTIVES_CONFIG
    if not path.exists():
        return ()
    payload = json.loads(path.read_text(encoding="utf-8"))
    raw = payload.get("terms", [])
    if not isinstance(raw, list):
        return ()
    out = tuple(sorted({str(term).strip().lower() for term in raw if str(term).strip()}))
    return out


def check_registry_all_checks_have_canonical_id(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for entry in _entries(repo_root):
        if _CANONICAL_RE.match(entry.id) is None:
            errors.append(f"{entry.id}: invalid canonical id format")
    return (1, errors) if errors else (0, [])


def check_registry_no_duplicate_canonical_id(repo_root: Path) -> tuple[int, list[str]]:
    seen: set[str] = set()
    dupes: list[str] = []
    for entry in _entries(repo_root):
        if entry.id in seen:
            dupes.append(entry.id)
        seen.add(entry.id)
    return (1, sorted(set(dupes))) if dupes else (0, [])


def check_registry_canonical_id_matches_module_path(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for entry in _entries(repo_root):
        if not (entry.module.startswith("atlasctl.checks.") or entry.module.startswith("atlasctl.commands.")):
            errors.append(f"{entry.id}: check module must live under atlasctl.checks.* or atlasctl.commands.* ({entry.module})")
    return (1, errors) if errors else (0, [])


def check_registry_owners_required(repo_root: Path) -> tuple[int, list[str]]:
    errors = [f"{entry.id}: owner is required" for entry in _entries(repo_root) if not entry.owner.strip()]
    return (1, errors) if errors else (0, [])


def check_registry_speed_required(repo_root: Path) -> tuple[int, list[str]]:
    allowed = {"fast", "slow", "nightly"}
    errors = [f"{entry.id}: speed must be one of {sorted(allowed)}" for entry in _entries(repo_root) if entry.speed not in allowed]
    return (1, errors) if errors else (0, [])


def check_registry_suite_membership_required(repo_root: Path) -> tuple[int, list[str]]:
    from ...registry.suites import resolve_check_ids, suite_manifest_specs

    covered: set[str] = set()
    for spec in suite_manifest_specs():
        covered.update(resolve_check_ids(spec))
    errors = [f"{entry.id}: not assigned to any suite registry manifest" for entry in _entries(repo_root) if entry.id not in covered]
    return (1, errors) if errors else (0, [])


def check_registry_docs_link_required(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for row in _catalog_rows(repo_root):
        check_id = str(row.get("id", "")).strip()
        docs_link = str(row.get("docs_link", "")).strip()
        if not docs_link:
            errors.append(f"{check_id}: docs_link is required")
            continue
        docs_path = docs_link.split("#", 1)[0]
        if not (repo_root / docs_path).exists():
            errors.append(f"{check_id}: docs_link target missing: {docs_link}")
    return (1, errors) if errors else (0, [])


def check_registry_remediation_link_required(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for row in _catalog_rows(repo_root):
        check_id = str(row.get("id", "")).strip()
        remediation = str(row.get("remediation_link", "")).strip()
        if not remediation:
            errors.append(f"{check_id}: remediation_link is required")
            continue
        if not (repo_root / remediation).exists():
            errors.append(f"{check_id}: remediation_link target missing: {remediation}")
    return (1, errors) if errors else (0, [])


def check_registry_docs_meta_matches_runtime(repo_root: Path) -> tuple[int, list[str]]:
    meta_path = repo_root / _DOCS_META_PATH
    if not meta_path.exists():
        return 1, [f"missing generated checks docs meta: {_DOCS_META_PATH.as_posix()}"]
    lines = [line.strip() for line in meta_path.read_text(encoding="utf-8").splitlines() if line.strip() and not line.startswith("#")]
    if not lines:
        return 1, ["checks docs meta is empty"]
    data = lines[1:] if lines and lines[0].startswith("id\t") else lines
    seen = {line.split("\t", 1)[0] for line in data if line}
    runtime = {entry.id for entry in _entries(repo_root)}
    missing = sorted(runtime - seen)
    extra = sorted(seen - runtime)
    errors: list[str] = []
    if missing:
        errors.append("checks docs meta missing runtime ids: " + ", ".join(missing[:20]))
    if extra:
        errors.append("checks docs meta has unknown ids: " + ", ".join(extra[:20]))
    return (1, errors) if errors else (0, [])


def check_registry_transition_complete(repo_root: Path) -> tuple[int, list[str]]:
    from ..registry import check_id_alias_expiry, check_id_renames
    from datetime import date

    renames = check_id_renames(repo_root)
    if not renames:
        return 0, []
    expiry = check_id_alias_expiry(repo_root)
    if not expiry:
        return 1, ["check id migration map has aliases but no expiry date"]
    try:
        expires_on = date.fromisoformat(expiry)
    except ValueError:
        return 1, [f"invalid check id alias expiry date: {expiry}"]
    if date.today() > expires_on:
        return 1, [f"check id transition expired on {expiry} but legacy alias mapping still contains {len(renames)} entries"]
    return 0, []


def check_suites_inventory_ssot(repo_root: Path) -> tuple[int, list[str]]:
    from ...registry.suites import suite_manifest_specs

    specs = list(suite_manifest_specs())
    names = [spec.name for spec in specs]
    errors: list[str] = []
    if names != sorted(names):
        errors.append("suite registry names must be sorted deterministically")
    if len(names) != len(set(names)):
        errors.append("suite registry contains duplicate suite names")
    return (1, errors) if errors else (0, [])


def check_suites_no_orphans(repo_root: Path) -> tuple[int, list[str]]:
    return check_registry_suite_membership_required(repo_root)


def check_owners_ssot(repo_root: Path) -> tuple[int, list[str]]:
    return check_registry_owners_required(repo_root)


def check_remediation_ssot(repo_root: Path) -> tuple[int, list[str]]:
    return check_registry_remediation_link_required(repo_root)


def check_docs_index_complete(repo_root: Path) -> tuple[int, list[str]]:
    index = repo_root / "packages/atlasctl/docs/checks/index.md"
    if not index.exists():
        return 1, ["missing checks docs index: packages/atlasctl/docs/checks/index.md"]
    text = index.read_text(encoding="utf-8")
    required_refs = (
        "# Check Domains",
        "Generated from check registry",
        "atlasctl check list --json",
    )
    missing = [ref for ref in required_refs if ref not in text]
    if missing:
        return 1, [f"checks docs index missing required generated refs: {', '.join(missing)}"]
    return 0, []


def check_count_budget(repo_root: Path) -> tuple[int, list[str]]:
    total = len(tuple(_entries(repo_root)))
    path = repo_root / _COUNT_BUDGET_PATH
    if not path.exists():
        return 1, [f"missing checks count budget file: {_COUNT_BUDGET_PATH.as_posix()}"]
    payload = json.loads(path.read_text(encoding="utf-8"))
    max_total = int(payload.get("max_total", total))
    if total > max_total:
        return 1, [f"checks count budget exceeded: {total} > {max_total} ({_COUNT_BUDGET_PATH.as_posix()})"]
    return 0, []


def _shape_budget(repo_root: Path) -> dict[str, int]:
    path = repo_root / _SHAPE_BUDGET_PATH
    if not path.exists():
        return {
            "checks_root_max_entries": 15,
            "checks_domains_max_modules": 10,
            "checks_tools_max_modules": 10,
            "checks_module_max_loc": 600,
            "checks_legacy_signature_max": 398,
        }
    payload = json.loads(path.read_text(encoding="utf-8"))
    return {
        "checks_root_max_entries": int(payload.get("checks_root_max_entries", 15)),
        "checks_domains_max_modules": int(payload.get("checks_domains_max_modules", 10)),
        "checks_tools_max_modules": int(payload.get("checks_tools_max_modules", 10)),
        "checks_module_max_loc": int(payload.get("checks_module_max_loc", 600)),
        "checks_legacy_signature_max": int(payload.get("checks_legacy_signature_max", 398)),
    }


def _shape_budget_loc_exemptions(repo_root: Path) -> set[str]:
    path = repo_root / _SHAPE_BUDGET_PATH
    if not path.exists():
        return set()
    payload = json.loads(path.read_text(encoding="utf-8"))
    rows = payload.get("checks_module_loc_exemptions", [])
    if not isinstance(rows, list):
        return set()
    return {str(item).strip() for item in rows if str(item).strip()}


def check_checks_root_entry_budget(repo_root: Path) -> tuple[int, list[str]]:
    budget = _shape_budget(repo_root)["checks_root_max_entries"]
    root = repo_root / "packages/atlasctl/src/atlasctl/checks"
    if not root.exists():
        return 1, ["missing checks root directory: packages/atlasctl/src/atlasctl/checks"]
    ignored = {"__pycache__", "__init__.py", "README.md", "REGISTRY.generated.json", "api.py"}
    entries = [
        item.name
        for item in root.iterdir()
        if item.name not in ignored
    ]
    count = len(entries)
    if count > budget:
        return 1, [f"checks root entry budget exceeded: {count} > {budget}"]
    return 0, []


def check_checks_domains_module_budget(repo_root: Path) -> tuple[int, list[str]]:
    budget = _shape_budget(repo_root)["checks_domains_max_modules"]
    root = repo_root / "packages/atlasctl/src/atlasctl/checks/domains"
    if not root.exists():
        return 1, ["missing checks domains directory: packages/atlasctl/src/atlasctl/checks/domains"]
    module_like: list[str] = []
    for item in root.iterdir():
        if item.name == "__pycache__":
            continue
        if item.is_file() and item.suffix == ".py":
            module_like.append(item.name)
            continue
        if item.is_dir() and (item / "__init__.py").exists():
            module_like.append(item.name)
    count = len(module_like)
    if count > budget:
        return 1, [f"checks domains module budget exceeded: {count} > {budget}"]
    return 0, []


def check_checks_tools_module_budget(repo_root: Path) -> tuple[int, list[str]]:
    budget = _shape_budget(repo_root)["checks_tools_max_modules"]
    root = repo_root / "packages/atlasctl/src/atlasctl/checks/tools"
    if not root.exists():
        return 1, ["missing checks tools directory: packages/atlasctl/src/atlasctl/checks/tools"]
    modules = [path.name for path in root.glob("*.py")]
    count = len(modules)
    if count > budget:
        return 1, [f"checks tools module budget exceeded: {count} > {budget}"]
    return 0, []


def check_checks_module_loc_budget(repo_root: Path) -> tuple[int, list[str]]:
    budget = _shape_budget(repo_root)["checks_module_max_loc"]
    exemptions = _shape_budget_loc_exemptions(repo_root)
    root = repo_root / "packages/atlasctl/src/atlasctl/checks"
    offenders: list[str] = []
    for path in sorted(root.rglob("*.py")):
        if "__pycache__" in path.parts:
            continue
        rel = path.relative_to(repo_root).as_posix()
        if rel in exemptions:
            continue
        loc = len(path.read_text(encoding="utf-8", errors="ignore").splitlines())
        if loc > budget:
            offenders.append(f"{rel}: {loc} LOC > {budget}")
    return (1, offenders) if offenders else (0, [])


def check_structured_results_legacy_signature_budget(repo_root: Path) -> tuple[int, list[str]]:
    budget = _shape_budget(repo_root)["checks_legacy_signature_max"]
    root = repo_root / "packages/atlasctl/src/atlasctl/checks"
    pattern = re.compile(r"def\s+check_[A-Za-z0-9_]+\([^)]*\)\s*->\s*tuple\[\s*int\s*,\s*list\[\s*str\s*\]\s*\]")
    legacy_defs: list[str] = []
    for path in sorted(root.rglob("*.py")):
        if "__pycache__" in path.parts:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if pattern.search(text):
            legacy_defs.append(path.relative_to(repo_root).as_posix())
    total = len(legacy_defs)
    if total > budget:
        return 1, [
            f"legacy tuple-signature budget exceeded: {total} > {budget}",
            "migrate checks to structured Violation/CheckOutcome returns",
        ]
    return 0, []


def check_domains_contracts_no_nested_dirs(repo_root: Path) -> tuple[int, list[str]]:
    targets = (
        "packages/atlasctl/src/atlasctl/checks/domains/ops/contracts",
        "packages/atlasctl/src/atlasctl/checks/domains/policies/make",
    )
    errors: list[str] = []
    for rel_root in targets:
        root = repo_root / rel_root
        if not root.exists():
            continue
        for path in sorted(root.rglob("*")):
            if not path.is_dir() or path == root:
                continue
            depth = len(path.relative_to(root).parts)
            if depth > 1 and path.name != "__pycache__":
                errors.append(f"nested domain directory forbidden: {path.relative_to(repo_root).as_posix()}")
    return (1, errors) if errors else (0, [])


def check_registry_no_banned_adjectives_in_ids(repo_root: Path) -> tuple[int, list[str]]:
    terms = _forbidden_terms(repo_root)
    if not terms:
        return 0, []
    errors: list[str] = []
    for entry in _entries(repo_root):
        lowered = entry.id.lower()
        bad = [term for term in terms if term in lowered]
        if bad:
            errors.append(f"{entry.id}: check id contains banned adjective(s): {', '.join(sorted(set(bad)))}")
    return (1, errors) if errors else (0, [])


def check_registry_profile_tag_vocabulary(repo_root: Path) -> tuple[int, list[str]]:
    del repo_root
    from ..registry import TAGS_VOCAB

    available = {str(tag) for tag in TAGS_VOCAB}
    missing = [tag for tag in _REQUIRED_PROFILE_TAGS if tag not in available]
    if missing:
        return 1, [f"missing required profile tags in registry vocabulary: {', '.join(missing)}"]
    return 0, []


def check_surfaces_no_banned_adjectives_in_paths(repo_root: Path) -> tuple[int, list[str]]:
    terms = _forbidden_terms(repo_root)
    if not terms:
        return 0, []
    result = run_command(
        ["git", "ls-files", "packages/atlasctl/src/atlasctl", "packages/atlasctl/docs"],
        cwd=repo_root,
    )
    if result.code != 0:
        return 1, ["unable to list tracked atlasctl surfaces for adjective scan"]
    errors: list[str] = []
    for rel in sorted(line.strip() for line in result.stdout.splitlines() if line.strip()):
        lowered = rel.lower()
        bad = [term for term in terms if term in lowered]
        if bad:
            errors.append(f"{rel}: path contains banned adjective(s): {', '.join(sorted(set(bad)))}")
    return (1, errors) if errors else (0, [])


def check_no_raw_string_check_id_usage(repo_root: Path) -> tuple[int, list[str]]:
    checks_root = repo_root / "packages/atlasctl/src/atlasctl/checks"
    model_path = checks_root / "model.py"
    violations: list[str] = []
    for path in sorted(checks_root.rglob("*.py")):
        if "__pycache__" in path.parts or path == model_path:
            continue
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        if re.search(r"check_id\s*:\s*str\b", text):
            violations.append(f"{rel}: use CheckId wrapper instead of raw string check id annotation")
    return (1, violations) if violations else (0, [])


def check_checks_evidence_not_tracked(repo_root: Path) -> tuple[int, list[str]]:
    proc = run_command(["git", "ls-files", "artifacts/atlasctl/checks"], cwd=repo_root)
    if proc.code != 0:
        return 1, ["unable to inspect tracked files under artifacts/atlasctl/checks"]
    tracked = sorted(line.strip() for line in proc.stdout.splitlines() if line.strip())
    if tracked:
        return 1, [f"tracked check evidence file forbidden: {path}" for path in tracked]
    return 0, []


def check_no_generated_timestamp_dirs(repo_root: Path) -> tuple[int, list[str]]:
    timestamp = re.compile(r"(?:^|[-_/])(?:19|20)\d{2}[-_]?(?:0[1-9]|1[0-2])[-_]?(?:0[1-9]|[12]\d|3[01])(?:[-_T]?\d{2}[._-]?\d{2}[._-]?\d{2})?(?:$|[-_/])")
    roots = (
        repo_root / "ops" / "_generated",
        repo_root / "configs" / "_generated",
    )
    violations: list[str] = []
    for root in roots:
        if not root.exists():
            continue
        for path in sorted(root.rglob("*")):
            rel = path.relative_to(repo_root).as_posix()
            if path.name.startswith("."):
                continue
            if timestamp.search(rel):
                violations.append(f"generated path contains timestamp-like segment: {rel}")
    return (1, violations) if violations else (0, [])


def _forbidden_imports(
    root: Path,
    *,
    include_glob: str,
    forbidden_prefixes: tuple[str, ...],
) -> list[str]:
    violations: list[str] = []
    for path in sorted(root.glob(include_glob)):
        rel = path.relative_to(root).as_posix()
        try:
            tree = ast.parse(path.read_text(encoding="utf-8"))
        except SyntaxError as exc:
            violations.append(f"{rel}: syntax error while parsing imports: {exc.msg}")
            continue
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                for alias in node.names:
                    name = str(alias.name).strip()
                    if any(name == prefix or name.startswith(f"{prefix}.") for prefix in forbidden_prefixes):
                        violations.append(f"{rel}:{node.lineno}: forbidden import `{name}`")
            elif isinstance(node, ast.ImportFrom):
                module = str(node.module or "").strip()
                if any(module == prefix or module.startswith(f"{prefix}.") for prefix in forbidden_prefixes):
                    violations.append(f"{rel}:{node.lineno}: forbidden import `{module}`")
    return violations


def _check_modules(checks_root: Path) -> dict[str, Path]:
    modules: dict[str, Path] = {}
    for path in sorted(checks_root.rglob("*.py")):
        if "__pycache__" in path.parts:
            continue
        rel = path.relative_to(checks_root)
        if rel.name == "__init__.py":
            parts = rel.parts[:-1]
        else:
            parts = rel.with_suffix("").parts
        if not parts:
            module_name = "atlasctl.checks"
        else:
            module_name = "atlasctl.checks." + ".".join(parts)
        modules[module_name] = path
    return modules


def _resolve_imported_modules(current_module: str, node: ast.AST) -> Iterable[str]:
    if isinstance(node, ast.Import):
        for alias in node.names:
            name = str(alias.name).strip()
            if name:
                yield name
        return
    if not isinstance(node, ast.ImportFrom):
        return
    module = str(node.module or "").strip()
    if node.level <= 0:
        if module:
            yield module
            for alias in node.names:
                name = str(alias.name).strip()
                if name and name != "*":
                    yield f"{module}.{name}"
        return
    current_parts = current_module.split(".")
    base_parts = current_parts[:-1]
    if node.level > len(base_parts):
        return
    target_parts = base_parts[: len(base_parts) - node.level + 1]
    if module:
        target_parts = [*target_parts, *module.split(".")]
    if target_parts:
        base = ".".join(target_parts)
        yield base
        for alias in node.names:
            name = str(alias.name).strip()
            if name and name != "*":
                yield f"{base}.{name}"


def _cycle_paths(graph: dict[str, set[str]]) -> list[list[str]]:
    visiting: set[str] = set()
    visited: set[str] = set()
    stack: list[str] = []
    cycles: list[list[str]] = []

    def _visit(node: str) -> None:
        if node in visited:
            return
        if node in visiting:
            if node in stack:
                start = stack.index(node)
                cycle = stack[start:] + [node]
                cycles.append(cycle)
            return
        visiting.add(node)
        stack.append(node)
        for nxt in sorted(graph.get(node, ())):
            _visit(nxt)
        stack.pop()
        visiting.remove(node)
        visited.add(node)

    for mod in sorted(graph):
        _visit(mod)
    return cycles


def check_checks_import_cycles(repo_root: Path) -> tuple[int, list[str]]:
    checks_root = repo_root / "packages/atlasctl/src/atlasctl/checks"
    if not checks_root.exists():
        return 1, ["missing checks root: packages/atlasctl/src/atlasctl/checks"]
    modules = _check_modules(checks_root)
    graph: dict[str, set[str]] = {module: set() for module in modules}
    for module, path in modules.items():
        try:
            tree = ast.parse(path.read_text(encoding="utf-8"))
        except SyntaxError as exc:
            rel = path.relative_to(repo_root).as_posix()
            return 1, [f"{rel}: syntax error while parsing imports: {exc.msg}"]
        for node in tree.body:
            for imported in _resolve_imported_modules(module, node):
                if imported in modules and imported != module:
                    graph[module].add(imported)
    cycles = _cycle_paths(graph)
    if not cycles:
        return 0, []
    errors = [f"import cycle in atlasctl.checks: {' -> '.join(cycle)}" for cycle in cycles[:10]]
    if len(cycles) > 10:
        errors.append(f"additional cycles omitted: {len(cycles) - 10}")
    return 1, errors


def check_checks_forbidden_imports(repo_root: Path) -> tuple[int, list[str]]:
    checks_root = repo_root / "packages/atlasctl/src/atlasctl/checks"
    if not checks_root.exists():
        return 1, ["missing checks root: packages/atlasctl/src/atlasctl/checks"]
    modules = _check_modules(checks_root)
    forbidden_tokens = (".tests", ".fixtures", "atlasctl.commands.ops.tests")
    errors: list[str] = []
    for module, path in modules.items():
        try:
            tree = ast.parse(path.read_text(encoding="utf-8"))
        except SyntaxError as exc:
            rel = path.relative_to(repo_root).as_posix()
            errors.append(f"{rel}: syntax error while parsing imports: {exc.msg}")
            continue
        rel = path.relative_to(repo_root).as_posix()
        for node in ast.walk(tree):
            for imported in _resolve_imported_modules(module, node):
                dotted = f".{imported}"
                if any(token in imported or token in dotted for token in forbidden_tokens):
                    errors.append(f"{rel}:{getattr(node, 'lineno', 0)}: forbidden import `{imported}`")
    return (1, errors) if errors else (0, [])


def check_no_checks_outside_domains_tools(repo_root: Path) -> tuple[int, list[str]]:
    checks_root = repo_root / "packages/atlasctl/src/atlasctl/checks"
    if not checks_root.exists():
        return 1, ["missing checks root: packages/atlasctl/src/atlasctl/checks"]
    allowed_prefixes = (
        "atlasctl.checks.domains.",
        "atlasctl.checks.tools.",
        "atlasctl.checks.domains",
        "atlasctl.checks.tools",
    )
    violations: list[str] = []
    from ..registry import list_checks

    for check in list_checks():
        fn = getattr(check, "fn", None)
        module = str(getattr(fn, "__module__", "")).strip()
        if module and module.startswith("atlasctl.checks.") and not module.startswith(allowed_prefixes):
            violations.append(f"{check.check_id}: implementation must live under checks/domains or checks/tools ({module})")
    return (1, sorted(set(violations))) if violations else (0, [])


def check_legacy_check_directories_absent(repo_root: Path) -> tuple[int, list[str]]:
    legacy_dirs = (
        repo_root / "packages/atlasctl/src/atlasctl/checks/layout",
        repo_root / "packages/atlasctl/src/atlasctl/checks/repo",
    )
    violations = [f"legacy checks directory must be removed: {path.relative_to(repo_root).as_posix()}" for path in legacy_dirs if path.exists()]
    return (1, violations) if violations else (0, [])


def _check_python_files(repo_root: Path) -> list[Path]:
    root = repo_root / "packages/atlasctl/src/atlasctl/checks"
    if not root.exists():
        return []
    return [path for path in sorted(root.rglob("*.py")) if "__pycache__" not in path.parts]


def check_checks_file_count_budget(repo_root: Path) -> tuple[int, list[str]]:
    root = repo_root / "packages/atlasctl/src/atlasctl/checks"
    if not root.exists():
        return 1, ["missing checks root: packages/atlasctl/src/atlasctl/checks"]
    budget = 40
    total = sum(1 for path in root.rglob("*") if path.is_file() and "__pycache__" not in path.parts)
    if total > budget:
        return 1, [f"checks file-count budget exceeded: {total} > {budget}"]
    return 0, []


def check_checks_tree_depth_budget(repo_root: Path) -> tuple[int, list[str]]:
    root = repo_root / "packages/atlasctl/src/atlasctl/checks"
    if not root.exists():
        return 1, ["missing checks root: packages/atlasctl/src/atlasctl/checks"]
    budget = 2
    errors: list[str] = []
    for path in sorted(root.rglob("*")):
        if "__pycache__" in path.parts:
            continue
        depth = len(path.relative_to(root).parts)
        if depth > budget:
            errors.append(f"checks tree depth exceeded: {path.relative_to(repo_root).as_posix()} depth={depth} > {budget}")
            if len(errors) >= 20:
                break
    return (1, errors) if errors else (0, [])


def check_domains_directory_shape(repo_root: Path) -> tuple[int, list[str]]:
    root = repo_root / "packages/atlasctl/src/atlasctl/checks/domains"
    if not root.exists():
        return 1, ["missing checks domains root: packages/atlasctl/src/atlasctl/checks/domains"]
    errors: list[str] = []
    for path in sorted(root.iterdir(), key=lambda row: row.name):
        if path.name == "__pycache__":
            continue
        if path.is_dir():
            errors.append(f"domains root must contain only python modules: {path.relative_to(repo_root).as_posix()}")
            continue
        if path.is_file() and path.suffix == ".py":
            continue
        errors.append(f"domains root contains unsupported entry: {path.relative_to(repo_root).as_posix()}")
    return (1, errors) if errors else (0, [])


def check_no_relative_imports_across_domains(repo_root: Path) -> tuple[int, list[str]]:
    domains_root = repo_root / "packages/atlasctl/src/atlasctl/checks/domains"
    if not domains_root.exists():
        return 1, ["missing checks domains root: packages/atlasctl/src/atlasctl/checks/domains"]
    errors: list[str] = []
    for path in sorted(domains_root.glob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        try:
            tree = ast.parse(path.read_text(encoding="utf-8", errors="ignore"))
        except SyntaxError:
            continue
        for node in ast.walk(tree):
            if not isinstance(node, ast.ImportFrom):
                continue
            if int(getattr(node, "level", 0)) <= 0:
                continue
            errors.append(f"{rel}:{node.lineno}: cross-domain relative imports are forbidden; use absolute atlasctl.checks imports")
    return (1, errors) if errors else (0, [])


def check_no_ops_runtime_asset_imports(repo_root: Path) -> tuple[int, list[str]]:
    root = repo_root / "packages/atlasctl/src/atlasctl/checks"
    if not root.exists():
        return 1, ["missing checks root: packages/atlasctl/src/atlasctl/checks"]
    forbidden = ("atlasctl.commands.ops.runtime_modules", "atlasctl.commands.ops.runtime", "atlasctl.ops.runtime")
    errors: list[str] = []
    for path in _check_python_files(repo_root):
        rel = path.relative_to(repo_root).as_posix()
        try:
            tree = ast.parse(path.read_text(encoding="utf-8", errors="ignore"))
        except SyntaxError:
            continue
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                modules = [alias.name for alias in node.names]
            elif isinstance(node, ast.ImportFrom):
                modules = [str(node.module or "")]
            else:
                continue
            for module in modules:
                if any(module == tok or module.startswith(f"{tok}.") for tok in forbidden):
                    errors.append(f"{rel}:{getattr(node, 'lineno', 0)}: forbidden import from ops runtime assets `{module}`")
    return (1, errors) if errors else (0, [])


def check_no_tests_fixtures_imports(repo_root: Path) -> tuple[int, list[str]]:
    root = repo_root / "packages/atlasctl/src/atlasctl/checks"
    if not root.exists():
        return 1, ["missing checks root: packages/atlasctl/src/atlasctl/checks"]
    errors: list[str] = []
    for path in _check_python_files(repo_root):
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        if ".tests" in text or ".fixtures" in text:
            errors.append(f"{rel}: imports from tests/fixtures are forbidden in checks package")
    return (1, errors) if errors else (0, [])


def check_unused_imports_in_checks(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for path in _check_python_files(repo_root):
        rel = path.relative_to(repo_root).as_posix()
        try:
            tree = ast.parse(path.read_text(encoding="utf-8", errors="ignore"))
        except SyntaxError:
            continue
        imported: dict[str, int] = {}
        used: set[str] = set()
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                for alias in node.names:
                    name = alias.asname or alias.name.split(".", 1)[0]
                    imported[name] = node.lineno
            elif isinstance(node, ast.ImportFrom):
                for alias in node.names:
                    if alias.name == "*":
                        continue
                    name = alias.asname or alias.name
                    imported[name] = node.lineno
            elif isinstance(node, ast.Name):
                used.add(node.id)
        for name, lineno in sorted(imported.items(), key=lambda item: item[1]):
            if name.startswith("_"):
                continue
            if name not in used:
                errors.append(f"{rel}:{lineno}: unused import `{name}` in checks package")
    return (1, errors) if errors else (0, [])


def check_checks_no_print_calls(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for path in _check_python_files(repo_root):
        rel = path.relative_to(repo_root).as_posix()
        try:
            tree = ast.parse(path.read_text(encoding="utf-8", errors="ignore"))
        except SyntaxError:
            continue
        for node in ast.walk(tree):
            if isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and node.func.id == "print":
                errors.append(f"{rel}:{node.lineno}: checks must not call print(); use structured violations/reporting")
    return (1, errors) if errors else (0, [])


def check_checks_no_sys_exit_calls(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for path in _check_python_files(repo_root):
        rel = path.relative_to(repo_root).as_posix()
        try:
            tree = ast.parse(path.read_text(encoding="utf-8", errors="ignore"))
        except SyntaxError:
            continue
        for node in ast.walk(tree):
            if (
                isinstance(node, ast.Call)
                and isinstance(node.func, ast.Attribute)
                and isinstance(node.func.value, ast.Name)
                and node.func.value.id == "sys"
                and node.func.attr == "exit"
            ):
                errors.append(f"{rel}:{node.lineno}: checks must not call sys.exit(); return violations/results")
    return (1, errors) if errors else (0, [])


def check_checks_no_direct_env_reads(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for path in _check_python_files(repo_root):
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        if "ctx.env" in text:
            continue
        if "os.environ" in text or "os.getenv(" in text:
            errors.append(f"{rel}: direct environment reads are forbidden; use CheckContext.env snapshot")
    return (1, errors) if errors else (0, [])


def check_checks_no_path_dot_usage(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    pattern = re.compile(r"\b(?:Path|pathlib\.Path)\(\s*[\"']\.[\"']\s*\)")
    for path in _check_python_files(repo_root):
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        for idx, line in enumerate(text.splitlines(), start=1):
            if pattern.search(line):
                errors.append(f"{rel}:{idx}: dot-path construction is forbidden; anchor paths to repo_root")
    return (1, errors) if errors else (0, [])


def check_checks_no_cwd_reliance(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for path in _check_python_files(repo_root):
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        try:
            tree = ast.parse(text, filename=rel)
        except SyntaxError:
            continue
        for node in ast.walk(tree):
            if not isinstance(node, ast.Call):
                continue
            lineno = getattr(node, "lineno", 1)
            func = node.func
            if isinstance(func, ast.Attribute):
                if isinstance(func.value, ast.Name):
                    if func.value.id == "Path" and func.attr == "cwd":
                        errors.append(f"{rel}:{lineno}: cwd reliance is forbidden; use explicit repo_root context")
                        continue
                    if func.value.id == "os" and func.attr in {"getcwd", "chdir"}:
                        errors.append(f"{rel}:{lineno}: cwd reliance is forbidden; use explicit repo_root context")
                        continue
                if isinstance(func.value, ast.Attribute) and isinstance(func.value.value, ast.Name):
                    if func.value.value.id == "pathlib" and func.value.attr == "Path" and func.attr == "cwd":
                        errors.append(f"{rel}:{lineno}: cwd reliance is forbidden; use explicit repo_root context")
    return (1, errors) if errors else (0, [])


def check_write_effect_declared_for_writing_checks(repo_root: Path) -> tuple[int, list[str]]:
    from ..registry import list_checks

    violations: list[str] = []
    write_tokens = ("write_text(", "write_bytes(", ".open(", "json.dump(", "yaml.safe_dump(", "dump(")
    for check in list_checks():
        fn = getattr(check, "fn", None)
        code_obj = getattr(fn, "__code__", None)
        if code_obj is None:
            continue
        path = Path(code_obj.co_filename)
        if not path.exists():
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        effects = {str(effect) for effect in getattr(check, "effects", ())}
        if any(token in text for token in write_tokens) and "fs_write" not in effects:
            violations.append(f"{check.check_id}: file write usage requires effects to include fs_write")
    return (1, sorted(set(violations))) if violations else (0, [])


def check_no_duplicate_model_definitions(repo_root: Path) -> tuple[int, list[str]]:
    canonical = repo_root / "packages/atlasctl/src/atlasctl/checks/model.py"
    if not canonical.exists():
        return 1, ["missing canonical model module: packages/atlasctl/src/atlasctl/checks/model.py"]
    target_classes = {
        "CheckDef",
        "CheckResult",
        "CheckRunReport",
        "CheckContext",
        "Violation",
        "Effect",
    }
    errors: list[str] = []
    for path in _check_python_files(repo_root):
        if path == canonical:
            continue
        rel = path.relative_to(repo_root).as_posix()
        try:
            tree = ast.parse(path.read_text(encoding="utf-8", errors="ignore"))
        except SyntaxError:
            continue
        for node in tree.body:
            if isinstance(node, ast.ClassDef) and node.name in target_classes:
                errors.append(f"{rel}:{node.lineno}: duplicate model class `{node.name}`; keep canonical definition in checks/model.py")
    return (1, errors) if errors else (0, [])


def check_checks_root_allowed_entries_only(repo_root: Path) -> tuple[int, list[str]]:
    checks_root = repo_root / "packages/atlasctl/src/atlasctl/checks"
    if not checks_root.exists():
        return 1, ["missing checks root: packages/atlasctl/src/atlasctl/checks"]
    allowed = {
        "README.md",
        "__init__.py",
        "model.py",
        "registry.py",
        "selectors.py",
        "policy.py",
        "runner.py",
        "report.py",
        "gen_registry.py",
        "tools",
        "domains",
    }
    errors: list[str] = []
    for item in sorted(checks_root.iterdir(), key=lambda row: row.name):
        if item.name == "__pycache__":
            continue
        if item.name not in allowed:
            errors.append(f"checks root contains non-canonical entry: {item.relative_to(repo_root).as_posix()}")
    return (1, errors) if errors else (0, [])


def check_registry_generated_read_only(repo_root: Path) -> tuple[int, list[str]]:
    path = repo_root / "packages/atlasctl/src/atlasctl/checks/REGISTRY.generated.json"
    if not path.exists():
        return 1, ["missing generated registry file: packages/atlasctl/src/atlasctl/checks/REGISTRY.generated.json"]
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        return 1, [f"generated registry json parse error: {exc}"]
    if "schema_version" not in payload:
        return 1, ["generated registry missing schema_version marker"]
    if not isinstance(payload.get("checks", []), list):
        return 1, ["generated registry missing checks list payload"]
    return 0, []


def check_registry_generated_is_readonly(repo_root: Path) -> tuple[int, list[str]]:
    return check_registry_generated_read_only(repo_root)


def check_registry_toml_generated_contract(repo_root: Path) -> tuple[int, list[str]]:
    path = repo_root / "packages/atlasctl/src/atlasctl/checks/REGISTRY.toml"
    if not path.exists():
        return 0, []
    header = path.read_text(encoding="utf-8", errors="ignore").splitlines()[:8]
    combined = "\n".join(header).lower()
    if "regenerate with:" in combined:
        return 0, []
    return 1, ["REGISTRY.toml must include generator marker header (`Regenerate with: ./bin/atlasctl gen checks-registry`)"]


def check_all_checks_have_owner(repo_root: Path) -> tuple[int, list[str]]:
    from ..registry import list_checks

    violations = [f"{check.check_id}: owner is required" for check in list_checks() if not getattr(check, "owners", ())]
    return (1, violations) if violations else (0, [])


def check_checks_approved_top_level_entries_only(repo_root: Path) -> tuple[int, list[str]]:
    return check_checks_root_allowed_entries_only(repo_root)


def check_registry_generated_canonical_hash(repo_root: Path) -> tuple[int, list[str]]:
    path = repo_root / "packages/atlasctl/src/atlasctl/checks/REGISTRY.generated.json"
    if not path.exists():
        return 1, ["missing generated registry: packages/atlasctl/src/atlasctl/checks/REGISTRY.generated.json"]
    raw = path.read_text(encoding="utf-8")
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as exc:
        return 1, [f"invalid generated registry json: {exc}"]
    canonical = json.dumps(payload, indent=2, sort_keys=True) + "\n"
    if raw != canonical:
        digest = hashlib.sha256(canonical.encode("utf-8")).hexdigest()
        return 1, [f"generated registry formatting is non-deterministic; canonical sha256={digest}"]
    return 0, []


def check_all_checks_have_docs_string(repo_root: Path) -> tuple[int, list[str]]:
    from ..registry import list_checks

    violations = []
    for check in list_checks():
        description = str(getattr(check, "description", "")).strip()
        if not description:
            violations.append(f"{check.check_id}: description is required")
    return (1, violations) if violations else (0, [])


def check_no_subprocess_import_outside_adapters(repo_root: Path) -> tuple[int, list[str]]:
    checks_root = repo_root / "packages/atlasctl/src/atlasctl/checks"
    allowed = {
        "packages/atlasctl/src/atlasctl/checks/adapters.py",
        "packages/atlasctl/src/atlasctl/checks/engine.py",
    }
    violations: list[str] = []
    for path in sorted(checks_root.rglob("*.py")):
        if "__pycache__" in path.parts:
            continue
        rel = path.relative_to(repo_root).as_posix()
        if rel in allowed:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if re.search(r"^\s*import\s+subprocess\b", text, flags=re.MULTILINE) or re.search(
            r"^\s*from\s+subprocess\s+import\b", text, flags=re.MULTILINE
        ):
            violations.append(f"{rel}: subprocess import is restricted to adapters")
    return (1, violations) if violations else (0, [])


def check_no_network_client_import_outside_adapters(repo_root: Path) -> tuple[int, list[str]]:
    checks_root = repo_root / "packages/atlasctl/src/atlasctl/checks"
    allowed = {
        "packages/atlasctl/src/atlasctl/checks/adapters.py",
    }
    violations: list[str] = []
    for path in sorted(checks_root.rglob("*.py")):
        if "__pycache__" in path.parts:
            continue
        rel = path.relative_to(repo_root).as_posix()
        if rel in allowed:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if re.search(r"^\s*import\s+requests\b", text, flags=re.MULTILINE) or re.search(
            r"^\s*from\s+requests\s+import\b", text, flags=re.MULTILINE
        ):
            violations.append(f"{rel}: requests import is restricted to adapters")
        if re.search(r"^\s*import\s+httpx\b", text, flags=re.MULTILINE) or re.search(
            r"^\s*from\s+httpx\s+import\b", text, flags=re.MULTILINE
        ):
            violations.append(f"{rel}: httpx import is restricted to adapters")
    return (1, violations) if violations else (0, [])


def check_no_tools_domain_mirrors_exist(repo_root: Path) -> tuple[int, list[str]]:
    tools_root = repo_root / "packages/atlasctl/src/atlasctl/checks/tools"
    if not tools_root.exists():
        return 1, ["missing checks tools directory: packages/atlasctl/src/atlasctl/checks/tools"]
    mirrors = [path.relative_to(repo_root).as_posix() for path in sorted(tools_root.glob("*_domain")) if path.is_dir()]
    return (1, [f"tools domain mirror directory must be removed: {item}" for item in mirrors]) if mirrors else (0, [])


def check_all_checks_declare_effects(repo_root: Path) -> tuple[int, list[str]]:
    from ..registry import list_checks

    violations = [f"{check.check_id}: effects declaration is required" for check in list_checks() if not getattr(check, "effects", ())]
    return (1, violations) if violations else (0, [])


def check_all_checks_have_tags(repo_root: Path) -> tuple[int, list[str]]:
    from ..registry import list_checks

    violations = [f"{check.check_id}: tags declaration is required" for check in list_checks() if not getattr(check, "tags", ())]
    return (1, violations) if violations else (0, [])


def check_no_subprocess_usage_without_declared_effect(repo_root: Path) -> tuple[int, list[str]]:
    from ..registry import list_checks

    violations: list[str] = []
    for check in list_checks():
        fn = getattr(check, "fn", None)
        file = Path(getattr(fn, "__code__", None).co_filename) if getattr(fn, "__code__", None) else None
        if file is None or not file.exists():
            continue
        text = file.read_text(encoding="utf-8", errors="ignore")
        subprocess_tokens = ("subprocess.run(", "subprocess.Popen(", "run_command(")
        if any(token in text for token in subprocess_tokens) and "subprocess" not in {str(e) for e in getattr(check, "effects", ())}:
            violations.append(f"{check.check_id}: subprocess usage requires effects to include subprocess")
    return (1, sorted(set(violations))) if violations else (0, [])


def check_no_network_usage_without_declared_effect(repo_root: Path) -> tuple[int, list[str]]:
    from ..registry import list_checks

    violations: list[str] = []
    for check in list_checks():
        fn = getattr(check, "fn", None)
        file = Path(getattr(fn, "__code__", None).co_filename) if getattr(fn, "__code__", None) else None
        if file is None or not file.exists():
            continue
        text = file.read_text(encoding="utf-8", errors="ignore")
        network_tokens = ("requests.", "httpx.", "urllib.request", "socket.")
        if any(token in text for token in network_tokens) and "network" not in {str(e) for e in getattr(check, "effects", ())}:
            violations.append(f"{check.check_id}: network usage requires effects to include network")
    return (1, sorted(set(violations))) if violations else (0, [])


def check_write_roots_are_evidence_only(repo_root: Path) -> tuple[int, list[str]]:
    from ..registry import list_checks

    violations: list[str] = []
    for check in list_checks():
        roots = tuple(getattr(check, "writes_allowed_roots", ()))
        for root in roots:
            value = str(root).strip()
            if not value:
                continue
            if not (
                value.startswith("artifacts/evidence/")
                or value.startswith("artifacts/atlasctl/checks/")
                or value.startswith("artifacts/runs/")
            ):
                violations.append(f"{check.check_id}: write root must be under managed evidence roots ({value})")
    return (1, sorted(set(violations))) if violations else (0, [])


def check_internal_checks_tree_policy(repo_root: Path) -> tuple[int, list[str]]:
    checks_root = repo_root / "packages/atlasctl/src/atlasctl/checks"
    if not checks_root.exists():
        return 1, ["missing checks root: packages/atlasctl/src/atlasctl/checks"]
    errors: list[str] = []
    for rel in ("layout", "repo", "registry"):
        path = checks_root / rel
        if path.exists():
            errors.append(f"forbidden checks tree entry: {path.relative_to(repo_root).as_posix()}")
    domains_root = checks_root / "domains"
    if domains_root.exists():
        for path in sorted(domains_root.rglob("*")):
            if not path.is_dir():
                continue
            rel_parts = path.relative_to(domains_root).parts
            if len(rel_parts) >= 2 and "__pycache__" not in rel_parts:
                errors.append(f"nested domain directory forbidden: {path.relative_to(repo_root).as_posix()}")
    return (1, errors) if errors else (0, [])


def check_internal_no_layout_repo_registry_dirs(repo_root: Path) -> tuple[int, list[str]]:
    return check_internal_checks_tree_policy(repo_root)


def check_internal_checks_root_budget(repo_root: Path) -> tuple[int, list[str]]:
    checks_root = repo_root / "packages/atlasctl/src/atlasctl/checks"
    if not checks_root.exists():
        return 1, ["missing checks root: packages/atlasctl/src/atlasctl/checks"]
    budget = 15
    entries = [path for path in checks_root.iterdir() if path.name != "__pycache__"]
    if len(entries) > budget:
        return 1, [f"checks root entry budget exceeded: {len(entries)} > {budget}"]
    return 0, []


def check_internal_root_budget(repo_root: Path) -> tuple[int, list[str]]:
    return check_internal_checks_root_budget(repo_root)


def check_internal_domains_flat_modules_only(repo_root: Path) -> tuple[int, list[str]]:
    domains_root = repo_root / "packages/atlasctl/src/atlasctl/checks/domains"
    if not domains_root.exists():
        return 1, ["missing checks domains root: packages/atlasctl/src/atlasctl/checks/domains"]
    errors: list[str] = []
    for path in sorted(domains_root.iterdir()):
        if path.name in {"__pycache__", "__init__.py"}:
            continue
        if path.is_file() and path.suffix == ".py":
            continue
        if path.is_dir():
            # Allow package dirs that are currently active only as a migration exception.
            errors.append(f"domains must be flat modules only: {path.relative_to(repo_root).as_posix()}")
    return (1, errors) if errors else (0, [])


def check_internal_domains_flat(repo_root: Path) -> tuple[int, list[str]]:
    return check_internal_domains_flat_modules_only(repo_root)


def check_internal_no_file_per_check_explosion(repo_root: Path) -> tuple[int, list[str]]:
    checks_root = repo_root / "packages/atlasctl/src/atlasctl/checks"
    budget = 15
    errors: list[str] = []
    for directory in sorted(path for path in checks_root.rglob("*") if path.is_dir() and "__pycache__" not in path.parts):
        py_files = [row for row in directory.glob("*.py")]
        if len(py_files) > budget:
            errors.append(
                f"python file explosion under checks tree: {directory.relative_to(repo_root).as_posix()} has {len(py_files)} files > {budget}"
            )
    return (1, errors) if errors else (0, [])


def check_internal_no_duplicate_engines(repo_root: Path) -> tuple[int, list[str]]:
    checks_root = repo_root / "packages/atlasctl/src/atlasctl/checks"
    files = {
        "runner.py": checks_root / "runner.py",
        "policy.py": checks_root / "policy.py",
        "report.py": checks_root / "report.py",
        "selectors.py": checks_root / "selectors.py",
    }
    errors: list[str] = []
    for name, canonical in files.items():
        if not canonical.exists():
            errors.append(f"missing canonical checks module: {canonical.relative_to(repo_root).as_posix()}")
            continue
        duplicates = [path for path in checks_root.rglob(name) if path.resolve() != canonical.resolve() and "__pycache__" not in path.parts]
        for dup in sorted(duplicates):
            errors.append(f"duplicate engine module forbidden: {dup.relative_to(repo_root).as_posix()} (canonical: {canonical.relative_to(repo_root).as_posix()})")
    return (1, errors) if errors else (0, [])


def check_internal_no_adapters_usage(repo_root: Path) -> tuple[int, list[str]]:
    checks_root = repo_root / "packages/atlasctl/src/atlasctl/checks"
    if not checks_root.exists():
        return 1, ["missing checks root: packages/atlasctl/src/atlasctl/checks"]
    errors: list[str] = []
    for path in sorted(checks_root.rglob("*.py")):
        if "__pycache__" in path.parts:
            continue
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        if "atlasctl.checks.adapters" in text and not rel.endswith("checks/adapters.py"):
            errors.append(f"forbidden adapters import usage outside compatibility module: {rel}")
    return (1, errors) if errors else (0, [])


def check_internal_no_registry_toml_reads(repo_root: Path) -> tuple[int, list[str]]:
    checks_root = repo_root / "packages/atlasctl/src/atlasctl/checks"
    errors: list[str] = []
    for path in sorted(checks_root.rglob("*.py")):
        if "__pycache__" in path.parts:
            continue
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        if rel.endswith("checks/domains/internal.py") or rel.endswith("checks/registry/__init__.py"):
            continue
        if "REGISTRY.toml" in text and "gen_registry.py" not in rel and "registry/ssot.py" not in rel:
            errors.append(f"runtime code must not read REGISTRY.toml directly: {rel}")
    return (1, errors) if errors else (0, [])


def check_internal_no_generated_registry_as_input(repo_root: Path) -> tuple[int, list[str]]:
    checks_root = repo_root / "packages/atlasctl/src/atlasctl/checks"
    errors: list[str] = []
    for path in sorted(checks_root.rglob("*.py")):
        if "__pycache__" in path.parts:
            continue
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        if rel.endswith("checks/domains/internal.py") or rel.endswith("checks/registry/__init__.py"):
            continue
        if "REGISTRY.generated.json" in text and "gen_registry.py" not in rel and "registry/ssot.py" not in rel:
            errors.append(f"runtime code must not use generated registry json as input: {rel}")
    return (1, errors) if errors else (0, [])


def check_internal_single_runner_surface(repo_root: Path) -> tuple[int, list[str]]:
    commands_root = repo_root / "packages/atlasctl/src/atlasctl/commands"
    if not commands_root.exists():
        return 1, ["missing commands root: packages/atlasctl/src/atlasctl/commands"]
    violations: list[str] = []
    for path in sorted(commands_root.rglob("*.py")):
        if "__pycache__" in path.parts:
            continue
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        if "atlasctl.engine.runner" in text or "from ...engine.runner" in text or "from ..engine.runner" in text:
            violations.append(f"commands must use checks.runner surface only (found engine.runner import): {rel}")
    return (1, violations) if violations else (0, [])


def check_internal_no_command_logic_in_checks(repo_root: Path) -> tuple[int, list[str]]:
    return check_checks_no_commands_import(repo_root)


def check_internal_no_checks_logic_in_commands(repo_root: Path) -> tuple[int, list[str]]:
    return check_commands_no_domains_import(repo_root)


def check_internal_adapters_module_quarantined(repo_root: Path) -> tuple[int, list[str]]:
    path = repo_root / "packages/atlasctl/src/atlasctl/checks/adapters.py"
    if not path.exists():
        return 0, []
    text = path.read_text(encoding="utf-8", errors="ignore")
    marker = "compatibility adapter"
    if marker not in text.lower():
        return 1, ["checks/adapters.py must be explicitly marked as compatibility adapter while migration is active"]
    return 0, []


def check_root_policy_compat_shims_not_expired(repo_root: Path) -> tuple[int, list[str]]:
    try:
        policy = load_root_policy(repo_root)
    except Exception as exc:
        return 1, [f"root policy parse error: {exc}"]
    errors: list[str] = []
    for shim in sorted(policy.compat_shims):
        expiry = policy.compat_shim_expiry.get(shim)
        if expiry is None:
            errors.append(f"compat shim `{shim}` missing expiry in checks/tools/root_policy.json::compat_shim_expiry")
            continue
        if date.today() > expiry:
            errors.append(f"compat shim `{shim}` expired on {expiry.isoformat()}; remove shim or renew policy explicitly")
    return (1, errors) if errors else (0, [])


def check_registry_import_hygiene(repo_root: Path) -> tuple[int, list[str]]:
    registry_file = repo_root / "packages/atlasctl/src/atlasctl/checks/registry.py"
    if not registry_file.exists():
        return 1, ["missing checks registry module: packages/atlasctl/src/atlasctl/checks/registry.py"]
    banned_roots = {
        "numpy",
        "pandas",
        "torch",
        "tensorflow",
        "sklearn",
        "matplotlib",
        "seaborn",
        "scipy",
    }
    errors: list[str] = []
    for path in (registry_file,):
        rel = path.relative_to(repo_root).as_posix()
        try:
            tree = ast.parse(path.read_text(encoding="utf-8"))
        except SyntaxError as exc:
            errors.append(f"{rel}: syntax error while parsing imports: {exc.msg}")
            continue
        for node in tree.body:
            imports = list(_resolve_imported_modules(f"atlasctl.checks.registry.{path.stem}", node))
            for imported in imports:
                root = imported.split(".", 1)[0]
                if root in banned_roots:
                    errors.append(f"{rel}:{getattr(node, 'lineno', 0)}: forbidden heavy import `{imported}`")
    return (1, errors) if errors else (0, [])


def check_internal_registry_ssot_only(repo_root: Path) -> tuple[int, list[str]]:
    checks_root = repo_root / "packages/atlasctl/src/atlasctl/checks"
    allowed_importers = {
        "packages/atlasctl/src/atlasctl/checks/registry.py",
        "packages/atlasctl/src/atlasctl/checks/domains/internal.py",
        "packages/atlasctl/src/atlasctl/checks/gen_registry.py",
    }
    errors: list[str] = []
    for path in sorted(checks_root.rglob("*.py")):
        if "__pycache__" in path.parts:
            continue
        rel = path.relative_to(repo_root).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        if "registry_legacy" in text and rel not in allowed_importers:
            errors.append(f"registry legacy imports are forbidden outside explicit migration wrappers: {rel}")
    return (1, errors) if errors else (0, [])


def check_docs_checks_no_ops_imports(repo_root: Path) -> tuple[int, list[str]]:
    docs_root = repo_root / "packages/atlasctl/src/atlasctl/checks/tools"
    candidates = [
        docs_root / "docs_integrity.py",
        docs_root / "docs_ci_surface_documented.py",
    ]
    existing = [path for path in candidates if path.exists()]
    if not existing:
        return 1, ["missing docs checks module roots: packages/atlasctl/src/atlasctl/checks/tools/docs_integrity.py"]
    errors: list[str] = []
    forbidden_prefixes = ("atlasctl.commands.ops", "atlasctl.ops")
    for path in sorted(existing):
        rel = path.relative_to(repo_root).as_posix()
        try:
            tree = ast.parse(path.read_text(encoding="utf-8", errors="ignore"))
        except SyntaxError as exc:
            errors.append(f"{rel}: syntax error while parsing imports: {exc.msg}")
            continue
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                modules = [alias.name for alias in node.names]
            elif isinstance(node, ast.ImportFrom):
                modules = [str(node.module or "")]
            else:
                continue
            for module in modules:
                if any(module == prefix or module.startswith(f"{prefix}.") for prefix in forbidden_prefixes):
                    errors.append(f"{rel}:{getattr(node, 'lineno', 0)}: docs checks must not import ops modules directly (`{module}`)")
    return (1, errors) if errors else (0, [])


def check_scattered_registry_caches(repo_root: Path) -> tuple[int, list[str]]:
    checks_root = repo_root / "packages/atlasctl/src/atlasctl/checks"
    if not checks_root.exists():
        return 1, ["missing checks root: packages/atlasctl/src/atlasctl/checks"]
    allowed = {
        "packages/atlasctl/src/atlasctl/checks/registry/catalog.py",
    }
    violations: list[str] = []
    for path in sorted(checks_root.rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        try:
            tree = ast.parse(path.read_text(encoding="utf-8", errors="ignore"))
        except SyntaxError:
            continue
        has_lru_cache = False
        for node in ast.walk(tree):
            if not isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                continue
            for decorator in node.decorator_list:
                if isinstance(decorator, ast.Name) and decorator.id == "lru_cache":
                    has_lru_cache = True
                    break
                if isinstance(decorator, ast.Call):
                    func = decorator.func
                    if isinstance(func, ast.Name) and func.id == "lru_cache":
                        has_lru_cache = True
                        break
                    if isinstance(func, ast.Attribute) and func.attr == "lru_cache":
                        has_lru_cache = True
                        break
                if isinstance(decorator, ast.Attribute) and decorator.attr == "lru_cache":
                    has_lru_cache = True
                    break
            if has_lru_cache:
                break
        if not has_lru_cache:
            continue
        if rel not in allowed:
            violations.append(f"{rel}: lru_cache usage outside registry index is forbidden")
    return (1, violations) if violations else (0, [])


def check_commands_no_domains_import(repo_root: Path) -> tuple[int, list[str]]:
    commands_root = repo_root / "packages/atlasctl/src/atlasctl/commands"
    if not commands_root.exists():
        return 1, ["missing commands directory: packages/atlasctl/src/atlasctl/commands"]
    violations = _forbidden_imports(
        commands_root,
        include_glob="**/*.py",
        forbidden_prefixes=("atlasctl.checks.domains",),
    )
    return (1, violations) if violations else (0, [])


def check_checks_no_commands_import(repo_root: Path) -> tuple[int, list[str]]:
    checks_root = repo_root / "packages/atlasctl/src/atlasctl/checks"
    if not checks_root.exists():
        return 1, ["missing checks directory: packages/atlasctl/src/atlasctl/checks"]
    violations = _forbidden_imports(
        checks_root,
        include_glob="**/*.py",
        forbidden_prefixes=("atlasctl.commands",),
    )
    return (1, violations) if violations else (0, [])

CHECKS = (
    CheckDef(
        "checks.registry_integrity",
        "checks",
        "validate checks registry TOML/JSON integrity and drift",
        2500,
        check_registry_integrity,
        category=CheckCategory.CONTRACT,
        fix_hint="Run `./bin/atlasctl gen checks-registry` and commit REGISTRY.generated.json.",
        slow=True,
        owners=("platform",),
        tags=("checks", "registry"),
    ),
    CheckDef(
        "checks.registry_change_owners_gate",
        "checks",
        "require ownership metadata update when checks registry changes",
        500,
        check_registry_change_requires_owner_update,
        category=CheckCategory.POLICY,
        fix_hint="Update configs/make/ownership.json when REGISTRY.generated.json changes.",
        owners=("platform",),
        tags=("checks", "registry", "ci"),
    ),
    CheckDef(
        "checks.registry_change_docs_gate",
        "checks",
        "require docs update when checks registry changes",
        500,
        check_registry_change_requires_docs_update,
        category=CheckCategory.POLICY,
        fix_hint="Update docs/checks/registry.md when REGISTRY.generated.json changes.",
        owners=("platform",),
        tags=("checks", "registry", "ci"),
    ),
    CheckDef(
        "checks.registry_change_goldens_gate",
        "checks",
        "require checks list/tree/owners goldens update when registry changes",
        500,
        check_registry_change_requires_golden_update,
        category=CheckCategory.POLICY,
        fix_hint="Refresh check list/tree/owners goldens when REGISTRY.generated.json changes.",
        owners=("platform",),
        tags=("checks", "registry", "ci"),
    ),
    CheckDef(
        "checks.registry_all_checks_have_canonical_id",
        "checks",
        "require every registered check to expose a canonical checks_* id",
        700,
        check_registry_all_checks_have_canonical_id,
        category=CheckCategory.CONTRACT,
        fix_hint="Update registry ids to checks_<domain>_<area>_<name>.",
        owners=("platform",),
        tags=("checks", "registry", "required"),
    ),
    CheckDef(
        "checks.registry_no_duplicate_canonical_id",
        "checks",
        "forbid duplicate canonical check ids in registry",
        700,
        check_registry_no_duplicate_canonical_id,
        category=CheckCategory.CONTRACT,
        fix_hint="Deduplicate conflicting registry ids.",
        owners=("platform",),
        tags=("checks", "registry", "required"),
    ),
    CheckDef(
        "checks.registry_canonical_id_matches_module_path",
        "checks",
        "require canonical id domain/area segments to match implementation module path",
        900,
        check_registry_canonical_id_matches_module_path,
        category=CheckCategory.POLICY,
        fix_hint="Align check module paths with canonical check id segments.",
        owners=("platform",),
        tags=("checks", "registry", "required"),
    ),
    CheckDef(
        "checks.registry_owners_required",
        "checks",
        "require every registry entry to declare an owner",
        500,
        check_registry_owners_required,
        category=CheckCategory.CONTRACT,
        fix_hint="Add owner field to registry entries.",
        owners=("platform",),
        tags=("checks", "registry", "required"),
    ),
    CheckDef(
        "checks.registry_speed_required",
        "checks",
        "require speed classification for every registry entry (fast|slow|nightly)",
        500,
        check_registry_speed_required,
        category=CheckCategory.CONTRACT,
        fix_hint="Set speed to fast, slow, or nightly in the generated registry source.",
        owners=("platform",),
        tags=("checks", "registry", "required"),
    ),
    CheckDef(
        "checks.registry_suite_membership_required",
        "checks",
        "require every registry check to be selected by at least one suite manifest",
        900,
        check_registry_suite_membership_required,
        category=CheckCategory.POLICY,
        fix_hint="Add unassigned checks to at least one suite in registry/suites_catalog.json.",
        owners=("platform",),
        tags=("checks", "registry", "required"),
    ),
    CheckDef(
        "checks.registry_docs_link_required",
        "checks",
        "require docs_link metadata for every registry check entry",
        700,
        check_registry_docs_link_required,
        category=CheckCategory.CONTRACT,
        fix_hint="Set docs_link to an existing documentation page in checks catalog generation.",
        owners=("platform",),
        tags=("checks", "registry", "required"),
    ),
    CheckDef(
        "checks.registry_remediation_link_required",
        "checks",
        "require remediation_link metadata for every registry check entry",
        700,
        check_registry_remediation_link_required,
        category=CheckCategory.CONTRACT,
        fix_hint="Set remediation_link to an existing remediation playbook page.",
        owners=("platform",),
        tags=("checks", "registry", "required"),
    ),
    CheckDef(
        "checks.registry_docs_meta_runtime_drift",
        "checks",
        "require generated docs _meta checks registry file to match runtime registry",
        700,
        check_registry_docs_meta_matches_runtime,
        category=CheckCategory.DRIFT,
        fix_hint="Run `./bin/atlasctl gen checks-registry` and commit docs/_meta/checks-registry.txt.",
        owners=("platform",),
        tags=("checks", "registry", "required"),
    ),
    CheckDef(
        "checks.registry_transition_complete",
        "checks",
        "fail when check-id migration aliases remain after transition deadline",
        700,
        check_registry_transition_complete,
        category=CheckCategory.POLICY,
        fix_hint="Clear configs/policy/check-id-migration.json aliases after deadline or extend with explicit policy.",
        owners=("platform",),
        tags=("checks", "registry", "required"),
    ),
    CheckDef(
        "checks.suites_inventory_ssot",
        "checks",
        "require deterministic suite inventory in suite registry SSOT",
        700,
        check_suites_inventory_ssot,
        category=CheckCategory.CONTRACT,
        fix_hint="Sort suite names and remove duplicates in registry/suites_catalog.json.",
        owners=("platform",),
        tags=("checks", "suite", "required"),
    ),
    CheckDef(
        "checks.suites_no_orphans",
        "checks",
        "require every registered check to belong to at least one suite",
        700,
        check_suites_no_orphans,
        category=CheckCategory.POLICY,
        fix_hint="Add orphan checks to suite registry manifests.",
        owners=("platform",),
        tags=("checks", "suite", "required"),
    ),
    CheckDef(
        "checks.owners_ssot",
        "checks",
        "require every check entry to have an owner in SSOT registry",
        500,
        check_owners_ssot,
        category=CheckCategory.CONTRACT,
        fix_hint="Set owners for every check in the generated registry source.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.remediation_ssot",
        "checks",
        "require every check entry to have remediation documentation link",
        500,
        check_remediation_ssot,
        category=CheckCategory.CONTRACT,
        fix_hint="Set remediation_link for every check in checks catalog generation.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.docs_index_complete",
        "checks",
        "require checks docs index to include canonical check ids",
        700,
        check_docs_index_complete,
        category=CheckCategory.DRIFT,
        fix_hint="Regenerate/update packages/atlasctl/docs/checks/index.md with full canonical inventory.",
        owners=("platform",),
        tags=("checks", "docs", "required"),
    ),
    CheckDef(
        "checks.count_budget",
        "checks",
        "enforce checks-count ratchet budget from configs/policy/checks-count-budget.json",
        500,
        check_count_budget,
        category=CheckCategory.POLICY,
        fix_hint="Reduce checks count or intentionally raise budget in configs/policy/checks-count-budget.json.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.root_entry_budget",
        "checks",
        "enforce checks root entry budget",
        500,
        check_checks_root_entry_budget,
        category=CheckCategory.POLICY,
        fix_hint="Reduce top-level entries under packages/atlasctl/src/atlasctl/checks or adjust policy budget intentionally.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.domains_module_budget",
        "checks",
        "enforce checks domains module budget",
        500,
        check_checks_domains_module_budget,
        category=CheckCategory.POLICY,
        fix_hint="Keep checks domain module count within budget or adjust policy budget intentionally.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.tools_module_budget",
        "checks",
        "enforce checks tools module budget",
        500,
        check_checks_tools_module_budget,
        category=CheckCategory.POLICY,
        fix_hint="Keep checks tools module count within budget or adjust policy budget intentionally.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.module_loc_budget",
        "checks",
        "enforce checks module loc budget",
        600,
        check_checks_module_loc_budget,
        category=CheckCategory.POLICY,
        fix_hint="Split oversized checks modules into intent-focused modules.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.structured_results_legacy_signature_budget",
        "checks",
        "ratchet down legacy tuple check signatures in favor of structured results",
        600,
        check_structured_results_legacy_signature_budget,
        category=CheckCategory.POLICY,
        fix_hint="Migrate tuple[int, list[str]] check signatures to structured Violation/CheckOutcome results.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.domains_contracts_no_nested_dirs",
        "checks",
        "forbid nested directories under canonical domain contract modules",
        500,
        check_domains_contracts_no_nested_dirs,
        category=CheckCategory.POLICY,
        fix_hint="Keep domain contracts flat: avoid nested directories under checks/domains/ops/contracts and checks/domains/policies/make.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.registry_no_banned_adjectives_in_ids",
        "checks",
        "forbid policy-listed adjectives in canonical check ids",
        500,
        check_registry_no_banned_adjectives_in_ids,
        category=CheckCategory.POLICY,
        fix_hint="Rename check ids to remove policy-listed adjectives.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.registry_profile_tag_vocabulary",
        "checks",
        "require profile tag vocabulary to include ci dev internal lint slow fast",
        500,
        check_registry_profile_tag_vocabulary,
        category=CheckCategory.CONTRACT,
        fix_hint="Ensure check tags include ci/dev/internal/lint/slow/fast where applicable.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.surfaces_no_banned_adjectives_in_paths",
        "checks",
        "forbid policy-listed adjectives in atlasctl module/docs path names",
        500,
        check_surfaces_no_banned_adjectives_in_paths,
        category=CheckCategory.POLICY,
        fix_hint="Rename atlasctl source/docs files and directories to remove banned adjectives.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.commands_no_domains_import",
        "checks",
        "forbid command modules from importing checks domain modules directly",
        500,
        check_commands_no_domains_import,
        category=CheckCategory.POLICY,
        fix_hint="Use checks.registry APIs from command code; do not import checks.domains modules directly.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.checks_no_commands_import",
        "checks",
        "forbid checks modules from importing command modules",
        500,
        check_checks_no_commands_import,
        category=CheckCategory.POLICY,
        fix_hint="Keep checks implementation independent from command layer imports.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.no_raw_string_check_id_usage",
        "checks",
        "forbid raw string check_id/domain annotations outside model module",
        500,
        check_no_raw_string_check_id_usage,
        category=CheckCategory.POLICY,
        fix_hint="Use CheckId and DomainId wrapper types instead of raw str annotations.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.evidence_not_tracked",
        "checks",
        "forbid tracked evidence outputs under artifacts/atlasctl/checks",
        500,
        check_checks_evidence_not_tracked,
        category=CheckCategory.HYGIENE,
        fix_hint="Remove tracked evidence files under artifacts/atlasctl/checks and keep them runtime-only.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.generated_no_timestamp_dirs",
        "checks",
        "forbid timestamp-like directory names under generated roots",
        600,
        check_no_generated_timestamp_dirs,
        category=CheckCategory.HYGIENE,
        fix_hint="Use deterministic, stable generated directory names without timestamps.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.internal_checks_tree_policy",
        "checks",
        "enforce canonical checks tree and forbid legacy layout/repo/registry package trees",
        500,
        check_internal_checks_tree_policy,
        category=CheckCategory.POLICY,
        fix_hint="Keep only canonical checks tree and remove legacy checks/layout, checks/repo, and checks/registry package directories.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.internal_no_layout_repo_registry_dirs",
        "checks",
        "forbid checks layout/repo/registry legacy trees",
        500,
        check_internal_no_layout_repo_registry_dirs,
        category=CheckCategory.POLICY,
        fix_hint="Remove legacy checks/layout, checks/repo, and checks/registry package trees after migration.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.internal_checks_root_budget",
        "checks",
        "enforce checks package root entry budget",
        500,
        check_internal_checks_root_budget,
        category=CheckCategory.POLICY,
        fix_hint="Reduce checks package root entries to stay within budget.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.internal_root_budget",
        "checks",
        "enforce checks root entry budget alias contract",
        500,
        check_internal_root_budget,
        category=CheckCategory.POLICY,
        fix_hint="Reduce checks root entries to budget.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.internal_domains_flat_modules_only",
        "checks",
        "enforce flat modules under checks/domains",
        500,
        check_internal_domains_flat_modules_only,
        category=CheckCategory.POLICY,
        fix_hint="Flatten nested checks/domains packages into top-level domain modules.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.internal_domains_flat",
        "checks",
        "enforce flat checks domains tree",
        500,
        check_internal_domains_flat,
        category=CheckCategory.POLICY,
        fix_hint="Flatten checks/domains to top-level modules only.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.internal_no_file_per_check_explosion",
        "checks",
        "enforce python file-count budget per checks directory",
        600,
        check_internal_no_file_per_check_explosion,
        category=CheckCategory.POLICY,
        fix_hint="Group per-check files into intent modules and keep per-directory python file count below budget.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.internal_no_duplicate_engines",
        "checks",
        "enforce single canonical checks engine modules",
        500,
        check_internal_no_duplicate_engines,
        category=CheckCategory.POLICY,
        fix_hint="Keep only one canonical checks runner/policy/report/selectors module.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.internal_no_adapters_usage",
        "checks",
        "forbid adapter compatibility usage outside checks/adapters.py",
        500,
        check_internal_no_adapters_usage,
        category=CheckCategory.POLICY,
        fix_hint="Remove compatibility adapter imports from runtime checks code.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.internal_no_registry_toml_reads",
        "checks",
        "forbid runtime reads of REGISTRY.toml",
        500,
        check_internal_no_registry_toml_reads,
        category=CheckCategory.POLICY,
        fix_hint="Use python registry APIs at runtime and keep TOML reads generation-only.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.internal_no_generated_registry_as_input",
        "checks",
        "forbid runtime reads of REGISTRY.generated.json",
        500,
        check_internal_no_generated_registry_as_input,
        category=CheckCategory.POLICY,
        fix_hint="Use python registry APIs at runtime and keep generated registry json output-only.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.internal_single_runner_surface",
        "checks",
        "require command layer to use checks runner surface only",
        500,
        check_internal_single_runner_surface,
        category=CheckCategory.POLICY,
        fix_hint="Replace engine.runner imports in command modules with checks.runner APIs.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.internal_no_command_logic_in_checks",
        "checks",
        "forbid command-layer imports inside checks modules",
        500,
        check_internal_no_command_logic_in_checks,
        category=CheckCategory.POLICY,
        fix_hint="Keep checks modules independent from commands imports.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.internal_no_checks_logic_in_commands",
        "checks",
        "forbid commands from importing checks domain implementations directly",
        500,
        check_internal_no_checks_logic_in_commands,
        category=CheckCategory.POLICY,
        fix_hint="Use checks registry/runner surface from commands.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.internal_adapters_module_quarantined",
        "checks",
        "require explicit compatibility marker for checks adapters module",
        300,
        check_internal_adapters_module_quarantined,
        category=CheckCategory.POLICY,
        fix_hint="Keep adapters module migration-only and mark it explicitly as compatibility adapter.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.root_policy_compat_shim_expiry",
        "checks",
        "require root-policy compatibility shims to declare expiry and stay within deprecation window",
        500,
        check_root_policy_compat_shims_not_expired,
        category=CheckCategory.POLICY,
        fix_hint="Add compat_shim_expiry metadata for each compat shim or remove expired shims from checks/tools/root_policy.json.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.no_checks_outside_domains_tools",
        "checks",
        "require check implementations to live under checks/domains or checks/tools",
        700,
        check_no_checks_outside_domains_tools,
        category=CheckCategory.POLICY,
        fix_hint="Move check implementation modules under checks/domains or checks/tools and update registry callables.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.legacy_directories_absent",
        "checks",
        "require legacy checks/layout and checks/repo directories to be removed",
        500,
        check_legacy_check_directories_absent,
        category=CheckCategory.POLICY,
        fix_hint="Complete migration and delete checks/layout and checks/repo directories.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.registry_generated_read_only",
        "checks",
        "require generated registry json to keep generated markers",
        500,
        check_registry_generated_read_only,
        category=CheckCategory.CONTRACT,
        fix_hint="Regenerate REGISTRY.generated.json via atlasctl registry generator and keep generated metadata markers.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.registry_generated_is_readonly",
        "checks",
        "require generated registry json to be generator-owned",
        500,
        check_registry_generated_is_readonly,
        category=CheckCategory.CONTRACT,
        fix_hint="Regenerate REGISTRY.generated.json via atlasctl registry generator and keep generated metadata markers.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.all_checks_have_owner",
        "checks",
        "require every check to declare at least one owner",
        500,
        check_all_checks_have_owner,
        category=CheckCategory.CONTRACT,
        fix_hint="Populate owners for every check definition.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.all_checks_declare_effects",
        "checks",
        "require every check to declare effects",
        500,
        check_all_checks_declare_effects,
        category=CheckCategory.CONTRACT,
        fix_hint="Declare effect metadata for every check definition.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.all_checks_have_tags",
        "checks",
        "require every check to declare tags",
        500,
        check_all_checks_have_tags,
        category=CheckCategory.CONTRACT,
        fix_hint="Declare at least one policy tag for every check definition.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.subprocess_effect_declared",
        "checks",
        "require subprocess usage in check modules to declare subprocess effect",
        800,
        check_no_subprocess_usage_without_declared_effect,
        category=CheckCategory.POLICY,
        fix_hint="Add subprocess effect declaration for checks that call subprocess or run_command.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.network_effect_declared",
        "checks",
        "require network usage in check modules to declare network effect",
        800,
        check_no_network_usage_without_declared_effect,
        category=CheckCategory.POLICY,
        fix_hint="Add network effect declaration for checks that call network client libraries.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.write_roots_evidence_only",
        "checks",
        "require check write roots to stay under managed evidence directories",
        700,
        check_write_roots_are_evidence_only,
        category=CheckCategory.POLICY,
        fix_hint="Restrict writes_allowed_roots to artifacts/evidence, artifacts/atlasctl/checks, or artifacts/runs.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.registry_import_hygiene",
        "checks",
        "forbid heavy imports in checks registry modules",
        500,
        check_registry_import_hygiene,
        category=CheckCategory.POLICY,
        fix_hint="Keep checks registry imports lightweight and move heavy logic behind runtime boundaries.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.registry_cache_scope",
        "checks",
        "forbid lru cache usage outside registry index module",
        500,
        check_scattered_registry_caches,
        category=CheckCategory.POLICY,
        fix_hint="Scope caches to registry/catalog index and remove scattered cache decorators.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.internal_registry_ssot_only",
        "checks",
        "isolate legacy registry bridge usage to SSOT bridge modules",
        500,
        check_internal_registry_ssot_only,
        category=CheckCategory.POLICY,
        fix_hint="Route registry access through checks.registry and remove registry_legacy import references.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.docs_checks_no_ops_imports",
        "checks",
        "forbid docs checks from importing ops implementation modules directly",
        500,
        check_docs_checks_no_ops_imports,
        category=CheckCategory.POLICY,
        fix_hint="Keep docs checks independent from ops modules and rely on documented contracts/inputs.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.file_count_budget",
        "checks",
        "enforce checks package file-count budget",
        500,
        check_checks_file_count_budget,
        category=CheckCategory.POLICY,
        fix_hint="Reduce checks package file count to budget by consolidating modules.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.depth_budget",
        "checks",
        "enforce checks package depth budget",
        500,
        check_checks_tree_depth_budget,
        category=CheckCategory.POLICY,
        fix_hint="Flatten checks package directories to depth budget.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.domains_shape",
        "checks",
        "enforce flat domains directory shape",
        500,
        check_domains_directory_shape,
        category=CheckCategory.POLICY,
        fix_hint="Keep checks/domains as flat python modules only.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.no_cross_domain_relative_imports",
        "checks",
        "forbid cross-domain relative imports in checks domains",
        500,
        check_no_relative_imports_across_domains,
        category=CheckCategory.POLICY,
        fix_hint="Use absolute imports rooted at atlasctl.checks.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.no_ops_runtime_asset_imports",
        "checks",
        "forbid checks package imports from ops runtime asset modules",
        500,
        check_no_ops_runtime_asset_imports,
        category=CheckCategory.POLICY,
        fix_hint="Keep checks package independent from ops runtime asset modules.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.no_tests_fixtures_imports",
        "checks",
        "forbid checks package imports from tests/fixtures paths",
        500,
        check_no_tests_fixtures_imports,
        category=CheckCategory.POLICY,
        fix_hint="Remove tests/fixtures imports from checks package modules.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.no_unused_imports",
        "checks",
        "forbid unused imports in checks package modules",
        700,
        check_unused_imports_in_checks,
        category=CheckCategory.POLICY,
        fix_hint="Remove unused imports from checks package modules.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.import_cycles",
        "checks",
        "forbid import cycles across atlasctl checks modules",
        700,
        check_checks_import_cycles,
        category=CheckCategory.POLICY,
        fix_hint="Break circular imports across atlasctl checks modules by extracting shared contracts/tools.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.forbidden_imports",
        "checks",
        "forbid checks imports from ops test/fixture surfaces",
        500,
        check_checks_forbidden_imports,
        category=CheckCategory.POLICY,
        fix_hint="Remove checks imports from test/fixture surfaces and keep checks dependent on runtime contracts only.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.no_print_calls",
        "checks",
        "forbid print calls inside checks package modules",
        600,
        check_checks_no_print_calls,
        category=CheckCategory.POLICY,
        fix_hint="Remove print calls from checks code and emit structured violations/results instead.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.no_sys_exit_calls",
        "checks",
        "forbid sys.exit calls inside checks package modules",
        500,
        check_checks_no_sys_exit_calls,
        category=CheckCategory.POLICY,
        fix_hint="Replace sys.exit usage with structured check results and propagated command exit codes.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.no_direct_env_reads",
        "checks",
        "forbid direct environment reads in checks package modules",
        500,
        check_checks_no_direct_env_reads,
        category=CheckCategory.POLICY,
        fix_hint="Read environment via CheckContext.env rather than os.environ/os.getenv inside checks modules.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.no_path_dot_usage",
        "checks",
        "forbid Path('.') usage in checks package modules",
        500,
        check_checks_no_path_dot_usage,
        category=CheckCategory.POLICY,
        fix_hint="Resolve paths from explicit repo_root and avoid dot-path construction.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.no_cwd_reliance",
        "checks",
        "forbid cwd-dependent path calls in checks package modules",
        500,
        check_checks_no_cwd_reliance,
        category=CheckCategory.POLICY,
        fix_hint="Use explicit repo_root context and avoid Path.cwd/os.getcwd/os.chdir in checks modules.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.write_effect_declared",
        "checks",
        "require fs_write effect declaration for checks that perform file writes",
        700,
        check_write_effect_declared_for_writing_checks,
        category=CheckCategory.POLICY,
        fix_hint="Declare fs_write effect for checks that write files.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.no_duplicate_model_definitions",
        "checks",
        "forbid duplicate model class definitions outside checks/model.py",
        600,
        check_no_duplicate_model_definitions,
        category=CheckCategory.POLICY,
        fix_hint="Keep model contracts in checks/model.py and remove duplicate class definitions from other modules.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.root_allowed_entries_only",
        "checks",
        "require checks root to contain only canonical modules and directories",
        500,
        check_checks_root_allowed_entries_only,
        category=CheckCategory.POLICY,
        fix_hint="Keep only canonical files and directories in packages/atlasctl/src/atlasctl/checks root.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.approved_top_level_entries_only",
        "checks",
        "require checks package top-level entries to match approved allowlist",
        500,
        check_checks_approved_top_level_entries_only,
        category=CheckCategory.POLICY,
        fix_hint="Remove non-canonical top-level entries under packages/atlasctl/src/atlasctl/checks.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.registry_generated_canonical_hash",
        "checks",
        "require generated registry json to use deterministic canonical formatting",
        500,
        check_registry_generated_canonical_hash,
        category=CheckCategory.CONTRACT,
        fix_hint="Regenerate checks registry using atlasctl generator and commit canonical output.",
        owners=("platform",),
        tags=("checks", "registry", "required"),
    ),
    CheckDef(
        "checks.all_checks_have_docs_string",
        "checks",
        "require every check definition to provide a non-empty description string",
        500,
        check_all_checks_have_docs_string,
        category=CheckCategory.CONTRACT,
        fix_hint="Set a concise non-empty description for each check definition.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.no_subprocess_import_outside_adapters",
        "checks",
        "forbid subprocess imports outside approved checks adapter modules",
        500,
        check_no_subprocess_import_outside_adapters,
        category=CheckCategory.POLICY,
        fix_hint="Move subprocess access into checks adapters and import through adapter interfaces.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.no_network_client_import_outside_adapters",
        "checks",
        "forbid requests/httpx imports outside approved checks adapter modules",
        500,
        check_no_network_client_import_outside_adapters,
        category=CheckCategory.POLICY,
        fix_hint="Move network client usage into checks adapters and keep checks modules pure.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
    CheckDef(
        "checks.no_tools_domain_mirrors",
        "checks",
        "forbid checks/tools mirrored domain directories",
        500,
        check_no_tools_domain_mirrors_exist,
        category=CheckCategory.POLICY,
        fix_hint="Remove checks/tools/*_domain mirrors and keep canonical helpers under checks/tools.",
        owners=("platform",),
        tags=("checks", "required"),
    ),
)

def register() -> tuple[CheckDef, ...]:
    return CHECKS


__all__ = ["CHECKS", "register"]
