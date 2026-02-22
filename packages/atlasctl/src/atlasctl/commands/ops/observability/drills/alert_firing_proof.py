from __future__ import annotations
import subprocess
from pathlib import Path

def main() -> int:
    root = Path.cwd(); primary = root/'ops/obs/alerts/atlas-alert-rules.yaml'; burn = root/'ops/obs/alerts/slo-burn-rules.yaml'
    subprocess.check_call(['python3','packages/atlasctl/src/atlasctl/commands/ops/observability/drills/alerts_validation.py'])
    for a in ['BijuxAtlasHigh5xxRate','BijuxAtlasP95LatencyRegression','AtlasOverloadSustained','BijuxAtlasCheapSloBurnFast','BijuxAtlasCheapSloBurnMedium','BijuxAtlasCheapSloBurnSlow','BijuxAtlasStandardSloBurnFast','BijuxAtlasStandardSloBurnMedium','BijuxAtlasStandardSloBurnSlow','BijuxAtlasOverloadSurvivalViolated','BijuxAtlasRegistryRefreshStale','BijuxAtlasStoreBackendErrorSpike']:
        subprocess.check_call(['rg','-n',f'alert:\s*{a}',str(primary),str(burn)], stdout=subprocess.DEVNULL)
    print('slo burn alert synthetic proof passed')
    print('alert firing proof drill passed')
    return 0
if __name__ == '__main__': raise SystemExit(main())
