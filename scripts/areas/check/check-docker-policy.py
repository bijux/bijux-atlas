#!/usr/bin/env python3
# Purpose: enforce Docker policy (no latest tags, pinned build args, required OCI labels).
# Inputs: docker/Dockerfile and Make docker targets.
# Outputs: non-zero when policy contracts are violated.
from __future__ import annotations

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[3]
dockerfile = (ROOT / "docker/Dockerfile").read_text(encoding="utf-8")
root_mk = (ROOT / "makefiles/root.mk").read_text(encoding="utf-8")

errors: list[str] = []

if re.search(r"^FROM\s+[^\n]*:latest\b", dockerfile, re.MULTILINE):
    errors.append("docker/Dockerfile uses forbidden latest tag")

m = re.search(r"^ARG\s+RUST_VERSION=([0-9]+\.[0-9]+\.[0-9]+)$", dockerfile, re.MULTILINE)
if not m:
    errors.append("docker/Dockerfile must pin ARG RUST_VERSION=<semver>")

required_labels = [
    "org.opencontainers.image.version",
    "org.opencontainers.image.revision",
    "org.opencontainers.image.created",
    "org.opencontainers.image.source",
    "org.opencontainers.image.ref.name",
]
for label in required_labels:
    if label not in dockerfile:
        errors.append(f"missing required OCI label: {label}")

if "docker build" in root_mk:
    for token in ["--pull=false", "--build-arg RUST_VERSION", "--build-arg IMAGE_VERSION", "--build-arg VCS_REF", "--build-arg BUILD_DATE"]:
        if token not in root_mk:
            errors.append(f"docker-build target missing reproducibility/provenance arg: {token}")

if errors:
    print("docker policy check failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("docker policy check passed")
