from __future__ import annotations

import json
import re
from pathlib import Path


_CANONICAL_RE = re.compile(r"^checks_[a-z0-9]+_[a-z0-9]+_[a-z0-9_]+$")
_CATALOG_PATH = Path("packages/atlasctl/src/atlasctl/registry/checks_catalog.json")
_DOCS_META_PATH = Path("packages/atlasctl/docs/_meta/checks-registry.txt")
_COUNT_BUDGET_PATH = Path("configs/policy/checks-count-budget.json")


def check_registry_integrity(repo_root: Path) -> tuple[int, list[str]]:
    from ....registry.ssot import generate_registry_json

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
    from ....registry.ssot import load_registry_entries

    return load_registry_entries(repo_root)


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
    from .....registry.suites import resolve_check_ids, suite_manifest_specs

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
    from ....registry.ssot import check_id_alias_expiry, check_id_renames
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
    from .....registry.suites import suite_manifest_specs

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
