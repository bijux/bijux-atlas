from __future__ import annotations

import json

from atlasctl.core.runtime.paths import write_text_file
from atlasctl.commands.ops.runtime_modules import ops_runtime_commands as legacy
from atlasctl.ops.adapters import k6
from atlasctl.ops.models import OpsTestReport
from .paths import ops_run_area_dir

from .guards import ensure_local_kind_context


def _emit_test_report(ctx, area: str, report, report_format: str) -> int:  # noqa: ANN001
    payload = report.to_payload()
    out = ops_run_area_dir(ctx, area) / "report.json"
    write_text_file(out, json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    if report_format == "json":
        print(json.dumps({**payload, "report_path": out.relative_to(ctx.repo_root).as_posix()}, sort_keys=True))
    else:
        print(out.relative_to(ctx.repo_root).as_posix())
    return 0 if report.status in {"ok", "pass"} else 1


def test_smoke(ctx, report_format: str) -> int:  # noqa: ANN001
    ok, message = ensure_local_kind_context(ctx)
    if not ok:
        return legacy._emit_ops_status(report_format, 2, message)
    code = legacy._ops_smoke_native(ctx, report_format, reuse=True)
    report = OpsTestReport(
        test_kind="smoke",
        run_id=ctx.run_id,
        status="pass" if code == 0 else "fail",
        command=["atlasctl", "ops", "e2e", "run", "--suite", "smoke"],
    )
    return _emit_test_report(ctx, "ops-smoke", report, report_format) if code == 0 else code


def test_e2e(ctx, report_format: str) -> int:  # noqa: ANN001
    ok, message = ensure_local_kind_context(ctx)
    if not ok:
        return legacy._emit_ops_status(report_format, 2, message)
    code = legacy._ops_e2e_run_native(ctx, report_format, "smoke")
    report = OpsTestReport(
        test_kind="e2e",
        run_id=ctx.run_id,
        status="pass" if code == 0 else "fail",
        command=["atlasctl", "suite", "run", "ci:ops-fast"],
    )
    return _emit_test_report(ctx, "ops-e2e", report, report_format) if code == 0 else code


def test_load(ctx, report_format: str) -> int:  # noqa: ANN001
    ok, message = ensure_local_kind_context(ctx)
    if not ok:
        return legacy._emit_ops_status(report_format, 2, message)
    k6_version = k6.run(ctx, "version")
    if k6_version.code != 0:
        return legacy._emit_ops_status(report_format, k6_version.code, k6_version.combined_output)
    out_dir = "artifacts/perf/results"
    threshold_dir = ctx.repo_root / "ops" / "load" / "thresholds"
    threshold_files = sorted(p.name for p in threshold_dir.glob("*") if p.is_file()) if threshold_dir.exists() else []
    code = legacy._ops_load_run_native(ctx, report_format, "mixed-80-20", out_dir)
    report = OpsTestReport(
        test_kind="load",
        run_id=ctx.run_id,
        status="pass" if code == 0 else "fail",
        command=["k6", "run", "ops/load/scenarios/mixed-80-20.js"],
        out_rel=out_dir,
        evidence=[f"ops/load/thresholds/{name}" for name in threshold_files],
    )
    return _emit_test_report(ctx, "ops-load", report, report_format) if code == 0 else code
