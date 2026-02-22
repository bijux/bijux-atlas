# Ops Intent-Based Naming

- Owner: `bijux-atlas-operations`

Use durable names that describe behavior and contract intent.

## Rules

- Name by behavior, not execution order.
- Prefer domain nouns (`store-outage-under-spike`) over temporal labels (`mid-load`).
- Use `suite.sh` only for manifest-defined suites.
- Other orchestration must be Make targets, not ad-hoc `run_all.sh` calls.
- Use `store` for backend role names; backend vendor stays in config.

## Enforced By

- `./bin/atlasctl ops naming-check --report text`
- `./bin/atlasctl ops no-direct-script-usage-check --report text`
- `atlasctl docs durable-naming-check --report text`
