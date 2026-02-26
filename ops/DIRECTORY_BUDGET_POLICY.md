# Directory Budget Policy

## Purpose

Keep `ops/` minimal, enforceable, and consumer-driven.

## Rules

- New directories require a concrete runtime/check consumer.
- New top-level directories under `ops/` are forbidden unless added to `ops/CONTRACT.md` and control-plane allowlists.
- Placeholder directories are allowed only when declared in `ops/inventory/placeholder-dirs.json` with explicit lifecycle policy.
- Directory depth must remain bounded by governance checks; deeper trees require explicit exception entries.
- Remove empty compatibility directories once all consumers are migrated.

## Review Criteria

- What breaks if this directory is removed?
- Which check, command surface, or runtime path consumes it?
- Is it authored input, generated output, or curated evidence?
