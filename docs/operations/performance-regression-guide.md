# Performance Regression Guide

- Owner: `platform`
- Stability: `stable`
- Last verified against: `main@2228f79ef`

## Purpose

Provide the operator workflow for investigating and resolving performance regressions.

## Workflow

1. Run regression detector against baseline and candidate.
2. Review classified severities and affected suites.
3. Confirm trend and anomaly assets.
4. Decide rollback, fix-forward, or threshold exception path.
5. Publish updated regression report and audit report.
