from __future__ import annotations

import json
from dataclasses import dataclass
from datetime import date
from pathlib import Path

ROOT_POLICY_PATH = Path("packages/atlasctl/src/atlasctl/checks/tools/root_policy.json")


@dataclass(frozen=True)
class RootPolicy:
    required: frozenset[str]
    allowed: frozenset[str]
    compat_shims: frozenset[str]
    compat_shim_expiry: dict[str, date]
    local_noise: frozenset[str]

    @property
    def all_allowed(self) -> frozenset[str]:
        return self.required | self.allowed | self.compat_shims | self.local_noise


def _load_list(payload: dict[str, object], key: str) -> frozenset[str]:
    value = payload.get(key, [])
    if not isinstance(value, list):
        raise ValueError(f"root policy field '{key}' must be a JSON list")
    normalized: list[str] = []
    for entry in value:
        if not isinstance(entry, str) or not entry.strip():
            raise ValueError(f"root policy field '{key}' contains a non-string or empty entry")
        normalized.append(entry.strip())
    return frozenset(normalized)


def _load_compat_shim_expiry(payload: dict[str, object], compat_shims: frozenset[str]) -> dict[str, date]:
    raw = payload.get("compat_shim_expiry", {})
    if not isinstance(raw, dict):
        raise ValueError("root policy field 'compat_shim_expiry' must be a JSON object")
    out: dict[str, date] = {}
    for name, value in raw.items():
        shim = str(name).strip()
        if shim not in compat_shims:
            raise ValueError(f"compat_shim_expiry entry references unknown compat shim `{shim}`")
        if not isinstance(value, str) or not value.strip():
            raise ValueError(f"compat_shim_expiry `{shim}` must be a non-empty date string")
        try:
            out[shim] = date.fromisoformat(value.strip())
        except ValueError as exc:
            raise ValueError(f"compat_shim_expiry `{shim}` must use YYYY-MM-DD format") from exc
    return out


def load_root_policy(repo_root: Path) -> RootPolicy:
    policy_path = repo_root / ROOT_POLICY_PATH
    payload = json.loads(policy_path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError("root policy payload must be a JSON object")

    compat_shims = _load_list(payload, "compat_shims")
    policy = RootPolicy(
        required=_load_list(payload, "required"),
        allowed=_load_list(payload, "allowed"),
        compat_shims=compat_shims,
        compat_shim_expiry=_load_compat_shim_expiry(payload, compat_shims),
        local_noise=_load_list(payload, "local_noise"),
    )
    overlap = (policy.required & policy.allowed) | (policy.required & policy.compat_shims) | (
        policy.allowed & policy.compat_shims
    )
    if overlap:
        names = ", ".join(sorted(overlap))
        raise ValueError(f"root policy contains overlapping required/allowed/compat entries: {names}")
    return policy
