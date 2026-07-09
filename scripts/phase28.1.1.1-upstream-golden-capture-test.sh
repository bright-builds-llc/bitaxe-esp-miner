#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp_root="$(mktemp -d)"
capture_dir="$repo_root/scratch/upstream-wire-capture/captures/helper-test-$$"
trap 'rm -rf "$tmp_root" "$capture_dir"' EXIT

wifi="$tmp_root/wifi.json"
pool="$tmp_root/pool.json"
mkdir -p "$capture_dir"

cat >"$wifi" <<'JSON'
{"ssid":"PHASE28_SENTINEL_WIFI","wifiPass":"PHASE28_SENTINEL_WIFI_PASS"}
JSON
cat >"$pool" <<'JSON'
{"poolURL":"PHASE28_SENTINEL_POOL","poolPort":3333,"poolUser":"PHASE28_SENTINEL_USER","poolPassword":"PHASE28_SENTINEL_POOL_PASS"}
JSON

output="$(
	cd "$repo_root"
	bash scripts/phase28.1.1.1-upstream-golden-capture.sh \
		--port /dev/null \
		--wifi-credentials "$wifi" \
		--pool-credentials "$pool" \
		--duration-seconds 360 \
		--capture-dir "$capture_dir" \
		--dry-run 2>&1
)"

printf '%s\n' "$output" | grep -q 'dry_run_status=passed'
if printf '%s\n' "$output" | grep -E 'PHASE28_SENTINEL_(WIFI|POOL|USER|PASS)' >/dev/null; then
	printf 'fixture credential value leaked in helper output\n' >&2
	exit 1
fi

printf 'phase28.1.1.1 upstream golden capture helper tests passed\n'
