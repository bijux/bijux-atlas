#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import sys
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def main() -> int:
    root = _repo_root()
    configmap_yaml = (root / "ops/k8s/charts/bijux-atlas/templates/configmap.yaml").read_text(encoding="utf-8")
    tmpl_keys = sorted(set(re.findall(r"^\s+(ATLAS_[A-Z0-9_]+):", configmap_yaml, flags=re.MULTILINE)))
    doc = json.loads((root / "docs/contracts/CONFIG_KEYS.json").read_text(encoding="utf-8"))
    runtime_keys = sorted(str(k) for k in doc.get("keys", []) if isinstance(k, str))
    runtime_set = set(runtime_keys)
    unknown = [k for k in tmpl_keys if k not in runtime_set]
    if unknown:
        print("configmap keys/runtime contract failed: keys not declared in docs/contracts/CONFIG_KEYS.json", file=sys.stderr)
        for key in unknown:
            print(key, file=sys.stderr)
        return 1
    print("configmap keys match runtime expected contract passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
