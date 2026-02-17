#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
from __future__ import annotations

import re
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[2]
DOCS = ROOT / "docs"
REGISTRY = DOCS / "_style" / "concepts.yml"

ID_PAT = re.compile(r"^Concept ID:\s*`?([a-z0-9.-]+)`?\s*$", re.MULTILINE)
IDS_PAT = re.compile(r"^Concept IDs:\s*`?([a-z0-9.,\s-]+)`?\s*$", re.MULTILINE)


def extract_ids(text: str) -> list[str]:
    ids: list[str] = []
    for m in ID_PAT.finditer(text):
        ids.append(m.group(1).strip())
    for m in IDS_PAT.finditer(text):
        parts = [p.strip() for p in m.group(1).split(",") if p.strip()]
        ids.extend(parts)
    # stable unique
    out: list[str] = []
    for i in ids:
        if i not in out:
            out.append(i)
    return out


def rel(p: Path) -> str:
    return str(p).replace(str(ROOT) + "/", "")


def main() -> int:
    data = yaml.safe_load(REGISTRY.read_text(encoding="utf-8"))
    concepts = data.get("concepts", []) if isinstance(data, dict) else []
    errors: list[str] = []

    if not concepts:
        errors.append(f"{rel(REGISTRY)}: missing concepts list")

    registry_ids: set[str] = set()
    canonical_by_id: dict[str, str] = {}
    pointers_by_id: dict[str, list[str]] = {}

    for c in concepts:
        cid = c.get("id")
        canonical = c.get("canonical")
        pointers = c.get("pointers", [])
        if not cid or not canonical:
            errors.append("concept entry missing id or canonical")
            continue
        if cid in registry_ids:
            errors.append(f"duplicate concept id in registry: {cid}")
        registry_ids.add(cid)
        canonical_by_id[cid] = canonical
        pointers_by_id[cid] = pointers

    canonical_claims: dict[str, list[str]] = {k: [] for k in registry_ids}

    for cid, canonical in canonical_by_id.items():
        p = ROOT / canonical
        if not p.exists():
            errors.append(f"{cid}: missing canonical file {canonical}")
            continue
        text = p.read_text(encoding="utf-8")
        ids = extract_ids(text)
        is_generated_contract_doc = canonical.startswith("docs/contracts/")
        if cid not in ids and not is_generated_contract_doc:
            errors.append(f"{canonical}: missing declaration for {cid}")
        if "Canonical page:" in text:
            errors.append(f"{canonical}: canonical page must not be a pointer")
        canonical_claims[cid].append(canonical)

        for pointer in pointers_by_id.get(cid, []):
            pp = ROOT / pointer
            if not pp.exists():
                errors.append(f"{cid}: missing pointer file {pointer}")
                continue
            ptxt = pp.read_text(encoding="utf-8")
            pids = extract_ids(ptxt)
            if cid not in pids:
                errors.append(f"{pointer}: missing declaration for {cid}")
            if "Canonical page:" not in ptxt:
                errors.append(f"{pointer}: pointer missing `Canonical page:` line")
            if canonical not in ptxt:
                errors.append(f"{pointer}: pointer must link to {canonical}")

    # no unknown concepts in docs + discover canonical claim collisions
    for md in DOCS.rglob("*.md"):
        text = md.read_text(encoding="utf-8")
        ids = extract_ids(text)
        for cid in ids:
            if cid not in registry_ids:
                errors.append(f"{rel(md)}: concept `{cid}` not declared in {rel(REGISTRY)}")
            else:
                # A page that declares a concept and has no Canonical page pointer claims canonical ownership.
                if "Canonical page:" not in text:
                    canonical_claims[cid].append(rel(md))

    # fail if two canonical docs claim same concept id
    for cid, files in canonical_claims.items():
        unique = sorted(set(files))
        if len(unique) != 1:
            errors.append(f"{cid}: expected one canonical page, got {unique}")
            continue
        expected = canonical_by_id[cid]
        if unique[0] != expected:
            errors.append(
                f"{cid}: canonical mismatch, expected {expected}, found {unique[0]}"
            )

    if errors:
        print("concept registry check failed:", file=sys.stderr)
        for e in errors:
            print(f"- {e}", file=sys.stderr)
        return 1

    print("concept registry check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
