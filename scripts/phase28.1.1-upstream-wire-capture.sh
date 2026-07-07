#!/usr/bin/env bash
# Out-of-tree upstream BM1366 SERIALTX_DEBUG capture helper (Plan 28.1.1-01).
# Never modifies reference/esp-miner. Capture artifacts stay under scratch/ (gitignored).
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
reference_pin="c1915b0a63bfabebdb95a515cedfee05146c1d50"
scratch_dir="${BM1366_UPSTREAM_SCRATCH:-$repo_root/scratch/upstream-wire-capture}"
port="${1:-}"

usage() {
	printf 'usage: %s [port]\n' "$(basename "$0")" >&2
	printf '  Copies pinned reference to scratch, documents SERIALTX_DEBUG enablement,\n' >&2
	printf '  and prints recovery commands. Full upstream ESP-IDF build is operator-local.\n' >&2
	exit 1
}

[[ "${1:-}" == "-h" || "${1:-}" == "--help" ]] && usage

cd "$repo_root"
just verify-reference

mkdir -p "$scratch_dir"
if [[ ! -f "$scratch_dir/esp-miner/components/asic/bm1366.c" ]]; then
	rm -rf "$scratch_dir/esp-miner"
	cp -R reference/esp-miner "$scratch_dir/esp-miner"
	printf 'upstream_scratch_copy=created path=%s pin=%s\n' "$scratch_dir/esp-miner" "$reference_pin"
else
	printf 'upstream_scratch_copy=reused path=%s\n' "$scratch_dir/esp-miner"
fi

cat <<EOF
upstream_wire_capture_next_steps:
  1. In scratch tree only, set BM1366_SERIALTX_DEBUG and BM1366_SERIALRX_DEBUG to 1 in:
     $scratch_dir/esp-miner/components/asic/bm1366.c
  2. Build/flash upstream Ultra 205 image from scratch (not reference/esp-miner).
  3. Monitor >= 360 s with NVS seed + wifi-credentials; capture init + first-minute job TX.
  4. Store raw bytes locally under $scratch_dir/captures/ (gitignored).
  5. Recovery: just flash-monitor board=205 port=<port> capture-timeout-seconds=360 wifi-credentials=wifi-credentials.json
  6. just verify-reference (must PASS — pin untouched)
EOF

if [[ -n "$port" ]]; then
	printf 'detected_port_hint=%s\n' "$port"
fi
