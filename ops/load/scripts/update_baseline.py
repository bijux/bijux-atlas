#!/usr/bin/env python3
from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BASELINES = ROOT / "configs/ops/perf/baselines"
CHANGELOG = BASELINES / "CHANGELOG.md"
TOOLS = ROOT / "configs/ops/tool-versions.json"
LOCK = ROOT / "ops/datasets/manifest.lock"


def _rows(results_dir: Path) -> list[dict[str, float | str]]:
    rows: list[dict[str, float | str]] = []
    for summary in sorted(results_dir.glob("*.summary.json")):
        data = json.loads(summary.read_text(encoding="utf-8"))
        vals = data.get("metrics", {}).get("http_req_duration", {}).get("values", {})
        fail = data.get("metrics", {}).get("http_req_failed", {}).get("values", {})
        rows.append(
            {
                "suite": summary.stem.replace(".summary", ""),
                "p95_ms": float(vals.get("p(95)", 0.0)),
                "p99_ms": float(vals.get("p(99)", 0.0)),
                "fail_rate": float(fail.get("rate", 0.0)),
            }
        )
    return rows


def _diff(old_rows: list[dict], new_rows: list[dict]) -> list[str]:
    old = {str(r["suite"]): r for r in old_rows}
    lines: list[str] = []
    for row in new_rows:
        suite = str(row["suite"])
        prev = old.get(suite)
        if not prev:
            lines.append(f"- {suite}: added (p95={row['p95_ms']:.2f} p99={row['p99_ms']:.2f})")
            continue
        d95 = float(row["p95_ms"]) - float(prev.get("p95_ms", 0.0))
        d99 = float(row["p99_ms"]) - float(prev.get("p99_ms", 0.0))
        lines.append(f"- {suite}: p95 {prev.get('p95_ms',0):.2f}->{row['p95_ms']:.2f} ({d95:+.2f}), p99 {prev.get('p99_ms',0):.2f}->{row['p99_ms']:.2f} ({d99:+.2f})")
    return lines


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--profile", default="local")
    ap.add_argument("--results", default="artifacts/perf/results")
    ap.add_argument("--environment", default="local")
    ap.add_argument("--k8s-profile", default="kind")
    ap.add_argument("--replicas", type=int, default=1)
    args = ap.parse_args()

    results = (ROOT / args.results).resolve()
    if not results.exists():
        raise SystemExit(f"missing results dir: {results}")

    rows = _rows(results)
    if not rows:
        raise SystemExit(f"no *.summary.json files in {results}")

    dst = BASELINES / f"{args.profile}.json"
    prev = json.loads(dst.read_text(encoding="utf-8")) if dst.exists() else {"rows": []}
    tools = json.loads(TOOLS.read_text(encoding="utf-8"))
    lock_hash = hashlib.sha256(LOCK.read_bytes()).hexdigest()[:16]
    now = dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")
    payload = {
        "schema_version": 1,
        "name": args.profile,
        "source": "make perf/baseline-update",
        "notes": "managed baseline (do not edit by hand)",
        "metadata": {
            "captured_at": now,
            "environment": args.environment,
            "profile": args.profile,
            "dataset_set": [f"lock:{lock_hash}"],
            "dataset_lock_hash": lock_hash,
            "k8s_profile": args.k8s_profile,
            "replicas": args.replicas,
            "tool_versions": {
                "k6": tools.get("k6", "unknown"),
                "kind": tools.get("kind", "unknown"),
                "kubectl": tools.get("kubectl", "unknown"),
                "helm": tools.get("helm", "unknown"),
            },
        },
        "rows": rows,
    }
    dst.parent.mkdir(parents=True, exist_ok=True)
    dst.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    diff_lines = _diff(prev.get("rows", []), rows)
    run_id = (ROOT / "artifacts/evidence/latest-run-id.txt").read_text(encoding="utf-8").strip() if (ROOT / "artifacts/evidence/latest-run-id.txt").exists() else "manual"
    out_dir = ROOT / "artifacts/evidence/perf" / run_id
    out_dir.mkdir(parents=True, exist_ok=True)
    summary = out_dir / f"baseline-update-{args.profile}.md"
    summary.write_text(
        "\n".join(
            [
                f"# Baseline Update Summary ({args.profile})",
                "",
                f"- baseline: `{dst.relative_to(ROOT)}`",
                f"- results: `{results.relative_to(ROOT)}`",
                "",
                "## Diff vs previous",
                *diff_lines,
                "",
            ]
        )
        + "\n",
        encoding="utf-8",
    )

    changelog_entry = (
        f"\n## {dt.date.today().isoformat()} ({args.profile})\n"
        f"- source: `make perf/baseline-update PROFILE={args.profile}`\n"
        f"- summary: `{summary.relative_to(ROOT)}`\n"
    )
    if CHANGELOG.exists():
        CHANGELOG.write_text(CHANGELOG.read_text(encoding="utf-8") + changelog_entry, encoding="utf-8")
    else:
        CHANGELOG.write_text("# Perf Baseline Changelog\n" + changelog_entry, encoding="utf-8")

    print(dst.relative_to(ROOT))
    print(summary.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
