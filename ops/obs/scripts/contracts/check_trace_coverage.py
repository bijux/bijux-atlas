#!/usr/bin/env python3
# Purpose: enforce captured trace coverage for endpoint classes and lifecycle spans.
# Inputs: artifacts/ops/obs/traces.exemplars.log and traces.snapshot.log
# Outputs: non-zero exit when required captured spans are absent while OTEL is enabled.
from __future__ import annotations

import os
import sys
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
TRACE_EXEMPLARS = ROOT / "artifacts" / "ops" / "observability" / "traces.exemplars.log"
TRACE_SNAPSHOT = ROOT / "artifacts" / "ops" / "observability" / "traces.snapshot.log"
CONTRACT = ROOT / "docs" / "contracts" / "TRACE_SPANS.json"


def main() -> int:
    if os.getenv("ATLAS_E2E_ENABLE_OTEL", "0") != "1":
        print("trace coverage skipped (ATLAS_E2E_ENABLE_OTEL=0)")
        return 0
    if not TRACE_EXEMPLARS.exists() or not TRACE_SNAPSHOT.exists():
        print("trace coverage failed: missing trace artifacts", file=sys.stderr)
        return 1
    corpus = (TRACE_EXEMPLARS.read_text(errors="replace") + "\n" + TRACE_SNAPSHOT.read_text(errors="replace")).lower()
    contract = json.loads(CONTRACT.read_text(encoding="utf-8"))
    required_spans = tuple(s["name"] for s in contract.get("spans", []))
    missing = [s for s in required_spans if s not in corpus]
    if missing:
        print("trace coverage failed: missing required spans in captured traces", file=sys.stderr)
        for span in missing:
            print(f"- {span}", file=sys.stderr)
        return 1
    # Root span and request id signal must be present for every request class capture.
    root = contract.get("request_root_span", {})
    if root.get("name", "").lower() not in corpus:
        print("trace coverage failed: missing request_root span signal", file=sys.stderr)
        return 1
    if "request_id" not in corpus:
        print("trace coverage failed: missing request_id signal in traces", file=sys.stderr)
        return 1
    slow_query = contract.get("slow_query_event", {})
    if slow_query.get("name", "").lower() in corpus:
        for field in slow_query.get("required_fields", []):
            if field.lower() not in corpus:
                print(f"trace coverage failed: slow_query event missing field signal {field}", file=sys.stderr)
                return 1
    endpoint_class_signals = {
        "cheap": ("/v1/version", "/metrics"),
        "medium": ("/v1/sequence/region", "/v1/transcripts/"),
        "heavy": ("/v1/genes", "/v1/diff/"),
    }
    missing_classes: list[str] = []
    for cls, signals in endpoint_class_signals.items():
        if not any(sig.lower() in corpus for sig in signals):
            missing_classes.append(cls)
    if missing_classes:
        print("trace coverage failed: missing endpoint-class trace signals", file=sys.stderr)
        for cls in missing_classes:
            print(f"- {cls}", file=sys.stderr)
        return 1
    print("trace coverage check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
