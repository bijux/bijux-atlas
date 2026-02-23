from __future__ import annotations

from enum import Enum


class CheckEffect(str, Enum):
    FS_READ = "fs_read"
    FS_WRITE = "fs_write"
    SUBPROCESS = "subprocess"
    NETWORK = "network"


LEGACY_EFFECT_ALIASES: dict[str, str] = {
    "read": CheckEffect.FS_READ.value,
    "write": CheckEffect.FS_WRITE.value,
    "fs-write": CheckEffect.FS_WRITE.value,
    "process": CheckEffect.SUBPROCESS.value,
}


def normalize_effect(value: str) -> str:
    raw = str(value).strip().lower()
    return LEGACY_EFFECT_ALIASES.get(raw, raw)

