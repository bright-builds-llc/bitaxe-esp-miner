#!/usr/bin/env bash
# Host-side dump of BM1366 mining-ready init TX frames for wire-diff review.
# No hardware required; output is safe to commit (init frames only, no pool job bytes).
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

cargo test -p bitaxe-asic dump_dynamic_init_frames_for_fixture_capture -- --ignored --nocapture 2>&1 \
	| rg '^frames\[' || true

echo "bm1366_init_tx_dump=complete"
