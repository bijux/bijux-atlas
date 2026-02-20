#!/usr/bin/env python3
# Purpose: ensure every ATLAS_* configmap key is documented in docs/operations/config.md.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CM_TEMPLATE = ROOT / "ops" / "k8s" / "charts" / "bijux-atlas" / "templates" / "configmap.yaml"
DOC = ROOT / "docs" / "operations" / "config.md"
VALUES_DOC = ROOT / "docs" / "operations" / "k8s" / "values.md"


def main() -> int:
    tmpl_text = CM_TEMPLATE.read_text(encoding="utf-8")
    doc_text = DOC.read_text(encoding="utf-8")
    values_doc_text = VALUES_DOC.read_text(encoding="utf-8")
    cfg_keys = sorted(set(re.findall(r"^\s+(ATLAS_[A-Z0-9_]+):", tmpl_text, flags=re.MULTILINE)))
    top_level_values = sorted(set(re.findall(r"\.Values\.([a-zA-Z0-9_]+)", tmpl_text)))
    errors: list[str] = []
    for key in cfg_keys:
        if f"`{key}`" not in doc_text:
            errors.append(f"missing key in docs/operations/config.md: {key}")
    for top in top_level_values:
        if f"`values.{top}`" not in values_doc_text:
            errors.append(f"missing values reference in docs/operations/k8s/values.md: values.{top}")
    if errors:
        print("configmap env docs check failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("configmap env docs check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
