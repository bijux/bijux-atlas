#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CONTRACT_PATH = ROOT / "ops" / "_meta" / "layer-contract.json"

SCAN_GLOBS = [
    "ops/e2e/scripts/**/*.sh",
    "ops/stack/scripts/**/*.sh",
    "ops/k8s/tests/checks/_lib/**/*.sh",
]

EXCLUDES = {
    str(ROOT / "ops" / "_meta" / "generate_layer_contract.py"),
}
ALLOWLIST = ROOT / "ops" / "_meta" / "layer-contract-literal-allowlist.txt"


def load_literals() -> list[str]:
    import json

    obj = json.loads(CONTRACT_PATH.read_text(encoding="utf-8"))
    vals = set()
    vals.update(obj.get("namespaces", {}).values())
    for svc in obj.get("services", {}).values():
        name = svc.get("service_name")
        if isinstance(name, str):
            vals.add(name)
    return sorted(v for v in vals if isinstance(v, str) and v)


def main() -> int:
    literals = load_literals()
    if not literals:
        print("no literals loaded from layer contract", file=sys.stderr)
        return 1

    allowed_paths = set()
    if ALLOWLIST.exists():
        for raw in ALLOWLIST.read_text(encoding="utf-8").splitlines():
            line = raw.strip()
            if line and not line.startswith("#"):
                allowed_paths.add((ROOT / line).resolve())

    patterns = [re.compile(rf"\b{re.escape(x)}\b") for x in literals]
    errors: list[str] = []

    files: set[Path] = set()
    for glob in SCAN_GLOBS:
        files.update(ROOT.glob(glob))

    for path in sorted(files):
        if str(path) in EXCLUDES or not path.is_file():
            continue
        if path.resolve() in allowed_paths:
            continue
        text = path.read_text(encoding="utf-8")
        if "ops_layer_contract_get" in text or "ops_layer_" in text:
            continue
        for i, line in enumerate(text.splitlines(), start=1):
            stripped = line.strip()
            if not stripped or stripped.startswith("#"):
                continue
            if "ops_layer_contract_get" in line or "ops_layer_" in line:
                continue
            for pat in patterns:
                if pat.search(line):
                    errors.append(f"{path.relative_to(ROOT)}:{i}: forbidden literal; source from ops/_meta/layer-contract.json")
                    break

    if errors:
        print("layer literal drift check failed", file=sys.stderr)
        for e in errors:
            print(f"- {e}", file=sys.stderr)
        return 1

    print("layer literal drift check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
