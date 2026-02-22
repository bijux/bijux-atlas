from __future__ import annotations
import os, subprocess, sys, tempfile

def main() -> int:
    base = os.environ.get('ATLAS_BASE_URL', 'http://127.0.0.1:18080')
    m1 = tempfile.NamedTemporaryFile(delete=False)
    m2 = tempfile.NamedTemporaryFile(delete=False)
    m1.close(); m2.close()
    try:
        subprocess.check_call(['curl','-fsS',f'{base}/metrics'], stdout=open(m1.name,'wb'))
        code = subprocess.check_output(['curl','-s','-o','/dev/null','-w','%{http_code}', f'{base}/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-999999999&limit=500'], text=True).strip()
        subprocess.check_call(['curl','-fsS',f'{base}/metrics'], stdout=open(m2.name,'wb'))
        txt = Path(m2.name).read_text(encoding='utf-8', errors='replace')
        if 'bijux_overload_shedding_active' not in txt or 'bijux_cheap_queries_served_while_overloaded_total' not in txt:
            print('missing overload drill metrics', file=sys.stderr); return 1
        if code not in {'200','422','429','503'}:
            print(f'unexpected overload drill status code: {code}', file=sys.stderr); return 1
        print('overload admission control drill passed')
        return 0
    finally:
        for p in (m1.name, m2.name):
            try: Path(p).unlink()
            except FileNotFoundError: pass

if __name__ == '__main__': raise SystemExit(main())
