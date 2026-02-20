#!/usr/bin/env python3
# Purpose: enforce Docker policy (no latest tags, pinned build args, required OCI labels).
# Inputs: docker/images/runtime/Dockerfile and Make docker targets.
# Outputs: non-zero when policy contracts are violated.
from __future__ import annotations

from pathlib import Path
import re
import sys
import json

ROOT = Path(__file__).resolve().parents[3]
dockerfile_path = ROOT / "docker/images/runtime/Dockerfile"
dockerfile = dockerfile_path.read_text(encoding="utf-8")
product_mk = (ROOT / "makefiles/product.mk").read_text(encoding="utf-8")
allowlist = json.loads((ROOT / "docker/contracts/base-image-allowlist.json").read_text(encoding="utf-8"))
pinning = json.loads((ROOT / "docker/contracts/digest-pinning.json").read_text(encoding="utf-8"))

errors: list[str] = []

if re.search(r"^FROM\s+[^\n]*:latest\b", dockerfile, re.MULTILINE):
    errors.append("docker/images/runtime/Dockerfile uses forbidden latest tag")

m = re.search(r"^ARG\s+RUST_VERSION=([0-9]+\.[0-9]+\.[0-9]+)$", dockerfile, re.MULTILINE)
if not m:
    errors.append("docker/images/runtime/Dockerfile must pin ARG RUST_VERSION=<semver>")

from_lines = re.findall(r"^FROM\s+([^\s]+)", dockerfile, re.MULTILINE)
if len(from_lines) < 2:
    errors.append("docker/images/runtime/Dockerfile must define builder and runtime stages")
else:
    builder, runtime = from_lines[0], from_lines[-1]
    if not any(builder.startswith(prefix) for prefix in allowlist.get("allowed_builder_images", [])):
        errors.append(f"builder base image not in allowlist: {builder}")
    if runtime not in set(allowlist.get("allowed_runtime_images", [])):
        errors.append(f"runtime base image not in allowlist: {runtime}")

if pinning.get("forbid_latest", False):
    if ":latest" in dockerfile:
        errors.append("forbid_latest=true but :latest appears in dockerfile")

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

if "docker build" in product_mk:
    for token in ["--pull=false", "--build-arg RUST_VERSION", "--build-arg IMAGE_VERSION", "--build-arg VCS_REF", "--build-arg BUILD_DATE"]:
        if token not in product_mk:
            errors.append(f"docker-build target missing reproducibility/provenance arg: {token}")

if errors:
    print("docker policy check failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("docker policy check passed")
