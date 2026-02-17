#!/usr/bin/env python3
# Purpose: extract blessed runnable shell snippets from docs markdown.
# Inputs: docs/**/*.md
# Outputs: artifacts/docs-snippets/*.sh and artifacts/docs-snippets/manifest.json
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DOCS = ROOT / "docs"
OUT = ROOT / "artifacts" / "docs-snippets"

FENCE_RE = re.compile(r"```(?:bash|sh)\n(.*?)```", re.DOTALL)


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    for old in OUT.glob("*.sh"):
        old.unlink()

    manifest = []
    idx = 0
    for path in sorted(DOCS.rglob("*.md")):
        text = path.read_text(encoding="utf-8")
        for block in FENCE_RE.findall(text):
            lines = [ln.rstrip("\n") for ln in block.splitlines()]
            cleaned = [ln for ln in lines if ln.strip()]
            if not cleaned:
                continue
            if cleaned[0].strip() != "# blessed-snippet":
                continue
            allow_network = any(ln.strip() == "# allow-network" for ln in cleaned[1:3])
            body = [ln for ln in cleaned[1:] if ln.strip() not in {"# allow-network"}]
            idx += 1
            out = OUT / f"snippet-{idx:03d}.sh"
            out.write_text("#!/usr/bin/env sh\nset -eu\n" + "\n".join(body) + "\n", encoding="utf-8")
            out.chmod(0o755)
            manifest.append(
                {
                    "id": idx,
                    "source": str(path.relative_to(ROOT)),
                    "path": str(out.relative_to(ROOT)),
                    "allow_network": allow_network,
                }
            )

    (OUT / "manifest.json").write_text(json.dumps({"snippets": manifest}, indent=2) + "\n", encoding="utf-8")
    print(f"extracted {len(manifest)} blessed snippet(s) to {OUT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
