#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
CORE_DOMAINS = ("stack", "k8s", "obs", "load", "e2e", "datasets")
REQUIRED_DOCS = ("CONTRACT.md", "INDEX.md", "OWNER.md", "README.md")


def _has_any_manifest_or_suite(domain_root: Path) -> bool:
    for p in domain_root.rglob("*.json"):
        rel = p.relative_to(domain_root).as_posix()
        if "manifest" in p.name or "/suites/" in rel or p.name == "suites.json":
            return True
    return False


def _has_any_schema(domain: str) -> bool:
    if domain == "e2e":
        root = ROOT / "ops" / "_schemas"
        return any(root.glob("e2e-*.schema.json"))
    schema_root = ROOT / "ops" / "_schemas" / domain
    return schema_root.exists() and any(schema_root.rglob("*.schema.json"))


def main() -> int:
    errors: list[str] = []
    for domain in CORE_DOMAINS:
        droot = ROOT / "ops" / domain
        if not droot.exists():
            errors.append(f"missing ops domain: ops/{domain}")
            continue
        for name in REQUIRED_DOCS:
            if not (droot / name).exists():
                errors.append(f"ops/{domain}: missing {name}")
        if not _has_any_manifest_or_suite(droot):
            errors.append(f"ops/{domain}: must contain at least one manifest/suites json")
        if domain != "report" and not _has_any_schema(domain):
            errors.append(f"ops/{domain}: missing schema(s) under ops/_schemas/{domain}")
    if errors:
        print("ops domain docs contract failed:", file=sys.stderr)
        for e in errors:
            print(e, file=sys.stderr)
        return 1
    print("ops domain docs contract passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
