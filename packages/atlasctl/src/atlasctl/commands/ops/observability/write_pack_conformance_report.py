#!/usr/bin/env python3
# owner: bijux-atlas-operations
# purpose: emit pack conformance report artifact from snapshot/bundle presence checks.
# stability: internal
# called-by: make observability-pack-test, make observability-pack-drills
import json,subprocess
from pathlib import Path
def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


root = _repo_root()
out=root/'artifacts/observability'
out.mkdir(parents=True,exist_ok=True)
checks={
  'metrics_snapshot': (root/'artifacts/ops/obs/metrics.prom').exists(),
  'traces_snapshot': (root/'artifacts/ops/obs/traces.snapshot.log').exists(),
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
