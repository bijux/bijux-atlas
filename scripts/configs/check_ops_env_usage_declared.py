#!/usr/bin/env python3
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ENV = json.loads((ROOT / "configs/ops/env.schema.json").read_text(encoding="utf-8"))
DECLARED = set(ENV.get("variables", {}).keys())
PAT = re.compile(r"\$\{?([A-Z][A-Z0-9_]+)\}?")
ALLOW = {"PWD","HOME","PATH","PPID","SHELL","USER","UID","GITHUB_SHA","GITHUB_REF_NAME","COSIGN_IMAGE_REF","COSIGN_CERT_IDENTITY"}


def main() -> int:
  errs=[]
  scopes = [ROOT / "ops/run", ROOT / "ops/_lib/env.sh", ROOT / "ops/_lib/common.sh"]
  files = []
  for scope in scopes:
    if isinstance(scope, Path) and scope.is_file():
      files.append(scope)
    elif isinstance(scope, Path) and scope.is_dir():
      files.extend(sorted(scope.rglob("*.sh")))
  for p in files:
    txt=p.read_text(encoding="utf-8",errors="ignore")
    for v in sorted(set(PAT.findall(txt))):
      if v in ALLOW:
        continue
      if v.startswith("ATLAS_") or v.startswith("OPS_") or v in {"RUN_ID","ARTIFACT_DIR","ISO_ROOT","CARGO_TARGET_DIR","CARGO_HOME","TMPDIR","TMP","TEMP","PROFILE","SUITE","DRILL","MODE","LATENCY_MS","JITTER_MS"}:
        if v not in DECLARED:
          errs.append(f"{p.relative_to(ROOT)} uses undeclared env var: {v}")
  if errs:
    print("ops env declaration check failed")
    for e in errs[:120]:
      print(f"- {e}")
    return 1
  print("ops env declaration check passed")
  return 0

if __name__=="__main__":
  raise SystemExit(main())
