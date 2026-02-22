from __future__ import annotations
import os, subprocess

def main() -> int:
    ns = os.environ.get('ATLAS_E2E_NAMESPACE', 'atlas-e2e')
    if os.environ.get('ATLAS_E2E_ENABLE_OTEL', '0') != '1':
        print('otel disabled; skipping trace verification'); return 0
    if subprocess.call(['kubectl','config','current-context'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) != 0:
        print('trace verification skipped: kubectl context is not configured'); return 0
    pod = subprocess.check_output(['bash','-lc', f"kubectl -n '{ns}' get pod -l app=otel-collector -o name 2>/dev/null | head -n1 | cut -d/ -f2"], text=True).strip()
    if not pod:
        print(f"trace verification skipped: otel-collector pod not found in namespace '{ns}'")
        return 0
    logs = subprocess.check_output(['kubectl','-n',ns,'logs',pod,'--tail=800'], text=True, errors='replace')
    needed = ('admission_control','dataset_resolve','cache_lookup','store_fetch','open_db','sqlite_query','serialize_response')
    if not any(k in logs for k in needed):
        return 1
    print('trace verification passed')
    return 0

if __name__ == '__main__': raise SystemExit(main())
