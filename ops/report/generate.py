#!/usr/bin/env python3
# Purpose: generate consolidated ops report JSON/markdown from run artifacts.
# Inputs: --run-dir and --schema file path.
# Outputs: <run-dir>/report.json and <run-dir>/report.md.
from __future__ import annotations

import argparse
import json
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-dir", required=True)
    parser.add_argument("--schema", required=True)
    args = parser.parse_args()

    run_dir = Path(args.run_dir)
    metadata_file = run_dir / "metadata.json"
    metadata = json.loads(metadata_file.read_text(encoding="utf-8")) if metadata_file.exists() else {}

    artifacts = {
        "logs": (run_dir / "logs" / "events.txt").exists() or (run_dir / "logs" / "pods.txt").exists(),
        "metrics": (run_dir / "metrics" / "metrics.txt").exists(),
        "smoke_report": (run_dir / "smoke" / "report.md").exists(),
        "perf_results": (run_dir / "perf" / "results").exists() or (run_dir / "perf").exists(),
    }

    report = {
        "run_id": metadata.get("run_id", run_dir.name),
        "namespace": metadata.get("namespace", "unknown"),
        "metadata": metadata,
        "artifacts": artifacts,
    }

    (run_dir / "report.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    lines = [
        "# Ops Run Report",
        "",
        f"- Run ID: `{report['run_id']}`",
        f"- Namespace: `{report['namespace']}`",
        "",
        "## Artifacts",
        "",
        f"- logs: `{artifacts['logs']}`",
        f"- metrics: `{artifacts['metrics']}`",
        f"- smoke_report: `{artifacts['smoke_report']}`",
        f"- perf_results: `{artifacts['perf_results']}`",
    ]
    md = "\n".join(lines) + "\n"
    (run_dir / "report.md").write_text(md, encoding="utf-8")
    (run_dir / "index.md").write_text(md, encoding="utf-8")
    print(run_dir / "report.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
