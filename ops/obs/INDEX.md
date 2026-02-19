# Obs INDEX

## Purpose
Own observability pack verification, dashboards, alerts, and drills.

## Public entrypoints
- `make ops-obs-up PROFILE=compose`
- `make ops-obs-verify`
- `make ops-drill-suite SUITE=<suite-id>`

## Suites
- `ops/obs/tests/suites.json`
- Runner: `ops/obs/tests/suite.sh --suite <suite-id>`

## Contracts
- `ops/CONTRACT.md`
- `ops/_meta/ownership.json`
