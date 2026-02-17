#!/usr/bin/env python3
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
contract = json.loads((ROOT / "docs" / "contracts" / "ENDPOINTS.json").read_text())
contract_paths = {e["path"] for e in contract["endpoints"]}

server_src = (ROOT / "crates" / "bijux-atlas-server" / "src" / "runtime" / "server_runtime_app.rs").read_text()
route_paths = set()
for p in re.findall(r'\.route\(\s*"([^"]+)"', server_src, flags=re.MULTILINE):
    p = re.sub(r":([A-Za-z_][A-Za-z0-9_]*)", r"{\1}", p)
    if p != "/":
        route_paths.add(p)

openapi = json.loads((ROOT / "openapi" / "v1" / "openapi.snapshot.json").read_text())
openapi_paths = set(openapi.get("paths", {}).keys())

if route_paths != contract_paths:
    print("endpoint contract drift with server routing", file=sys.stderr)
    print("missing in contract:", sorted(route_paths - contract_paths), file=sys.stderr)
    print("extra in contract:", sorted(contract_paths - route_paths), file=sys.stderr)
    sys.exit(1)
if openapi_paths != contract_paths:
    print("endpoint contract drift with OpenAPI", file=sys.stderr)
    print("missing in contract:", sorted(openapi_paths - contract_paths), file=sys.stderr)
    print("extra in contract:", sorted(contract_paths - openapi_paths), file=sys.stderr)
    sys.exit(1)

print("endpoints contract check passed")
