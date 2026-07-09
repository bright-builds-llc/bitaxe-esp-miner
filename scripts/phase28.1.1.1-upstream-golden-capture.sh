#!/usr/bin/env bash
# Capture upstream ESP-Miner BM1366 debug frames from an ignored scratch build.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
reference_pin="c1915b0a63bfabebdb95a515cedfee05146c1d50"
scratch_root="${BM1366_UPSTREAM_SCRATCH:-$repo_root/scratch/upstream-wire-capture}"
upstream_dir="$scratch_root/esp-miner"
port=""
wifi_credentials=""
pool_credentials=""
duration_seconds="360"
capture_dir=""
dry_run="false"

usage() {
	printf 'usage: %s --port PATH --wifi-credentials PATH --pool-credentials PATH --duration-seconds N --capture-dir PATH [--dry-run]\n' "$(basename "$0")"
	printf '  Builds and flashes an ignored upstream BM1366 debug image, then captures serial logs.\n'
	printf '  Raw logs stay under an ignored scratch capture directory. Credential values are never printed.\n'
}

die() {
	printf 'upstream_golden_capture_error: %s\n' "$*" >&2
	exit 1
}

while (($#)); do
	case "$1" in
	--port)
		port="${2:-}"
		shift 2
		;;
	--wifi-credentials)
		wifi_credentials="${2:-}"
		shift 2
		;;
	--pool-credentials)
		pool_credentials="${2:-}"
		shift 2
		;;
	--duration-seconds)
		duration_seconds="${2:-}"
		shift 2
		;;
	--capture-dir)
		capture_dir="${2:-}"
		shift 2
		;;
	--dry-run)
		dry_run="true"
		shift
		;;
	-h | --help)
		usage
		exit 0
		;;
	*)
		die "unknown argument: $1"
		;;
	esac
done

[[ -n "$port" ]] || die "--port is required"
[[ -n "$wifi_credentials" ]] || die "--wifi-credentials is required"
[[ -n "$pool_credentials" ]] || die "--pool-credentials is required"
[[ -n "$capture_dir" ]] || die "--capture-dir is required"
[[ "$duration_seconds" =~ ^[0-9]+$ ]] || die "--duration-seconds must be an integer"
((duration_seconds >= 360)) || die "--duration-seconds must be at least 360"

cd "$repo_root"
[[ -f "$wifi_credentials" ]] || die "Wi-Fi credential file is missing"
[[ -f "$pool_credentials" ]] || die "pool credential file is missing"
mkdir -p "$capture_dir"
git check-ignore -q "$capture_dir/probe.raw.log" ||
	die "capture directory must be ignored by git: $capture_dir"

nvs_work_dir="$capture_dir/nvs"
nvs_csv="$nvs_work_dir/config-205-local.cvs"
nvs_bin="$nvs_work_dir/config.bin"
mkdir -p "$nvs_work_dir"

node scripts/phase28.1.1.1-upstream-nvs-csv.mjs \
	--base-csv "$repo_root/reference/esp-miner/config-205.cvs" \
	--wifi-credentials "$wifi_credentials" \
	--pool-credentials "$pool_credentials" \
	--out "$nvs_csv"

if [[ "$dry_run" == "true" ]]; then
	printf 'dry_run_status=passed\n'
	printf 'capture_dir_ignored=true\n'
	printf 'credential_values_printed=false\n'
	exit 0
fi

run_logged() {
	local log_path="$1"
	shift
	"$@" >"$log_path" 2>&1
}

run_logged_with_timeout() {
	local timeout_seconds="$1"
	local log_path="$2"
	shift 2

	"$@" >"$log_path" 2>&1 &
	local command_pid=$!
	local deadline=$((SECONDS + timeout_seconds))

	while kill -0 "$command_pid" 2>/dev/null; do
		if ((SECONDS >= deadline)); then
			kill "$command_pid" 2>/dev/null || true
			wait "$command_pid" 2>/dev/null || true
			printf 'command_timed_out_after_seconds=%s\n' "$timeout_seconds" >>"$log_path"
			return 124
		fi
		sleep 1
	done

	wait "$command_pid"
}

count_log_matches() {
	local pattern="$1"
	local log_path="$2"

	{ rg -a -i --count-matches "$pattern" "$log_path" 2>/dev/null || true; } |
		awk -F: '{if (NF == 1) sum += $1; else sum += $NF} END {print sum + 0}'
}

count_bm1366_job_frames() {
	local log_path="$1"

	node - "$log_path" <<'NODE'
const fs = require("node:fs");

const logPath = process.argv[2];
const logText = fs.readFileSync(logPath, "latin1");
let count = 0;

for (const line of logText.split(/\r?\n/)) {
  const maybeDebug = line.match(/(?:^|\s)(?:tx|rx):\s*\[([0-9a-fA-F ]+)\]/i);
  if (!maybeDebug) continue;

  const bytes = [...maybeDebug[1].matchAll(/\b[0-9a-fA-F]{2}\b/g)].map((match) =>
    Number.parseInt(match[0], 16),
  );
  if (
    bytes.length === 88 &&
    bytes[0] === 0x55 &&
    bytes[1] === 0xaa &&
    bytes[2] === 0x21 &&
    bytes[3] === 0x56
  ) {
    count += 1;
  }
}

process.stdout.write(`${count}\n`);
NODE
}

write_usb_console_defaults() {
	local defaults_path="$1"

	cat >"$defaults_path" <<'EOF'
# Scratch-only Phase 28.1.1.1 capture override.
# Keep app logs on the USB Serial/JTAG path used by espflash monitor.
# CONFIG_ESP_CONSOLE_UART_DEFAULT is not set
CONFIG_ESP_CONSOLE_USB_SERIAL_JTAG=y
# CONFIG_ESP_CONSOLE_USB_CDC is not set
# CONFIG_ESP_CONSOLE_UART_CUSTOM is not set
# CONFIG_ESP_CONSOLE_NONE is not set
CONFIG_ESP_CONSOLE_SECONDARY_NONE=y
# CONFIG_ESP_CONSOLE_SECONDARY_USB_SERIAL_JTAG is not set
CONFIG_ESP_CONSOLE_USB_SERIAL_JTAG_ENABLED=y
# CONFIG_ESP_CONSOLE_UART is not set
EOF
}

classify_monitor_log() {
	local monitor_log="$1"
	local classification_path="$2"
	local app_marker_pattern
	local debug_frame_pattern
	local app_marker_count
	local bm1366_debug_frame_count
	local bm1366_job_frame_count

	app_marker_pattern='Welcome to the bitaxe|Device Model:|Board Version:|ASIC:|Initializing serial|Set chip address|Setting Frequency|Send Job:|stratum|pool|I2C initialized|RST pin initialized'
	debug_frame_pattern='(^|[[:space:]])(tx|rx):[[:space:]]*\[[0-9a-f ]+\]'
	app_marker_count="$(count_log_matches "$app_marker_pattern" "$monitor_log")"
	bm1366_debug_frame_count="$(count_log_matches "$debug_frame_pattern" "$monitor_log")"
	bm1366_job_frame_count="$(count_bm1366_job_frames "$monitor_log")"

	{
		printf 'app_marker_count=%s\n' "$app_marker_count"
		printf 'bm1366_debug_frame_count=%s\n' "$bm1366_debug_frame_count"
		printf 'bm1366_job_frame_count=%s\n' "$bm1366_job_frame_count"
		printf 'raw_bytes_committed=false\n'
		printf 'credential_values_printed=false\n'
	} >"$classification_path"

	if ((app_marker_count == 0)); then
		printf 'upstream_capture_status=blocked_app_log_missing\n' >&2
		return 10
	fi

	if ((bm1366_job_frame_count == 0)); then
		printf 'upstream_capture_status=blocked_bm1366_job_frame_missing\n' >&2
		return 11
	fi
}

run_logged "$capture_dir/verify-reference.raw.log" just verify-reference
run_logged "$capture_dir/detect-ultra205.raw.log" just detect-ultra205
detected_port="$(sed -n 's/^port=//p' "$capture_dir/detect-ultra205.raw.log" | tail -1)"
[[ "$detected_port" == "$port" ]] ||
	die "detected Ultra 205 port did not match requested port"

recreate_scratch="false"
if [[ ! -d "$upstream_dir" ]]; then
	recreate_scratch="true"
elif ! git -C "$upstream_dir" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
	recreate_scratch="true"
fi

if [[ "$recreate_scratch" == "true" ]]; then
	rm -rf "$upstream_dir"
	run_logged "$capture_dir/upstream-clone.raw.log" \
		git clone --recurse-submodules "$repo_root/reference/esp-miner" "$upstream_dir"
else
	run_logged "$capture_dir/upstream-reset.raw.log" \
		git -C "$upstream_dir" reset --hard "$reference_pin"
fi

run_logged "$capture_dir/upstream-checkout.raw.log" \
	git -C "$upstream_dir" checkout --detach "$reference_pin"
run_logged "$capture_dir/upstream-submodules.raw.log" \
	git -C "$upstream_dir" submodule update --init --recursive
run_logged "$capture_dir/upstream-clean.raw.log" \
	git -C "$upstream_dir" clean -ffdx -- build sdkconfig sdkconfig.old sdkconfig.phase28.usb-console.defaults dependencies.lock

phase_sdkconfig_defaults="$upstream_dir/sdkconfig.phase28.usb-console.defaults"
write_usb_console_defaults "$phase_sdkconfig_defaults"

bm1366_header="$upstream_dir/components/asic/include/bm1366.h"
perl -0pi -e 's/#define BM1366_SERIALTX_DEBUG false/#define BM1366_SERIALTX_DEBUG true/' "$bm1366_header"
perl -0pi -e 's/#define BM1366_SERIALRX_DEBUG false/#define BM1366_SERIALRX_DEBUG true/' "$bm1366_header"
perl -0pi -e 's/#define BM1366_DEBUG_WORK false/#define BM1366_DEBUG_WORK true/' "$bm1366_header"
perl -0pi -e 's/#define BM1366_DEBUG_JOBS false/#define BM1366_DEBUG_JOBS true/' "$bm1366_header"
grep -q '#define BM1366_SERIALTX_DEBUG true' "$bm1366_header" ||
	die "failed to enable BM1366_SERIALTX_DEBUG in scratch"
grep -q '#define BM1366_SERIALRX_DEBUG true' "$bm1366_header" ||
	die "failed to enable BM1366_SERIALRX_DEBUG in scratch"
grep -q '#define BM1366_DEBUG_WORK true' "$bm1366_header" ||
	die "failed to enable BM1366_DEBUG_WORK in scratch"
grep -q '#define BM1366_DEBUG_JOBS true' "$bm1366_header" ||
	die "failed to enable BM1366_DEBUG_JOBS in scratch"

if ! docker image inspect espminer-build:latest >/dev/null 2>&1; then
	run_logged_with_timeout "${PHASE28_DOCKER_BUILD_TIMEOUT_SECONDS:-900}" "$capture_dir/docker-build.raw.log" \
		docker build -t espminer-build "$upstream_dir/.devcontainer" ||
		die "Docker build failed or timed out; see ignored raw capture log"
fi

run_logged_with_timeout "${PHASE28_UPSTREAM_BUILD_TIMEOUT_SECONDS:-1800}" "$capture_dir/upstream-build.raw.log" \
	docker run --rm -v "$upstream_dir:/workspace" espminer-build /bin/bash -lc \
	'git config --global --add safe.directory /workspace && cd /workspace && idf.py -D "SDKCONFIG_DEFAULTS=sdkconfig.defaults;sdkconfig.phase28.usb-console.defaults" build && ./merge_bin.sh ./esp-miner-merged.bin' ||
	die "upstream firmware build failed or timed out; see ignored raw capture log"
grep -q 'CONFIG_ESP_CONSOLE_USB_SERIAL_JTAG=y' "$upstream_dir/sdkconfig" ||
	die "upstream build did not use USB Serial/JTAG console override"

nvs_python="${ESP_IDF_NVS_PYTHON:-$repo_root/.embuild/espressif/python_env/idf5.5_py3.14_env/bin/python}"
[[ -x "$nvs_python" ]] || die "ESP-IDF NVS Python not found: $nvs_python"
run_logged "$capture_dir/nvs-generate.raw.log" \
	"$nvs_python" -m esp_idf_nvs_partition_gen generate "$nvs_csv" "$nvs_bin" 0x6000

run_logged "$capture_dir/upstream-flash.raw.log" \
	espflash write-bin --chip esp32s3 --port "$port" --non-interactive 0x0 "$upstream_dir/esp-miner-merged.bin"
run_logged "$capture_dir/upstream-nvs-flash.raw.log" \
	espflash write-bin --chip esp32s3 --port "$port" --non-interactive 0x9000 "$nvs_bin"

monitor_log="$capture_dir/upstream-monitor.raw.log"
espflash monitor --chip esp32s3 --port "$port" --non-interactive >"$monitor_log" 2>&1 &
monitor_pid=$!
trap 'kill "$monitor_pid" 2>/dev/null || true' EXIT
sleep "$duration_seconds"
kill "$monitor_pid" 2>/dev/null || true
wait "$monitor_pid" 2>/dev/null || true
trap - EXIT
classify_monitor_log "$monitor_log" "$capture_dir/upstream-monitor-classification.txt" ||
	die "upstream monitor gate failed; see ignored raw capture log"

printf 'upstream_capture_status=complete\n'
printf 'capture_dir=%s\n' "$capture_dir"
printf 'monitor_log=%s\n' "$monitor_log"
