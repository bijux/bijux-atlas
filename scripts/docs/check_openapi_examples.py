#!/usr/bin/env python3
# Purpose: verify OpenAPI response examples conform to declared schemas.
# Inputs: openapi/v1/openapi.snapshot.json
# Outputs: non-zero exit on schema/example mismatch
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OPENAPI = json.loads((ROOT / "openapi" / "v1" / "openapi.snapshot.json").read_text())
SCHEMAS = OPENAPI.get("components", {}).get("schemas", {})


def resolve(schema: dict) -> dict:
    if "$ref" in schema:
        ref = schema["$ref"]
        name = ref.split("/")[-1]
        return resolve(SCHEMAS[name])
    return schema


def validate(value, schema, path: str) -> list[str]:
    schema = resolve(schema)
    errs: list[str] = []
    typ = schema.get("type")
    if typ == "object":
        if not isinstance(value, dict):
            return [f"{path}: expected object"]
        for req in schema.get("required", []):
            if req not in value:
                errs.append(f"{path}: missing required field `{req}`")
        props = schema.get("properties", {})
        for k, v in value.items():
            if k in props:
                errs.extend(validate(v, props[k], f"{path}.{k}"))
    elif typ == "array":
        if not isinstance(value, list):
            return [f"{path}: expected array"]
        item_schema = schema.get("items", {})
        for i, item in enumerate(value):
            errs.extend(validate(item, item_schema, f"{path}[{i}]"))
    elif typ == "string" and not isinstance(value, str):
        errs.append(f"{path}: expected string")
    elif typ == "integer" and not isinstance(value, int):
        errs.append(f"{path}: expected integer")
    elif typ == "number" and not isinstance(value, (int, float)):
        errs.append(f"{path}: expected number")
    elif typ == "boolean" and not isinstance(value, bool):
        errs.append(f"{path}: expected boolean")
    return errs


def main() -> int:
    errors: list[str] = []
    for p, methods in OPENAPI.get("paths", {}).items():
        for method, op in methods.items():
            responses = op.get("responses", {})
            for code, resp in responses.items():
                for media, mobj in resp.get("content", {}).items():
                    schema = mobj.get("schema")
                    if not schema:
                        continue
                    if "example" in mobj:
                        errors.extend(validate(mobj["example"], schema, f"{method} {p} {code} {media}"))
                    for name, ex in mobj.get("examples", {}).items():
                        if "value" in ex:
                            errors.extend(validate(ex["value"], schema, f"{method} {p} {code} {media} examples.{name}"))
    if errors:
        print("openapi examples check failed:")
        for err in errors[:100]:
            print(f"- {err}")
        return 1
    print("openapi examples check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
