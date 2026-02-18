#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: compare running observability pack component images to required pinned versions.
# stability: internal
# called-by: make ops-observability-pack-upgrade-check
set -euo pipefail
PROFILE="${ATLAS_OBS_PROFILE:-kind}"
if [ "${1:-}" = "--profile" ]; then
  PROFILE="${2:-}"
fi
PROFILE="$PROFILE" python3 - <<'PY'
import json,subprocess,sys,os
from pathlib import Path
cfg=json.load(open('configs/ops/obs-pack.json'))
required={
  'prometheus':cfg['images']['prometheus']['ref'],
  'grafana':cfg['images']['grafana']['ref'],
  'otel':cfg['images']['otel_collector']['ref'],
}
profile=os.environ.get('PROFILE','kind')
if profile=='local-compose':
  out=subprocess.check_output(['docker','ps','--format','{{.Image}} {{.Names}}'],text=True)
  have=out.splitlines()
  checks=[('prometheus',required['prometheus']),('grafana',required['grafana']),('otel-collector',required['otel'])]
  for name,img in checks:
    if not any(img in line and name in line for line in have):
      print(f'local-compose running image mismatch for {name}: expected {img}',file=sys.stderr)
      raise SystemExit(1)
else:
  ns=os.environ.get('ATLAS_OBS_NAMESPACE','atlas-observability')
  out=subprocess.check_output(['kubectl','-n',ns,'get','pods','-o','json'],text=True)
  pods=json.loads(out).get('items',[])
  found=set()
  for p in pods:
    for c in p.get('spec',{}).get('containers',[]):
      found.add(c.get('image',''))
  for img in required.values():
    if img not in found:
      print(f'k8s running image mismatch: expected {img}',file=sys.stderr)
      raise SystemExit(1)
print('observability pack upgrade check passed')
PY
