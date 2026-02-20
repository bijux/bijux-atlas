# Scope: shared python/tool CLI invocation contract for makefiles.
# Public targets: none
SHELL := /bin/sh

ATLAS_SCRIPTS ?= ./bin/bijux-atlas
PY_RUN ?= $(ATLAS_SCRIPTS) run

internal/scripts/cli-check:
	@[ -x "$(ATLAS_SCRIPTS)" ] || { \
		echo "missing atlas-scripts CLI at $(ATLAS_SCRIPTS)"; \
		echo "run: make dev-bootstrap"; \
		exit 2; \
	}

.PHONY: internal/scripts/cli-check
