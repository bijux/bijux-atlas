#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
CONTRACT = json.loads((ROOT / "ops/_meta/layer-contract.json").read_text(encoding="utf-8"))

AREAS = ("stack", "k8s", "e2e", "obs", "load")
AREA_RE = re.compile(r"ops/(stack|k8s|e2e|obs|load)/")


def _shell_files(area: str) -> list[Path]:
    out: list[Path] = []
    for sub in ("scripts",):
        base = ROOT / "ops" / area / sub
        if base.exists():
            out.extend(sorted(base.rglob("*.sh")))
            out.extend(sorted(base.rglob("*.py")))
    return sorted(set(out))


def _allowed_refs(area: str) -> set[str]:
    data = CONTRACT.get("layer_dependencies", {}).get(area, {})
    return set(data.get("may_reference", []))


def _check_reference_rules(errors: list[str]) -> None:
    for area in AREAS:
        allowed = _allowed_refs(area)
        files = _shell_files(area)
        for path in files:
            rel = path.relative_to(ROOT).as_posix()
            text = path.read_text(encoding="utf-8", errors="ignore")
            for no, raw in enumerate(text.splitlines(), start=1):
                line = raw.strip()
                if not line or line.startswith("#"):
                    continue
                for m in AREA_RE.finditer(line):
                    target = m.group(1)
                    if target == area:
                        continue
                    if target not in allowed:
                        errors.append(f"{rel}:{no}: `{area}` cannot reference `{target}` directly")


def _check_e2e_no_manifest_patch(errors: list[str]) -> None:
    patt = re.compile(r"\b(kubectl|helm)\b.*\b(patch|apply|replace)\b", re.I)
    for path in _shell_files("e2e"):
        rel = path.relative_to(ROOT).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        for no, raw in enumerate(text.splitlines(), start=1):
            line = raw.strip()
            if not line or line.startswith("#"):
                continue
            if patt.search(line):
                errors.append(f"{rel}:{no}: e2e must not patch/apply manifests directly")


def _check_k8s_no_stack_assumptions(errors: list[str]) -> None:
    ns_values = set(CONTRACT.get("namespaces", {}).values())
    svc_values = {v.get("service_name") for v in CONTRACT.get("services", {}).values() if isinstance(v, dict)}
    port_values = set()
    for entry in CONTRACT.get("ports", {}).values():
        if isinstance(entry, dict):
            for v in entry.values():
                if isinstance(v, int):
                    port_values.add(v)
    literals = [x for x in list(ns_values | svc_values) if isinstance(x, str)] + [str(p) for p in sorted(port_values)]
    patterns = [re.compile(rf"\b{re.escape(v)}\b") for v in literals]

    for path in sorted((ROOT / "ops" / "k8s" / "scripts").rglob("*.sh")):
        rel = path.relative_to(ROOT).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        if "ops_layer_" in text or "ops_layer_contract_get" in text:
            continue
        for no, raw in enumerate(text.splitlines(), start=1):
            line = raw.strip()
            if not line or line.startswith("#"):
                continue
            if "ops_layer_" in line or "ops_layer_contract_get" in line:
                continue
            for pat in patterns:
                if pat.search(line):
                    errors.append(f"{rel}:{no}: k8s assumption literal must come from layer contract")
                    break


def _check_stack_no_product_logic(errors: list[str]) -> None:
    bad = (
        re.compile(r"/v[0-9]+/"),
        re.compile(r"\b(healthz|readyz|metrics)\b"),
        re.compile(r"\b(crates/|src/|openapi)\b"),
    )
    for path in sorted((ROOT / "ops" / "stack" / "scripts").rglob("*.sh")):
        rel = path.relative_to(ROOT).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        for no, raw in enumerate(text.splitlines(), start=1):
            line = raw.strip()
            if not line or line.startswith("#"):
                continue
            for pat in bad:
                if pat.search(line):
                    errors.append(f"{rel}:{no}: stack script contains product/endpoint knowledge")
                    break


def _check_profiles_by_id_only(errors: list[str]) -> None:
    bad = re.compile(r"ops/stack/kind/cluster[-a-z0-9]*\.yaml")
    for area in ("k8s", "e2e"):
        for path in _shell_files(area):
            rel = path.relative_to(ROOT).as_posix()
            text = path.read_text(encoding="utf-8", errors="ignore")
            for no, raw in enumerate(text.splitlines(), start=1):
                line = raw.strip()
                if not line or line.startswith("#"):
                    continue
                if bad.search(line):
                    errors.append(f"{rel}:{no}: profile must be referenced by id, not stack cluster yaml path")


def _check_ports_via_contract(errors: list[str]) -> None:
    port_vals = []
    for entry in CONTRACT.get("ports", {}).values():
        if isinstance(entry, dict):
            for v in entry.values():
                if isinstance(v, int):
                    port_vals.append(str(v))
    patt = re.compile(r"\b(" + "|".join(sorted(set(port_vals))) + r")\b")
    for area in ("k8s", "e2e"):
        for path in _shell_files(area):
            rel = path.relative_to(ROOT).as_posix()
            text = path.read_text(encoding="utf-8", errors="ignore")
            if "ops_layer_port_" in text or "ops_layer_contract_get" in text:
                continue
            for no, raw in enumerate(text.splitlines(), start=1):
                line = raw.strip()
                if not line or line.startswith("#"):
                    continue
                if "ops_layer_port_" in line or "ops_layer_contract_get" in line:
                    continue
                if patt.search(line):
                    errors.append(f"{rel}:{no}: hardcoded port literal must be read from contract")
                    break


def main() -> int:
    errors: list[str] = []
    _check_reference_rules(errors)
    _check_e2e_no_manifest_patch(errors)
    _check_k8s_no_stack_assumptions(errors)
    _check_stack_no_product_logic(errors)
    _check_profiles_by_id_only(errors)
    _check_ports_via_contract(errors)
    if errors:
        print("layer drift check failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("layer drift check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
