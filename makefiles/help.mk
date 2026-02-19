SHELL := /bin/sh

.DEFAULT_GOAL := help

help: ## Show curated public make targets from SSOT
	@python3 ./scripts/layout/render_public_help.py

.PHONY: help
