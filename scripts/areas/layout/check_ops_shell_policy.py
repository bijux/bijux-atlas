#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
OPS_RUN = ROOT / "ops" / "run"

RAW_KUBECTL = re.compile(r"(^|[^\w])kubectl(\s|$)")
RAW_HELM = re.compile(r"(^|[^\w])helm(\s|$)")

ALLOW_RAW = (
    "command -v kubectl",
    "command -v helm",
    "kubectl version",
    "helm version",
    "ops_version_guard",
    "check_tool_versions.py",
    "ops-kubectl-version-check",
    "ops-helm-version-check",
)

FORBIDDEN_LITERALS = ("atlas-e2e",)


def is_comment(line: str) -> bool:
    return line.strip().startswith("#")


def main() -> int:
    errors: list[str] = []
    for path in sorted(OPS_RUN.glob("*.sh")):
        rel = path.relative_to(ROOT).as_posix()
        text = path.read_text(encoding="utf-8")
        if "ops/_lib/common.sh" not in text:
            errors.append(f"{rel}: must source ops/_lib/common.sh")
        for no, raw in enumerate(text.splitlines(), start=1):
            line = raw.strip()
            if not line or is_comment(line):
                continue
            if line.startswith("for ") and " in " in line:
                continue
            if any(token in line for token in ALLOW_RAW):
                continue
            if RAW_KUBECTL.search(line):
                errors.append(f"{rel}:{no}: direct kubectl call forbidden; use ops_kubectl/ops_kubectl_retry")
            if RAW_HELM.search(line):
                errors.append(f"{rel}:{no}: direct helm call forbidden; use ops_helm/ops_helm_retry")
            if any(lit in line for lit in FORBIDDEN_LITERALS):
                if "ops_layer_" not in line and "ops_layer_contract_get" not in line:
                    errors.append(f"{rel}:{no}: forbidden namespace literal; source from layer contract helpers")

    if errors:
        print("ops run shell policy check failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("ops run shell policy check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
