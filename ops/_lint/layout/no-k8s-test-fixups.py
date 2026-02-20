#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
K8S_TESTS = ROOT / "ops" / "k8s" / "tests"

# Mutating manifest-level fixups must stay in harness-managed operations,
# not in arbitrary test scripts.
FORBIDDEN = (
    re.compile(r"\bkubectl\b.*\bapply\b"),
    re.compile(r"\bkubectl\b.*\bpatch\b"),
    re.compile(r"\bkubectl\b.*\bcreate\s+-f\b"),
)

# Explicitly approved harness-level mutation checks.
ALLOWED_PATHS = {
    "ops/k8s/tests/checks/config/test_configmap_update_reload.sh",
    "ops/k8s/tests/checks/config/test_configmap_unknown_keys_rejected.sh",
    "ops/k8s/tests/checks/security/test_secrets_rotation.sh",
    "ops/k8s/tests/checks/datasets/test_pdb.sh",
}


def main() -> int:
    violations: list[str] = []
    for path in sorted(K8S_TESTS.rglob("*.sh")):
        rel = path.relative_to(ROOT).as_posix()
        if rel in ALLOWED_PATHS:
            continue
        for no, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            line = raw.strip()
            if not line or line.startswith("#"):
                continue
            for pat in FORBIDDEN:
                if pat.search(line):
                    violations.append(
                        f"{rel}:{no}: forbidden k8s test fixup command; use harness operations instead"
                    )
                    break

    if violations:
        print("k8s test boundary lint failed:", file=sys.stderr)
        for v in violations:
            print(f"- {v}", file=sys.stderr)
        return 1
    print("k8s test boundary lint passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
