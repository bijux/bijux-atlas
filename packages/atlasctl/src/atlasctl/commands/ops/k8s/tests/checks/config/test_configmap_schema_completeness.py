#!/usr/bin/env python3
from __future__ import annotations

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
    config_doc = (root / "docs/operations/config.md").read_text(encoding="utf-8")
    values_doc = (root / "docs/operations/k8s/values.md").read_text(encoding="utf-8")

    tmpl_keys = sorted(set(re.findall(r"^\s+(ATLAS_[A-Z0-9_]+):", configmap_yaml, flags=re.MULTILINE)))
    doc_keys = sorted(set(re.findall(r"`(ATLAS_[A-Z0-9_]+)`", config_doc)))
    val_keys = sorted(set(re.findall(r"`(values\.[a-zA-Z0-9_.-]+)`", values_doc)))
    doc_set = set(doc_keys)
    val_set = set(val_keys)

    missing_docs = [k for k in tmpl_keys if k not in doc_set]
    if missing_docs:
        print("configmap schema completeness failed: missing config key docs in docs/operations/config.md", file=sys.stderr)
        for key in missing_docs:
            print(key, file=sys.stderr)
        return 1

    top_values = sorted(set(re.findall(r"\.Values\.([a-zA-Z0-9_]+)", configmap_yaml)))
    for top in top_values:
        if f"values.{top}" not in val_set:
            print(f"configmap schema completeness failed: missing values.{top} in docs/operations/k8s/values.md", file=sys.stderr)
            return 1
    print("configmap schema completeness passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
