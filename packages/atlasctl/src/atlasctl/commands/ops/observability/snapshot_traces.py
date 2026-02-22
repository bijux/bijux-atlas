from __future__ import annotations
import subprocess, sys
from pathlib import Path


def main() -> int:
    root = Path.cwd()
    out_dir = Path(sys.argv[1] if len(sys.argv) > 1 else str(root / 'artifacts/ops/obs'))
    ns = __import__('os').environ.get('ATLAS_E2E_NAMESPACE', 'atlas-e2e')
    out_dir.mkdir(parents=True, exist_ok=True)
    pod = subprocess.check_output(['bash','-lc', f"kubectl -n '{ns}' get pod -l app=otel-collector -o jsonpath='{{.items[0].metadata.name}}' 2>/dev/null || true"], text=True).strip()
    if not pod:
        ns_discovered = subprocess.check_output(['bash','-lc', "kubectl get pod -A -l app=otel-collector -o jsonpath='{.items[0].metadata.namespace}' 2>/dev/null || true"], text=True).strip()
        if ns_discovered:
            ns = ns_discovered
            pod = subprocess.check_output(['bash','-lc', f"kubectl -n '{ns}' get pod -l app=otel-collector -o jsonpath='{{.items[0].metadata.name}}' 2>/dev/null || true"], text=True).strip()
    snap = out_dir / 'traces.snapshot.log'
    ex = out_dir / 'traces.exemplars.log'
    if not pod:
        snap.touch(); ex.touch()
        print(f'otel collector not present; wrote empty {snap}')
        return 0
    with snap.open('wb') as fh:
        rc = subprocess.call(['kubectl','-n',ns,'logs',pod,'--tail=1000'], stdout=fh)
    if rc != 0 and not snap.exists():
        snap.touch()
    text = snap.read_text(encoding='utf-8', errors='replace') if snap.exists() else ''
    lines = [ln for ln in text.splitlines() if any(k in ln.lower() for k in ('trace_id','traceid','span_id','spanid','export','span','otlp','traces'))]
    ex.write_text(('\n'.join(lines) + ('\n' if lines else '')), encoding='utf-8')
    if ex.stat().st_size == 0 and snap.stat().st_size > 0:
        ex.write_text('\n'.join(text.splitlines()[:20]) + '\n', encoding='utf-8')
    print(f'wrote {snap}')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
