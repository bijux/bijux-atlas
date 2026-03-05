---
title: Make Wrapper Snippet
audience: contributor
type: reference
stability: stable
owner: platform
last_reviewed: 2026-03-05
tags:
  - make
  - snippet
---

# Make wrapper snippet

```make
sample-target: ## Short description for help output
	@$(DEV_ATLAS) <domain> <command> --format $(FORMAT)
```

Use one recipe line and keep execution logic in `bijux-dev-atlas`.
