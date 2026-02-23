from __future__ import annotations

from atlasctl.core.context import RunContext
from atlasctl.core.errors import ScriptError
from atlasctl.core.exit_codes import ERR_CONFIG
from atlasctl.core.runtime.env import getenv


def enforce_context_safety_scaffold(ctx: RunContext, command_name: str) -> None:
    if getenv("ATLASCTL_REQUIRE_KIND_CONTEXT", "").strip() not in {"1", "true", "yes"}:
        return
    if command_name not in {"ops", "k8s"}:
        return
    current = getenv("ATLASCTL_KUBE_CONTEXT", "").strip()
    if current and current.startswith("kind-"):
        return
    raise ScriptError(
        "context safety check failed: destructive ops scaffolding requires kind context (set ATLASCTL_KUBE_CONTEXT=kind-<name>)",
        ERR_CONFIG,
    )

