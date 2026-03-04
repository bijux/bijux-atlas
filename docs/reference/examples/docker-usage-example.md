# Docker Usage Example

- Owner: `bijux-atlas-operations`
- Audience: `operator`
- Type: `guide`
- Stability: `stable`
- Reason to exist: provide a baseline Docker workflow example for local image handling.

## Example

```bash
docker build -t bijux-atlas:local .
docker run --rm -p 8080:8080 bijux-atlas:local
```
