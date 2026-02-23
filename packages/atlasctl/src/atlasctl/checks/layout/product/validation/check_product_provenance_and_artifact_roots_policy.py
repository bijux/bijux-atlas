from __future__ import annotations

import re
import sys
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("ops", "packages", "configs", "makefiles")):
            return parent
    raise RuntimeError("unable to resolve repo root")


ROOT = _repo_root()
COMMAND = ROOT / "packages/atlasctl/src/atlasctl/commands/product/command.py"
DOMAIN_DOC = ROOT / "packages/atlasctl/docs/control-plane/product-domain.md"
RETENTION_DOC = ROOT / "packages/atlasctl/docs/control-plane/product-artifact-retention.md"


def main() -> int:
    errs: list[str] = []
    text = COMMAND.read_text(encoding="utf-8", errors="ignore")
    for token in ('"git_sha":', '"pins_digest":', '"schema_versions":'):
        if token not in text:
            errs.append(f"product manifest provenance field missing from command payload: {token}")

    if "_sha256(" not in text or "rows.sort(" not in text:
        errs.append("product build reproducibility contract missing checksum/sorted rows logic")

    # 184: any packages/artifacts/tmp usage must be deterministic (per-run or explicit cleanup). Currently forbid in product command.
    if "packages/artifacts/tmp" in text:
        if "run_id" not in text and "cleanup" not in text:
            errs.append("packages/artifacts/tmp usage in product command must be per-run or explicitly cleaned")

    domain_doc = DOMAIN_DOC.read_text(encoding="utf-8", errors="ignore")
    for token in ("artifact-manifest.json", "git_sha", "pins_digest", "schema_versions"):
        if token not in domain_doc:
            errs.append(f"product-domain doc missing provenance/ssot token: {token}")

    if not RETENTION_DOC.exists():
        errs.append("missing product artifact retention policy doc")
    else:
        rtxt = RETENTION_DOC.read_text(encoding="utf-8", errors="ignore")
        for token in ("committed", "ephemeral", "artifacts/evidence/product", "packages/artifacts/tmp"):
            if token not in rtxt:
                errs.append(f"retention policy doc missing token: {token}")

    if errs:
        print("product provenance/tmp policy check failed:", file=sys.stderr)
        for e in errs:
            print(e, file=sys.stderr)
        return 1
    print("product provenance/tmp policy check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
