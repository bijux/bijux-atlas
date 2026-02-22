# Scope: registry wrapper surface.
# Public targets: none
SHELL := /bin/sh

registry-list: ## Print atlasctl registry inventory
	@./bin/atlasctl registry list

.PHONY: registry-list
