# Architecture

## Architecture

`bijux-atlas-api` is the wire-contract crate for Atlas v1. It defines request parsing, response envelope types, error contracts, and OpenAPI schema generation.

Business logic, storage access, query execution, and networking runtime concerns are outside this crate.
