from __future__ import annotations

from dataclasses import dataclass
from typing import Iterable

from .model import CheckDef, Effect
from .evidence import DEFAULT_WRITE_ROOT


@dataclass(frozen=True)
class EffectPolicy:
    allowed_effects: frozenset[str]
    allowed_write_roots: tuple[str, ...] = (DEFAULT_WRITE_ROOT,)
    allow_network: bool = False
    allow_subprocess: bool = False
    forbid_print: bool = True

    @classmethod
    def default(cls) -> EffectPolicy:
        return cls(allowed_effects=frozenset({Effect.FS_READ.value}))


@dataclass(frozen=True)
class Capabilities:
    allow_network: bool = False
    allow_subprocess: bool = False
    allow_fs_write: bool = False
    write_roots: tuple[str, ...] = (DEFAULT_WRITE_ROOT,)

    @classmethod
    def from_cli_flags(
        cls,
        *,
        allow_network: bool = False,
        allow_process: bool = False,
        allow_write: bool = False,
        write_roots: Iterable[str] = (DEFAULT_WRITE_ROOT,),
    ) -> Capabilities:
        roots = tuple(str(item).strip() for item in write_roots if str(item).strip()) or (DEFAULT_WRITE_ROOT,)
        return cls(
            allow_network=bool(allow_network),
            allow_subprocess=bool(allow_process),
            allow_fs_write=bool(allow_write),
            write_roots=roots,
        )

    def to_effect_policy(self) -> EffectPolicy:
        effects = {Effect.FS_READ.value}
        if self.allow_fs_write:
            effects.add(Effect.FS_WRITE.value)
        if self.allow_subprocess:
            effects.add(Effect.SUBPROCESS.value)
        if self.allow_network:
            effects.add(Effect.NETWORK.value)
        return EffectPolicy(
            allowed_effects=frozenset(effects),
            allowed_write_roots=self.write_roots,
            allow_network=self.allow_network,
            allow_subprocess=self.allow_subprocess,
        )


def evaluate_effects(check: CheckDef, policy: EffectPolicy) -> tuple[bool, list[str]]:
    declared = {str(effect) for effect in check.effects}
    missing = sorted(effect for effect in declared if effect not in policy.allowed_effects)
    if not missing:
        return True, []
    return False, [f"{check.id}: effect not allowed by current capabilities: {effect}" for effect in missing]


def default_write_roots() -> tuple[str, ...]:
    return (DEFAULT_WRITE_ROOT,)


__all__ = ["Capabilities", "EffectPolicy", "default_write_roots", "evaluate_effects"]
