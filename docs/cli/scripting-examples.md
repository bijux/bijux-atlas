# CLI Scripting Examples

## Bash

```bash
set -euo pipefail
bijux-dev-atlas help --format json | jq '.commands'
```

## Python

```python
import json
import subprocess
payload = subprocess.check_output(["bijux-dev-atlas", "help", "--format", "json"], text=True)
print(json.loads(payload))
```
