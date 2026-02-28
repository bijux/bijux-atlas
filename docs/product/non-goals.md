# Non Goals

Owner: `product`  
Type: `concept`  
Reason to exist: prevent scope creep and preserve contract stability.

## Out Of Scope

- Atlas does not perform variant interpretation.
- Atlas does not mutate published datasets.
- Atlas does not accept implicit dataset defaults.
- Atlas does not execute remote code from requests.
- Atlas does not expose write endpoints for genomic entities.

## Enforcement

Requests requiring excluded behavior must be rejected with explicit errors.
