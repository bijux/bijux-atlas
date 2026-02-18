#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
import json
import os
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
CONTRACT = ROOT / "ops/observability/contract/metrics-contract.json"
TRACE_EXEMPLARS = ROOT / "artifacts/ops/observability/traces.exemplars.log"

contract = json.loads(CONTRACT.read_text())
required = contract.get("required_spans", [])
if not required:
    print("no required_spans in contract", file=sys.stderr)
    sys.exit(1)

corpus = "\n".join(
    p.read_text()
    for p in (ROOT / "crates/bijux-atlas-server/src").rglob("*.rs")
)

missing = [s for s in required if s not in corpus]
if missing:
    print("required tracing spans/messages not found in source:", file=sys.stderr)
    for s in missing:
        print(f"- {s}", file=sys.stderr)
    sys.exit(1)

# Span coverage contract by query class (source-level guardrail).
required_by_query_class = {
    "cheap": ["admission_control", "dataset_resolve", "serialize_response"],
    "medium": ["admission_control", "dataset_resolve", "serialize_response"],
    "heavy": [
        "admission_control",
        "dataset_resolve",
        "cache_lookup",
        "store_fetch",
        "sqlite_query",
        "serialize_response",
    ],
}
for cls, spans in required_by_query_class.items():
    class_missing = [s for s in spans if s not in corpus]
    if class_missing:
        print(f"query class `{cls}` missing required spans in source:", file=sys.stderr)
        for span in class_missing:
            print(f"- {span}", file=sys.stderr)
        sys.exit(1)

# Require cache/store-path span coverage in source to keep trace contracts meaningful.
cache_span_tokens = ("cache_lookup", "store_fetch", "open_db")
if not any(token in corpus for token in cache_span_tokens):
    print("required cache/store tracing span tokens not found in source", file=sys.stderr)
    sys.exit(1)

# If exemplar traces were captured, ensure they include DB + cache/store spans.
if os.getenv("ATLAS_E2E_ENABLE_OTEL", "0") == "1" and TRACE_EXEMPLARS.exists():
    exemplars = TRACE_EXEMPLARS.read_text().strip()
    if exemplars:
        exemplar_missing: list[str] = []
        if "sqlite_query" not in exemplars:
            exemplar_missing.append("sqlite_query")
        if not any(token in exemplars for token in cache_span_tokens):
            exemplar_missing.append("cache/store span (cache_lookup|store_fetch|open_db)")
        if exemplar_missing:
            print("trace exemplars missing required spans:", file=sys.stderr)
            for item in exemplar_missing:
                print(f"- {item}", file=sys.stderr)
            sys.exit(1)

print("tracing contract passed")
