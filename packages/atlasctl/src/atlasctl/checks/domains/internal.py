from __future__ import annotations

import ast
import json
import os
import re
from pathlib import Path

from ...core.process import run_command
from ..model import CheckCategory, CheckDef


REGISTRY_PATH = "packages/atlasctl/src/atlasctl/checks/REGISTRY.toml"
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


def check_registry_integrity(repo_root: Path) -> tuple[int, list[str]]:
    from ..registry.ssot import generate_registry_json

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
    from ..registry.ssot import load_registry_entries

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
    from ..registry.ssot import check_id_alias_expiry, check_id_renames
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
    ignored = {"__pycache__", "__init__.py", "README.md", "REGISTRY.toml", "REGISTRY.generated.json", "api.py"}
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
        fix_hint="Run `./bin/atlasctl gen checks-registry` and commit REGISTRY.toml/REGISTRY.generated.json.",
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
        fix_hint="Update configs/make/ownership.json when REGISTRY.toml changes.",
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
        fix_hint="Update docs/checks/registry.md when REGISTRY.toml changes.",
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
        fix_hint="Refresh check list/tree/owners goldens when REGISTRY.toml changes.",
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
        fix_hint="Update REGISTRY.toml ids to checks_<domain>_<area>_<name>.",
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
        fix_hint="Deduplicate conflicting ids in REGISTRY.toml.",
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
        fix_hint="Set speed to fast, slow, or nightly in REGISTRY.toml.",
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
        fix_hint="Set owners for every check in REGISTRY.toml.",
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
)

__all__ = ["CHECKS"]
