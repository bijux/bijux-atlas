#!/usr/bin/env python3
# owner: platform
# purpose: ensure ops policy configs are reflected in make/ops runtime usage.
# stability: public
# called-by: make ops-policy-audit, make ci-ops-policy-audit
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def main() -> int:
    env_schema = json.loads(read(ROOT / "configs/ops/env.schema.json"))
    vars_declared = sorted(env_schema.get("variables", {}).keys())
    search_paths = [
        ROOT / "makefiles/env.mk",
        ROOT / "makefiles/ops.mk",
        ROOT / "scripts/areas/layout/validate_ops_env.py",
        ROOT / "scripts/areas/public/config-print.py",
        ROOT / "crates/bijux-atlas-server/src/main.rs",
    ]
    text = "\n".join(read(p) for p in search_paths)
    violations: list[str] = []
    for var in vars_declared:
        if re.search(rf"\b{re.escape(var)}\b", text) is None:
            violations.append(f"ops env variable `{var}` not reflected in make/scripts usage")

    # Ops tool versions config must be consumed by ops tool checks.
    tools_cfg_name = "configs/ops/tool-versions.json"
    ops_mk = read(ROOT / "makefiles/ops.mk")
    if tools_cfg_name not in ops_mk:
        violations.append("ops.mk must reference configs/ops/tool-versions.json")

    if violations:
        for v in violations:
            print(f"ops-policy-audit violation: {v}", file=sys.stderr)
        return 1
    print("ops policy audit passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
