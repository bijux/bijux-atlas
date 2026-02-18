#!/usr/bin/env python3
# owner: bijux-atlas-operations
# purpose: emit pack conformance report artifact from snapshot/bundle presence checks.
# stability: internal
# called-by: make observability-pack-test, make observability-pack-drills
import json,subprocess
from pathlib import Path
root=Path(__file__).resolve().parents[3]
out=root/'artifacts/observability'
out.mkdir(parents=True,exist_ok=True)
checks={
  'metrics_snapshot': (root/'artifacts/ops/observability/metrics.prom').exists(),
  'traces_snapshot': (root/'artifacts/ops/observability/traces.snapshot.log').exists(),
  'pack_bundle': (root/'artifacts/observability/pack-bundle').exists(),
  'pack_version_stamp': (root/'artifacts/observability/pack-bundle/pack-version-stamp.json').exists(),
}
status='passed' if all(checks.values()) else 'failed'
sha=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
payload={
  'status':status,
  'git_sha':sha,
  'checks':checks,
}
(out/'pack-conformance-report.json').write_text(json.dumps(payload,indent=2,sort_keys=True)+'\n',encoding='utf-8')
print(str(out/'pack-conformance-report.json'))
