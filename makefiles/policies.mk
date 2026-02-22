# Scope: policy report wrapper.
# Public targets: none
SHELL := /bin/sh

bypass-report: ## Emit consolidated policies bypass report
	@./bin/atlasctl policies bypass report --out artifacts/reports/atlasctl/policies-bypass-report.json

.PHONY: bypass-report
