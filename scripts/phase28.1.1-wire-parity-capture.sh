#!/usr/bin/env bash
# J3-equivalent capture: phase27 image + pool-input-bridge watcher (Plan 28.1.1-02).
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

port="${1:-/dev/cu.usbmodem1101}"
evidence_dir="${2:-.planning/phases/28.1.1-bm1366-nonce-production-wire-parity/hardware-runs/wire-parity-run-04}"
capture_timeout="${3:-360}"

bash scripts/phase27-live-hardware-bridge-package.sh >/dev/null

image_path="${repo_root}/bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin"
mkdir -p "${evidence_dir}/pool-input-bridge"
monitor_log="${evidence_dir}/flash-monitor.log"

just flash-monitor \
	board=205 \
	port="${port}" \
	image=bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin \
	capture-timeout-seconds="${capture_timeout}" \
	wifi-credentials=wifi-credentials.json \
	evidence-dir="${evidence_dir}" \
	>"${evidence_dir}/flash.stdout.log" 2>"${evidence_dir}/flash.stderr.log" &
capture_pid=$!

pool_bridge_applied=0
deadline=$((SECONDS + capture_timeout + 120))
while ((SECONDS < deadline)); do
	if [[ -f "$monitor_log" && "$pool_bridge_applied" -eq 0 ]]; then
		if device_url="$(rg -o 'device_url=http://[0-9.]+' "$monitor_log" | head -1 | cut -d= -f2-)"; then
			if [[ -n "$device_url" && -f pool-credentials.json ]]; then
				if bash scripts/phase21-pool-input-bridge.sh \
					--device-url "$device_url" \
					--pool-credentials pool-credentials.json \
					--out-dir "${evidence_dir}/pool-input-bridge"; then
					pool_bridge_applied=1
					echo "pool_input_bridge_status=applied" >>"${evidence_dir}/capture-meta.log"
				fi
			fi
		fi
	fi
	if ! kill -0 "$capture_pid" 2>/dev/null; then
		break
	fi
	sleep 2
done

wait "$capture_pid" || true
echo "wire_parity_capture_complete pool_bridge_applied=${pool_bridge_applied}"
