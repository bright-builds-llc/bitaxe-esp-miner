#!/usr/bin/env bash
# J3-equivalent capture: phase27 image + pool-input-bridge watcher (Plan 28.1.1-02).
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

port="/dev/cu.usbmodem1101"
evidence_dir=".planning/phases/28.1.1-bm1366-nonce-production-wire-parity/hardware-runs/wire-parity-run-04"
capture_timeout="360"
wifi_credentials="wifi-credentials.json"
pool_credentials="pool-credentials.json"
investigation_mode=""
chip_detect_investigation_mode=""
positional_index=0

while (($#)); do
	case "$1" in
	--wifi-credentials)
		[[ $# -ge 2 ]] || {
			printf 'missing value for --wifi-credentials\n' >&2
			exit 2
		}
		wifi_credentials="${2:-}"
		shift 2
		;;
	--pool-credentials)
		[[ $# -ge 2 ]] || {
			printf 'missing value for --pool-credentials\n' >&2
			exit 2
		}
		pool_credentials="${2:-}"
		shift 2
		;;
	--investigation)
		[[ $# -ge 2 ]] || {
			printf 'missing value for --investigation\n' >&2
			exit 2
		}
		investigation_mode="${2:-}"
		shift 2
		;;
	--chip-detect-investigation)
		[[ $# -ge 2 ]] || {
			printf 'missing value for --chip-detect-investigation\n' >&2
			exit 2
		}
		chip_detect_investigation_mode="${2:-}"
		shift 2
		;;
	-h | --help)
		printf 'usage: %s [port] [evidence-dir] [capture-timeout-seconds] [--wifi-credentials PATH] [--pool-credentials PATH] [--investigation MODE] [--chip-detect-investigation MODE]\n' "$(basename "$0")"
		exit 0
		;;
	--*)
		printf 'unknown argument: %s\n' "$1" >&2
		exit 2
		;;
	*)
		case "$positional_index" in
		0) port="$1" ;;
		1) evidence_dir="$1" ;;
		2) capture_timeout="$1" ;;
		*)
			printf 'unexpected positional argument: %s\n' "$1" >&2
			exit 2
			;;
		esac
		positional_index=$((positional_index + 1))
		shift
		;;
	esac
done

[[ -n "$wifi_credentials" ]] || {
	printf 'wifi credential path is required\n' >&2
	exit 2
}
[[ -n "$pool_credentials" ]] || {
	printf 'pool credential path is required\n' >&2
	exit 2
}

package_args=()
if [[ -n "$investigation_mode" ]]; then
	package_args+=(--investigation "$investigation_mode")
fi
if [[ -n "$chip_detect_investigation_mode" ]]; then
	package_args+=(--chip-detect-investigation "$chip_detect_investigation_mode")
fi

# Empty array under `set -u` is unbound on some bash versions; expand safely.
if ((${#package_args[@]} > 0)); then
	bash scripts/phase27-live-hardware-bridge-package.sh "${package_args[@]}" >/dev/null
else
	bash scripts/phase27-live-hardware-bridge-package.sh >/dev/null
fi

image_path="${repo_root}/bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin"
mkdir -p "${evidence_dir}/pool-input-bridge"
monitor_log="${evidence_dir}/flash-monitor.log"

just flash-monitor \
	board=205 \
	port="${port}" \
	image=bazel-bin/firmware/bitaxe/bitaxe-ultra205-factory.bin \
	capture-timeout-seconds="${capture_timeout}" \
	wifi-credentials="${wifi_credentials}" \
	evidence-dir="${evidence_dir}" \
	>"${evidence_dir}/flash.stdout.log" 2>"${evidence_dir}/flash.stderr.log" &
capture_pid=$!

pool_bridge_applied=0
deadline=$((SECONDS + capture_timeout + 120))
while ((SECONDS < deadline)); do
	if [[ -f "$monitor_log" && "$pool_bridge_applied" -eq 0 ]]; then
		if device_url="$(rg -o 'device_url=http://[0-9.]+' "$monitor_log" | head -1 | cut -d= -f2-)"; then
			if [[ -n "$device_url" && -f "$pool_credentials" ]]; then
				if bash scripts/phase21-pool-input-bridge.sh \
					--device-url "$device_url" \
					--pool-credentials "$pool_credentials" \
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
