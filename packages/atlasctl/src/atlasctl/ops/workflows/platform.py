from __future__ import annotations

from atlasctl.commands.ops.runtime_modules import ops_runtime_commands as legacy

from .guards import ensure_local_kind_context, require_destructive_ops_allowed


def platform_up(ctx, report_format: str) -> int:  # noqa: ANN001
    ok, message = ensure_local_kind_context(ctx)
    if not ok:
        return legacy._emit_ops_status(report_format, 2, message)
    return legacy._ops_stack_up_native(ctx, report_format, "kind", reuse=False)


def platform_down(ctx, report_format: str) -> int:  # noqa: ANN001
    ok, message = require_destructive_ops_allowed("ops platform down")
    if not ok:
        return legacy._emit_ops_status(report_format, 2, message)
    ok, context = ensure_local_kind_context(ctx)
    if not ok:
        return legacy._emit_ops_status(report_format, 2, context)
    return legacy._ops_stack_down_native(ctx, report_format)
