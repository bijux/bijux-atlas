# CLI Scripting Examples

## Bash

```bash
set -euo pipefail
python3 tools/cli/discover_subcommands.py --format json | jq '.operations'
```

## Python

```python
import json
import subprocess
payload = subprocess.check_output(["python3", "tools/cli/discover_subcommands.py", "--format", "json"], text=True)
print(json.loads(payload)["integration"])
```
