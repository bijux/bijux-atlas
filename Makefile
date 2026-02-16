SHELL := /bin/sh

JOBS ?= $(shell nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 8)

include makefiles/cargo.mk
include makefiles/cargo-dev.mk

.DEFAULT_GOAL := help

help:
	@printf '%s\n' \
	  'targets: fmt lint check test test-all coverage audit openapi-drift ci fetch-fixtures load-test run-medium-ingest run-medium-serve' \
	  'dev targets: dev-fmt dev-lint dev-check dev-test dev-test-all dev-coverage dev-audit dev-ci dev-clean'
