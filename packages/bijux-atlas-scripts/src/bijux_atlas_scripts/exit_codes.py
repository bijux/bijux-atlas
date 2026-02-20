from __future__ import annotations

# Atlasctl process exit code contract.
OK = 0
ERR_USER = 2
ERR_CONTRACT = 3
ERR_PREREQ = 10
ERR_INTERNAL = 20

# Backward-compatible aliases used across modules.
ERR_CONFIG = ERR_USER
ERR_CONTEXT = ERR_USER
ERR_VERSION = ERR_PREREQ
ERR_TIMEOUT = ERR_INTERNAL
ERR_VALIDATION = ERR_CONTRACT
ERR_ARTIFACT = ERR_CONTRACT
ERR_DOCS = ERR_CONTRACT
