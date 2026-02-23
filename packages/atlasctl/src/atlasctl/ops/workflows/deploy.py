from __future__ import annotations

from atlasctl.commands.ops.runtime_modules import ops_runtime_commands as legacy

from .guards import ensure_local_kind_context


def deploy_stack(ctx, report_format: str) -> int:  # noqa: ANN001
    ok, message = ensure_local_kind_context(ctx)
    if not ok:
        return legacy._emit_ops_status(report_format, 2, message)
    return legacy._ops_stack_up_native(ctx, report_format, "kind", reuse=True)


def deploy_atlas(ctx, report_format: str) -> int:  # noqa: ANN001
    ok, message = ensure_local_kind_context(ctx)
    if not ok:
        return legacy._emit_ops_status(report_format, 2, message)
    return legacy._ops_deploy_native(ctx, report_format)
