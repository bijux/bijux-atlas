# Scope: shared python/tool CLI invocation contract for makefiles.
# Public targets: none
SHELL := /bin/sh

ATLAS_SCRIPTS ?= PYTHONPATH=packages/bijux-atlas-scripts/src python3 -m bijux_atlas_scripts.cli
SCRIPTS ?= $(ATLAS_SCRIPTS)
PY_RUN ?= $(SCRIPTS) run

internal/scripts/cli-check:
	@PYTHONPATH=packages/bijux-atlas-scripts/src python3 -m bijux_atlas_scripts.cli --version >/dev/null 2>&1 || { \
		echo "atlasctl python module is not runnable via $(ATLAS_SCRIPTS)"; \
		echo "run: make scripts-install or make dev-bootstrap"; \
		exit 2; \
	}

.PHONY: internal/scripts/cli-check
