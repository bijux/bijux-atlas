#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / m).exists() for m in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def _build_json(root: Path, suite_json: str, out_json: str) -> None:
    cmd = [
        "python3",
        str(root / "packages/atlasctl/src/atlasctl/commands/ops/k8s/tests/build_conformance_report.py"),
        "--json",
        suite_json,
        "--out",
        out_json,
    ]
    p = subprocess.run(cmd, cwd=root, text=True, capture_output=True, check=False)
    if p.returncode != 0:
        raise SystemExit(p.returncode)


def _write_markdown(root: Path, json_path: Path, md_path: Path) -> None:
    payload = json.loads(json_path.read_text(encoding="utf-8"))
    lines = [
        "# K8s Conformance Report",
        f"- run_id: `{payload.get('run_id', '')}`",
        f"- suite_id: `{payload.get('suite_id', '')}`",
        f"- status: `{payload.get('status', '')}`",
        "",
        "## Sections",
    ]
    sections = payload.get("sections", {})
    for name in sorted(sections):
        row = sections[name]
        if not isinstance(row, dict):
            continue
        lines.append(f"- `{name}`: `{row.get('status', '')}`")
        missing = row.get("missing") or []
        failed = row.get("failed") or []
        if missing:
            lines.append(f"  - missing: {', '.join(str(x) for x in missing)}")
        if failed:
            lines.append(f"  - failed: {', '.join(str(x) for x in failed)}")
    md_path.parent.mkdir(parents=True, exist_ok=True)
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    ap = argparse.ArgumentParser(description="Generate k8s conformance report (json + markdown)")
    ap.add_argument("--suite-json", required=True, help="k8s suite results json input")
    ap.add_argument("--out-json", default="artifacts/reports/atlasctl/k8s-conformance-report.json")
    ap.add_argument("--out-md", default="artifacts/reports/atlasctl/k8s-conformance-report.md")
    args = ap.parse_args()

    root = _repo_root()
    out_json = root / args.out_json
    out_md = root / args.out_md
    out_json.parent.mkdir(parents=True, exist_ok=True)

    _build_json(root, str(root / args.suite_json), str(out_json))
    _write_markdown(root, out_json, out_md)
    print(out_json.relative_to(root).as_posix())
    print(out_md.relative_to(root).as_posix())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

