# Scope: policy report wrapper.
# Public targets: none
SHELL := /bin/sh

bypass-report: ## Emit consolidated policies bypass report
	@./bin/atlasctl report bypass --json --out artifacts/reports/atlasctl/policies-bypass-report.json

.PHONY: bypass-report
