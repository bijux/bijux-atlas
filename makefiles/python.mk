# Scope: shared python/tool CLI invocation contract for makefiles.
# Public targets: none
SHELL := /bin/sh

# Shared Python toolchain env defaults only (no atlasctl invoker aliases here).
PYTHONPATH ?= packages/atlasctl/src
ATLASCTL_ARTIFACT_ROOT ?= artifacts/atlasctl

internal/scripts/cli-check:
	@./bin/atlasctl --version >/dev/null 2>&1 || { \
		echo "atlasctl CLI is not runnable via ./bin/atlasctl"; \
		echo "run: make scripts-install or make dev-bootstrap"; \
		exit 2; \
	}

.PHONY: internal/scripts/cli-check
