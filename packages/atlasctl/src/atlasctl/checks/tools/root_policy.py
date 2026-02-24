from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path

ROOT_POLICY_PATH = Path("packages/atlasctl/src/atlasctl/checks/tools/root_policy.json")


@dataclass(frozen=True)
class RootPolicy:
    required: frozenset[str]
    allowed: frozenset[str]
    compat_shims: frozenset[str]
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


def load_root_policy(repo_root: Path) -> RootPolicy:
    policy_path = repo_root / ROOT_POLICY_PATH
    payload = json.loads(policy_path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError("root policy payload must be a JSON object")

    policy = RootPolicy(
        required=_load_list(payload, "required"),
        allowed=_load_list(payload, "allowed"),
        compat_shims=_load_list(payload, "compat_shims"),
        local_noise=_load_list(payload, "local_noise"),
    )
    overlap = (policy.required & policy.allowed) | (policy.required & policy.compat_shims) | (
        policy.allowed & policy.compat_shims
    )
    if overlap:
        names = ", ".join(sorted(overlap))
        raise ValueError(f"root policy contains overlapping required/allowed/compat entries: {names}")
    return policy

