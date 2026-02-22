from __future__ import annotations
import subprocess, sys


def main() -> int:
    suite = sys.argv[1] if len(sys.argv) > 1 else 'verify'
    mapping = {
        'verify': 'packages/atlasctl/src/atlasctl/commands/ops/datasets/fetch_and_verify.py',
        'qc': 'packages/atlasctl/src/atlasctl/commands/ops/datasets/dataset_qc.py',
        'promotion': 'packages/atlasctl/src/atlasctl/commands/ops/datasets/promotion_sim.py',
        'corruption': 'packages/atlasctl/src/atlasctl/commands/ops/datasets/corruption_drill.py',
    }
    target = mapping.get(suite)
    if not target:
        print(f'unknown dataset suite: {suite} (expected: verify|qc|promotion|corruption)', file=sys.stderr)
        return 2
    return subprocess.call(['python3', target])


if __name__ == '__main__':
    raise SystemExit(main())
