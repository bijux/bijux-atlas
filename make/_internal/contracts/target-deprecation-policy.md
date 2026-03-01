# Make target deprecation policy

## Purpose

Provide a stable process to deprecate curated make targets without silent breakage.

## Rules

1. Announce deprecation in docs and release notes before removal.
2. Provide a clear replacement target or control-plane command.
3. Keep a compatibility alias during the deprecation window when feasible.
4. Define a removal date and enforce it in a follow-up change.
5. Communicate migration steps for users and CI workflows.
