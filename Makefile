SHELL := /bin/sh

JOBS ?= $(shell nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 8)

include makefiles/cargo.mk
include makefiles/cargo-dev.mk
include makefiles/policies.mk

.DEFAULT_GOAL := help

help:
	@printf '%s\n' \
	  'targets: fmt lint check test test-all coverage audit openapi-drift ci fetch-fixtures load-test load-test-1000qps cold-start-bench memory-profile-load run-medium-ingest run-medium-serve culprits-all culprits-max_loc culprits-max_depth culprits-file-max_rs_files_per_dir culprits-file-max_modules_per_dir' \
	  'dev targets: dev-fmt dev-lint dev-check dev-test dev-test-all dev-coverage dev-audit dev-ci dev-clean'
