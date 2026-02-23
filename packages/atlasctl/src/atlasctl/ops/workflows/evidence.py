from __future__ import annotations

import json
from pathlib import Path

from atlasctl.commands.ops._shared.artifacts import ops_evidence_dir
from atlasctl.core.runtime.paths import write_text_file
from atlasctl.ops.adapters import kubectl
from atlasctl.ops.models import OpsEvidenceCollectReport

from .guards import ensure_local_kind_context


def _json_or_empty(raw: str) -> dict[str, object]:
    try:
        payload = json.loads(raw)
        return payload if isinstance(payload, dict) else {}
    except Exception:
        return {}


def collect_evidence(ctx, report_format: str, namespace: str = "atlas-e2e") -> int:  # noqa: ANN001
    ok, message = ensure_local_kind_context(ctx)
    if not ok:
        from atlasctl.commands.ops.runtime_modules import ops_runtime_commands as legacy

        return legacy._emit_ops_status(report_format, 2, message)

    out_dir = ops_evidence_dir(ctx, "ops-evidence")
    pods_json = kubectl.run(ctx, "-n", namespace, "get", "pods", "-o", "json")
    events_txt = kubectl.run(ctx, "get", "events", "-n", namespace, "--sort-by=.lastTimestamp")
    pod_payload = _json_or_empty(pods_json.stdout)
    pod_names: list[str] = []
    for item in pod_payload.get("items", []) if isinstance(pod_payload.get("items", []), list) else []:
        meta = item.get("metadata", {}) if isinstance(item, dict) else {}
        name = meta.get("name")
        if isinstance(name, str) and name.strip():
            pod_names.append(name.strip())

    logs_dir = out_dir / "logs"
    logs_dir.mkdir(parents=True, exist_ok=True)
    for pod in sorted(set(pod_names)):
        log_res = kubectl.run(ctx, "-n", namespace, "logs", pod, "--tail=2000")
        write_text_file(logs_dir / f"{pod}.log", log_res.stdout if log_res.code == 0 else "", encoding="utf-8")

    events_path = out_dir / "events.txt"
    write_text_file(events_path, events_txt.stdout if events_txt.code == 0 else "", encoding="utf-8")
    pods_path = out_dir / "pods.json"
    write_text_file(pods_path, pods_json.stdout if pods_json.code == 0 else "{}", encoding="utf-8")

    metrics_snapshot = ctx.repo_root / "artifacts" / "evidence" / "obs" / ctx.run_id / "metrics.json"
    traces_snapshot = ctx.repo_root / "artifacts" / "evidence" / "obs" / ctx.run_id / "traces.json"

    def _rel(path: Path) -> str:
        try:
            return path.relative_to(ctx.repo_root).as_posix()
        except ValueError:
            return path.as_posix()

    report = OpsEvidenceCollectReport(
        run_id=ctx.run_id,
        status="ok",
        namespace=namespace,
        pods=sorted(set(pod_names)),
        attachments={
            "pods_json": _rel(pods_path),
            "events_txt": _rel(events_path),
            "logs_dir": _rel(logs_dir),
            "metrics_snapshot_pointer": _rel(metrics_snapshot),
            "traces_snapshot_pointer": _rel(traces_snapshot),
        },
    )
    report_path = out_dir / "report.json"
    write_text_file(report_path, json.dumps(report.to_payload(), indent=2, sort_keys=True) + "\n", encoding="utf-8")
    if report_format == "json":
        print(json.dumps({**report.to_payload(), "report_path": _rel(report_path)}, sort_keys=True))
    else:
        print(_rel(report_path))
    return 0
