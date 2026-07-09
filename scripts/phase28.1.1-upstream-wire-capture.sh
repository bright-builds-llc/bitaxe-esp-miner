#!/usr/bin/env bash
# Out-of-tree upstream BM1366 SERIALTX_DEBUG capture helper (Plan 28.1.1-01).
# Never modifies reference/esp-miner. Capture artifacts stay under scratch/ (gitignored).
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
reference_pin="c1915b0a63bfabebdb95a515cedfee05146c1d50"
scratch_dir="${BM1366_UPSTREAM_SCRATCH:-$repo_root/scratch/upstream-wire-capture}"
upstream_source="${BM1366_UPSTREAM_SOURCE:-$repo_root/reference/esp-miner}"
verify_reference_bin="${PHASE28_VERIFY_REFERENCE_BIN:-}"
port="${1:-}"

usage() {
	printf 'usage: %s [port]\n' "$(basename "$0")" >&2
	printf '  Copies pinned reference to scratch, documents SERIALTX_DEBUG enablement,\n' >&2
	printf '  and prints recovery commands. Full upstream ESP-IDF build is operator-local.\n' >&2
	exit 1
}

[[ "${1:-}" == "-h" || "${1:-}" == "--help" ]] && usage

verify_reference() {
	if [[ -n "$verify_reference_bin" ]]; then
		"$verify_reference_bin"
	else
		just verify-reference
	fi
}

write_unavailable_markers() {
	local stage
	for stage in post_enumerate post_mining_ready post_max_baud post_mask_reload post_first_work; do
		printf 'accepted_state_snapshot stage=%s observation=unavailable chip_count_class=unavailable readable_responses=0 error_counter_active=false domain_counter_active=false total_counter_active=false power_delta_class=unavailable result_correlated=false submit_observed=false redacted=true\n' "$stage"
	done
}

cd "$repo_root"
verify_reference

mkdir -p "$scratch_dir"
if [[ ! -f "$scratch_dir/esp-miner/components/asic/bm1366.c" ]]; then
	rm -rf "$scratch_dir/esp-miner"
	cp -R "$upstream_source" "$scratch_dir/esp-miner"
	printf 'upstream_scratch_copy=created path=%s pin=%s\n' "$scratch_dir/esp-miner" "$reference_pin"
else
	printf 'upstream_scratch_copy=reused path=%s\n' "$scratch_dir/esp-miner"
fi

header="$scratch_dir/esp-miner/components/asic/include/bm1366.h"
[[ -f "$header" ]] || {
	printf 'upstream_wire_capture_error: scratch BM1366 header missing\n' >&2
	exit 1
}
sed -i.bak \
	-e 's/^#define BM1366_SERIALTX_DEBUG false$/#define BM1366_SERIALTX_DEBUG true/' \
	-e 's/^#define BM1366_SERIALRX_DEBUG false$/#define BM1366_SERIALRX_DEBUG true/' \
	"$header"
rm -f "$header.bak"

mkdir -p "$scratch_dir/captures"
write_unavailable_markers >"$scratch_dir/captures/accepted-state-template.log"
verify_reference

cat <<EOF
upstream_wire_capture_next_steps:
  1. Scratch-only BM1366 TX/RX diagnostics are enabled in:
     $scratch_dir/esp-miner/components/asic/include/bm1366.h
  2. Build/flash upstream Ultra 205 image from scratch (not reference/esp-miner).
  3. Monitor >= 360 s with NVS seed + wifi-credentials; capture init + first-minute job TX.
  4. Store raw bytes locally under $scratch_dir/captures/ (gitignored).
  5. Recovery: just flash-monitor board=205 port=<port> capture-timeout-seconds=360 wifi-credentials=wifi-credentials.json
  6. just verify-reference (must PASS — pin untouched)
  7. Normalize captured safe-register activity into the closed marker vocabulary in:
     $scratch_dir/captures/accepted-state-template.log
EOF

if [[ -n "$port" ]]; then
	printf 'detected_port_hint=%s\n' "$port"
fi
