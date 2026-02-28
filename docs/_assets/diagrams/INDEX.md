# Diagrams

## What
This directory stores canonical diagram sources and exported assets for architecture and operations documentation.

## Why
Diagram sources in one location provide deterministic rendering and stable references across docs.

## Contracts
- File names must use kebab-case.
- Source files use `.mmd` (Mermaid) or `.puml` (PlantUML).
- Exported files use `.svg` or `.png` with matching base names.
- Docs must link diagrams through `docs/_assets/diagrams/`.

## Failure Modes
- Missing source files prevents deterministic regeneration.
- Untracked binary assets cause review gaps.

## How to Verify
```bash
bijux dev atlas docs render-diagrams --report text
```

See also:
- [Architecture Index](../../architecture/index.md)
- [Docs Review Checklist](../../governance/docs-review-checklist.md)
- [Style Guide](../../governance/style-guide.md)
