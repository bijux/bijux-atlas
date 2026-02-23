from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
COMMAND = ROOT / "packages/atlasctl/src/atlasctl/commands/product/command.py"
SCHEMA = ROOT / "configs/product/artifact-manifest.schema.json"


def main() -> int:
    errs: list[str] = []
    text = COMMAND.read_text(encoding="utf-8", errors="ignore")
    schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
    if schema.get("title") != "Product Artifact Manifest":
        errs.append("artifact manifest schema title mismatch")
    required = set(schema.get("required", []))
    for key in ("schema_version", "kind", "run_id", "version", "artifacts"):
        if key not in required:
            errs.append(f"artifact-manifest schema missing required field: {key}")
    if "rows.sort(" not in text:
        errs.append("product artifact rows must be sorted deterministically")
    if 'sort_keys=True' not in text:
        errs.append("product artifact manifest/report JSON must be emitted with sort_keys=True")
    if "_write_artifact_manifest(ctx)" not in text:
        errs.append("product build must emit artifact-manifest.json")
    if "validate_json(payload, ctx.repo_root / PRODUCT_SCHEMA)" not in text:
        errs.append("product artifact manifest payload must be schema-validated before write")
    if errs:
        print("product artifact manifest contract check failed:", file=sys.stderr)
        for e in errs:
            print(e, file=sys.stderr)
        return 1
    print("product artifact manifest contract check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
