# Automation Doctrine Rules

1. Repository automation executes through `bijux-dev-atlas` command surfaces.
2. Root `tools/` and `scripts/` directories are forbidden.
3. Root automation scripts (`*.sh`, `*.py`) are forbidden.
4. Workflow shell execution must not bypass governed command surfaces.
5. Make wrappers remain delegation-only.
