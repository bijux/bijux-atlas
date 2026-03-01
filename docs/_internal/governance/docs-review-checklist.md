# Documentation Review Checklist

Owner: `docs-governance`  
Type: `policy`  
Reason to exist: define merge gates for documentation updates.

## Required Checks

- Page includes owner, audience, type, and reason-to-exist.
- Filename and directory naming use kebab-case.
- Content matches current commands, make targets, and behavior.
- Duplicate content is removed or redirected to the canonical page.
- Links resolve and point to canonical section entrypoints.
- Change is clearer and more actionable than the previous version.
- Examples are executable and failure behavior is explicit.
- Crate docs budget is respected (max 15 markdown files per crate docs/ directory) or an approved exception is linked.
