from __future__ import annotations

import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[7]


def main() -> int:
    root = _repo_root()
    out = root / "artifacts/ops/k8s-install-matrix.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(
        (
            "{\n"
            f'  "generated_at": "{datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")}",\n'
            '  "profiles": ["local", "offline", "perf", "ingress", "multi-registry"],\n'
            '  "tests": ["install", "networkpolicy", "hpa", "pdb", "rollout", "rollback", "secrets", "configmap", "serviceMonitor"]\n'
            "}\n"
        ),
        encoding="utf-8",
    )
    subprocess.run([sys.executable, str(root / "scripts/areas/docs/generate_k8s_install_matrix.py"), str(out)], check=True, cwd=root)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
