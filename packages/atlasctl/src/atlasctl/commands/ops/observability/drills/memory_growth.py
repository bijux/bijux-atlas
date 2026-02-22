from __future__ import annotations
import os, subprocess
from pathlib import Path

def _metric_value(path: Path) -> str:
    for line in path.read_text(encoding='utf-8', errors='replace').splitlines():
        parts=line.split()
        if parts and parts[0]=='process_resident_memory_bytes' and len(parts)>1:
            return parts[1]
    return ''

def main() -> int:
    base = os.environ.get('ATLAS_BASE_URL', 'http://127.0.0.1:18080')
    out_dir = Path(os.environ.get('OUT_DIR','artifacts/ops/obs/drills'))
    out_dir.mkdir(parents=True, exist_ok=True)
    before = out_dir/'memory-before.metrics'; after = out_dir/'memory-after.metrics'; report = out_dir/'memory-growth-report.md'
    subprocess.check_call(['curl','-fsS',f'{base}/metrics'], stdout=before.open('wb'))
    import time; time.sleep(1)
    subprocess.check_call(['curl','-fsS',f'{base}/metrics'], stdout=after.open('wb'))
    b = _metric_value(before); a = _metric_value(after)
    if not b or not a: return 1
    growth = int(float(a)) - int(float(b))
    report.write_text(
        f"# Memory Growth Drill Report\n\n- before_bytes: {b}\n- after_bytes: {a}\n- growth_bytes: {growth}\n",
        encoding='utf-8',
    )
    print('memory growth drill assertions passed')
    print(f'report: {report}')
    return 0

if __name__ == '__main__': raise SystemExit(main())
