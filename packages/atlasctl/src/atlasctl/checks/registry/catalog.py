from __future__ import annotations

import inspect
import json
import re
from collections import defaultdict
from dataclasses import replace
from pathlib import Path

from ..core.base import CheckDef
from ..contracts import CHECKS as CHECKS_CONTRACTS
from ..configs import CHECKS as CHECKS_CONFIGS
from ..docker import CHECKS as CHECKS_DOCKER
from ..docs import CHECKS as CHECKS_DOCS
from ..licensing import CHECKS as CHECKS_LICENSE
from ..make import CHECKS as CHECKS_MAKE
from ..ops import CHECKS as CHECKS_OPS
from ..python import CHECKS as CHECKS_PYTHON
from ..repo import CHECKS as CHECKS_REPO


_CHECKS: tuple[CheckDef, ...] = (
    *CHECKS_REPO,
    *CHECKS_LICENSE,
    *CHECKS_MAKE,
    *CHECKS_DOCS,
    *CHECKS_OPS,
    *CHECKS_CONFIGS,
    *CHECKS_PYTHON,
    *CHECKS_DOCKER,
    *CHECKS_CONTRACTS,
)

_STABLE_TAGS = {"docs", "dev", "ops", "policies", "configs", "internal"}
_DOMAIN_TO_STABLE_TAG = {
    "repo": "policies",
    "license": "policies",
    "make": "dev",
    "docs": "docs",
    "ops": "ops",
    "configs": "configs",
    "python": "dev",
    "docker": "ops",
    "contracts": "policies",
}
_RENAMES_PATH = Path("configs/policy/target-renames.json")
_FILENAME_ALLOWLIST_PATH = Path("configs/policy/check-filename-allowlist.json")
_DOMAIN_PATH_ALLOW = {
    "license": ("/checks/licensing/",),
    "repo": ("/checks/repo/", "/checks/layout/root/"),
    "configs": ("/checks/configs/", "/checks/repo/contracts/"),
}
_FORBIDDEN_TAGS = {"refgrade", "refgrade_required", "elite"}
_DEFAULT_DOMAIN_OWNER = {
    "repo": "platform",
    "make": "platform",
    "docs": "docs",
    "ops": "ops",
    "configs": "platform",
    "python": "platform",
    "docker": "ops",
    "contracts": "platform",
    "license": "platform",
}


def _default_new_check_id(check: CheckDef) -> str:
    token = check.check_id.strip().replace(".", "_").replace("-", "_")
    token = re.sub(r"[^a-zA-Z0-9_]+", "_", token)
    token = re.sub(r"_+", "_", token).strip("_").lower()
    if token.startswith("checks_"):
        return token
    if token.startswith(f"{check.domain}_"):
        return f"checks_{token}"
    return f"checks_{check.domain}_{token}"


def _load_rename_overrides(repo_root: Path) -> dict[str, str]:
    path = repo_root / _RENAMES_PATH
    if not path.exists():
        return {}
    payload = json.loads(path.read_text(encoding="utf-8"))
    check_ids = payload.get("check_ids", {})
    if not isinstance(check_ids, dict):
        return {}
    return {str(old): str(new) for old, new in check_ids.items()}


def _canonical_checks() -> tuple[tuple[CheckDef, ...], dict[str, str]]:
    repo_root = Path(__file__).resolve().parents[6]
    overrides = _load_rename_overrides(repo_root)
    out: list[CheckDef] = []
    aliases: dict[str, str] = {}
    for check in _CHECKS:
        new_id = overrides.get(check.check_id) or _default_new_check_id(check)
        legacy = check.check_id if new_id != check.check_id else None
        owners = check.owners or (_DEFAULT_DOMAIN_OWNER.get(check.domain, "platform"),)
        out.append(replace(check, check_id=new_id, legacy_check_id=legacy, owners=owners))
        if legacy:
            aliases[legacy] = new_id
    return tuple(out), aliases


_CHECKS_CANON, _ALIASES = _canonical_checks()


def _filename_allowlist() -> set[str]:
    repo_root = Path(__file__).resolve().parents[6]
    path = repo_root / _FILENAME_ALLOWLIST_PATH
    if not path.exists():
        return set()
    payload = json.loads(path.read_text(encoding="utf-8"))
    names = payload.get("allowlist", [])
    return {str(item) for item in names if str(item).strip()}


def check_tags(check: CheckDef) -> tuple[str, ...]:
    tags = set(check.tags)
    tags.add(check.domain)
    tags.add(_DOMAIN_TO_STABLE_TAG.get(check.domain, "internal"))
    if "internal" not in tags and "internal-only" not in tags:
        tags.add("required")
    if check.slow:
        tags.add("slow")
    else:
        tags.add("fast")
    tags.update(sorted(tag for tag in check.tags if tag in _STABLE_TAGS))
    return tuple(sorted(tags))


def list_checks() -> tuple[CheckDef, ...]:
    seen: set[str] = set()
    duplicates: set[str] = set()
    errors: list[str] = []
    filename_allowlist = _filename_allowlist()
    for check in _CHECKS_CANON:
        if check.check_id in seen:
            duplicates.add(check.check_id)
        seen.add(check.check_id)
        if not check.check_id.startswith("checks_"):
            errors.append(f"{check.check_id}: check id must start with `checks_`")
        expected_prefix = f"checks_{check.domain}_"
        if not check.check_id.startswith(expected_prefix):
            errors.append(f"{check.check_id}: check id must include matching domain prefix `{expected_prefix}`")
        source = inspect.getsourcefile(check.fn) or ""
        rel = source.replace("\\", "/")
        if "/checks/" in rel:
            allowed = _DOMAIN_PATH_ALLOW.get(check.domain, (f"/checks/{check.domain}/",))
            if not any(token in rel for token in allowed):
                errors.append(f"{check.check_id}: check fn path must match domain `{check.domain}` allowlist (got {rel})")
        name = Path(rel).name
        should_enforce_name = bool(check.legacy_check_id is None)
        if should_enforce_name and name not in {"", "<string>"} and not name.startswith("check_") and name not in filename_allowlist:
            errors.append(f"{check.check_id}: filename must start with `check_` or be allowlisted (got {name})")
        if not check.owners:
            errors.append(f"{check.check_id}: check must declare at least one owner")
        if check.severity.value not in {"error", "warn", "info"}:
            errors.append(f"{check.check_id}: unsupported severity `{check.severity.value}`")
        if not isinstance(check.slow, bool):
            errors.append(f"{check.check_id}: slow must be boolean")
        forbidden = _FORBIDDEN_TAGS.intersection(set(check.tags))
        if forbidden:
            errors.append(f"{check.check_id}: forbidden tags present: {', '.join(sorted(forbidden))}")
    if duplicates:
        dup_list = ", ".join(sorted(duplicates))
        raise ValueError(f"duplicate check ids in registry: {dup_list}")
    if errors:
        raise ValueError("check registry invariants failed: " + "; ".join(sorted(set(errors))))
    return tuple(sorted(_CHECKS_CANON, key=lambda c: c.check_id))


def list_domains() -> list[str]:
    return sorted({"all", *{c.domain for c in _CHECKS_CANON}})


def checks_by_domain() -> dict[str, list[CheckDef]]:
    grouped: dict[str, list[CheckDef]] = defaultdict(list)
    for check in _CHECKS_CANON:
        grouped[check.domain].append(check)
    return dict(grouped)


def run_checks_for_domain(repo_root: Path, domain: str) -> list[CheckDef]:
    if domain == "all":
        return list(list_checks())
    return [c for c in list_checks() if c.domain == domain]


def get_check(check_id: str) -> CheckDef | None:
    if check_id in _ALIASES:
        check_id = _ALIASES[check_id]
    for check in list_checks():
        if check.check_id == check_id:
            return check
    return None


def check_rename_aliases() -> dict[str, str]:
    return dict(sorted(_ALIASES.items()))
