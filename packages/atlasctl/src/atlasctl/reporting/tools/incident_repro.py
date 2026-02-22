from __future__ import annotations

import os
import shutil
import subprocess
from datetime import datetime, timezone
from pathlib import Path


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[4]


def _copy_if_exists(src: Path, dst: Path) -> None:
    if src.exists():
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, dst)


def main() -> int:
    root = _repo_root()
    run_id = os.environ.get("OPS_RUN_ID", f"atlas-incident-{datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S')}")
    out = root / "artifacts" / "incident" / run_id
    out.mkdir(parents=True, exist_ok=True)

    with (out / "config.print.json").open("w", encoding="utf-8") as fh:
        subprocess.run([str(root / "bin/atlasctl"), "--quiet", "configs", "print"], check=True, stdout=fh, cwd=root)

    _copy_if_exists(root / "ops/_schemas/report/schema.json", out / "ops-report-schema.json")
    _copy_if_exists(root / "artifacts/ops" / run_id / "metadata.json", out / "metadata.json")
    _copy_if_exists(root / "artifacts/ops" / run_id / "report.json", out / "report.json")
    _copy_if_exists(root / "artifacts/ops/obs/metrics.prom", out / "metrics.prom")
    _copy_if_exists(root / "artifacts/ops/obs/traces.snapshot.log", out / "traces.snapshot.log")
    _copy_if_exists(root / "artifacts/ops/obs/traces.exemplars.log", out / "traces.exemplars.log")

    art_root = root / "artifacts/ops"
    lines: list[str] = []
    if art_root.exists():
        for p in art_root.rglob("*"):
            if not p.is_file():
                continue
            if p.suffix not in {".log", ".json", ".md"}:
                continue
            lines.append(str(p))
        lines = lines[-200:]
    (out / "artifact-file-list.txt").write_text("\n".join(lines) + ("\n" if lines else ""), encoding="utf-8")
    print(out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
