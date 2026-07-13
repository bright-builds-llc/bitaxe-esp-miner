#!/usr/bin/env bash
set -euo pipefail
script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
exec /bin/bash "$script_dir/phase28.1.1-terminal-closure-entrypoint-test.sh" "$script_dir/phase28.1.1.3-rx-acquisition-diagnostic.sh"
