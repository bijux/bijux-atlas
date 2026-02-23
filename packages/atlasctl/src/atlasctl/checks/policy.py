from __future__ import annotations

from dataclasses import dataclass

from .model import CheckDef, Effect


@dataclass(frozen=True)
class EffectPolicy:
    allow_fs_write: bool = False
    allow_subprocess: bool = False
    allow_network: bool = False


def evaluate_effects(check: CheckDef, policy: EffectPolicy) -> tuple[bool, list[str]]:
    declared = {str(effect) for effect in check.effects}
    errors: list[str] = []
    if Effect.FS_WRITE.value in declared and not policy.allow_fs_write:
        errors.append(f"{check.id}: fs_write declared but write capability is disabled")
    if Effect.SUBPROCESS.value in declared and not policy.allow_subprocess:
        errors.append(f"{check.id}: subprocess declared but process capability is disabled")
    if Effect.NETWORK.value in declared and not policy.allow_network:
        errors.append(f"{check.id}: network declared but network capability is disabled")
    return (not errors, errors)


__all__ = ["EffectPolicy", "evaluate_effects"]
