from __future__ import annotations
import os, re, subprocess, sys, tempfile
from pathlib import Path

def main() -> int:
    base = os.environ.get('ATLAS_BASE_URL', 'http://127.0.0.1:18080')
    tmp = tempfile.NamedTemporaryFile(delete=False)
    tmp.close()
    try:
        subprocess.check_call(['curl','-fsS',f'{base}/healthz'], stdout=subprocess.DEVNULL)
        subprocess.check_call(['curl','-fsS',f'{base}/metrics'], stdout=open(tmp.name,'wb'))
        text = Path(tmp.name).read_text(encoding='utf-8', errors='replace')
        if not re.search(r'bijux_store_(circuit|breaker)_open', text): return 1
        if 'bijux_dataset_hits' not in text: return 1
        print('store outage drill assertions passed')
        return 0
    finally:
        try: Path(tmp.name).unlink()
        except FileNotFoundError: pass

if __name__ == '__main__': raise SystemExit(main())
