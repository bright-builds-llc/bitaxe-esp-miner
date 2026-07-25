#!/usr/bin/env bash
set -euo pipefail

readonly script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
readonly workspace_dir="${BUILD_WORKSPACE_DIRECTORY:-$(git -C "$script_dir/.." rev-parse --show-toplevel)}"

operation=""
board=""
port=""
attempt_child=""
package_identity_digest=""
manifest_path=""
manifest_digest=""
firmware_elf_path=""
firmware_elf_digest=""
executable_image_path=""
executable_image_digest=""
factory_image_path=""
factory_image_digest=""
capture_timeout_seconds=""
wall_clock_timeout_seconds=""
wifi_credentials=""
trusted_origin=""

fail() {
	printf 'category=%s\n' "$1" >&2
	exit 2
}

assign_once() {
	local name="$1"
	local value="$2"
	[[ -z "${!name}" ]] || fail duplicate_argument
	printf -v "$name" '%s' "$value"
}

for argument in "$@"; do
	[[ "$argument" == *=* ]] || fail invalid_argument
	name="${argument%%=*}"
	value="${argument#*=}"
	[[ -n "$value" ]] || fail invalid_argument
	case "$name" in
	operation) assign_once operation "$value" ;;
	board) assign_once board "$value" ;;
	port) assign_once port "$value" ;;
	attempt-child) assign_once attempt_child "$value" ;;
	package-identity-digest) assign_once package_identity_digest "$value" ;;
	manifest-path) assign_once manifest_path "$value" ;;
	manifest-digest) assign_once manifest_digest "$value" ;;
	firmware-elf-path) assign_once firmware_elf_path "$value" ;;
	firmware-elf-digest) assign_once firmware_elf_digest "$value" ;;
	executable-image-path) assign_once executable_image_path "$value" ;;
	executable-image-digest) assign_once executable_image_digest "$value" ;;
	factory-image-path) assign_once factory_image_path "$value" ;;
	factory-image-digest) assign_once factory_image_digest "$value" ;;
	capture-timeout-seconds) assign_once capture_timeout_seconds "$value" ;;
	wall-clock-timeout-seconds) assign_once wall_clock_timeout_seconds "$value" ;;
	wifi-credentials) assign_once wifi_credentials "$value" ;;
	trusted-origin) assign_once trusted_origin "$value" ;;
	*) fail unknown_argument ;;
	esac
done

case "$operation" in
exact-package-flash | passive-serial-observation | read-only-system-info | read-only-websocket | read-only-retained-facts | typed-recovery | cleanup) ;;
*) fail invalid_operation ;;
esac
[[ "$board" == 205 ]] || fail wrong_board
if [[ "$operation" == cleanup && "$port" == unavailable ]]; then
	:
elif [[ "$port" == /* || "$port" =~ ^COM[0-9]+$ ]]; then
	:
else
	fail port_invalid
fi
[[ "$attempt_child" == /* && -d "$attempt_child" && ! -L "$attempt_child" ]] ||
	fail attempt_child_invalid
[[ "$package_identity_digest" =~ ^[0-9a-f]{64}$ &&
	"$manifest_digest" =~ ^[0-9a-f]{64}$ &&
	"$firmware_elf_digest" =~ ^[0-9a-f]{64}$ &&
	"$executable_image_digest" =~ ^[0-9a-f]{64}$ &&
	"$factory_image_digest" =~ ^[0-9a-f]{64}$ ]] ||
	fail identity_invalid
[[ "$capture_timeout_seconds" =~ ^[0-9]+$ ]] || fail capture_timeout_invalid
((capture_timeout_seconds >= 360)) || fail capture_timeout_invalid
[[ "$wall_clock_timeout_seconds" == 420 ]] || fail wall_clock_timeout_invalid

resolve_runfile() {
	local relative="$1"
	local candidate
	for candidate in \
		"${RUNFILES_DIR:-}/_main/$relative" \
		"$workspace_dir/bazel-bin/$relative" \
		"$workspace_dir/$relative"; do
		if [[ -x "$candidate" || -f "$candidate" ]]; then
			printf '%s\n' "$candidate"
			return
		fi
	done
	return 1
}

readonly flash_tool="$(resolve_runfile tools/flash/flash)" || fail flash_tool_unavailable
readonly http_reader="$(resolve_runfile scripts/phase35_http_boundary_read)" ||
	fail http_reader_unavailable
readonly websocket_reader="$(resolve_runfile scripts/phase17-websocket-capture.mjs)" ||
	fail websocket_reader_unavailable
readonly report="$(resolve_runfile tools/parity/report)" || fail report_unavailable

case "$operation" in
exact-package-flash | typed-recovery)
	[[ -f "$manifest_path" && -f "$factory_image_path" ]] || fail package_input_missing
	arguments=(
		flash
		--board 205
		--port "$port"
		--manifest "$manifest_path"
		--image "$factory_image_path"
		--redact-evidence
		--evidence-dir "$attempt_child/${operation}"
	)
	if [[ "$operation" == exact-package-flash ]]; then
		[[ -n "$wifi_credentials" ]] || fail wifi_credentials_missing
		arguments+=(--wifi-credentials "$wifi_credentials")
	fi
	exec "$flash_tool" "${arguments[@]}"
	;;
passive-serial-observation)
	monitor_dir="$attempt_child/passive-serial-observation"
	"$flash_tool" monitor \
		--board 205 \
		--port "$port" \
		--capture-timeout-seconds "$capture_timeout_seconds" \
		--evidence-mode dual \
		--redact-evidence \
		--evidence-dir "$monitor_dir"
	classifier_log="$monitor_dir/monitor.classifier-input.log"
	if [[ ! -f "$classifier_log" ]]; then
		classifier_log="$monitor_dir/flash-monitor.classifier-input.log"
	fi
	[[ -f "$classifier_log" ]] || fail passive_serial_log_missing
	origins=()
	while IFS= read -r origin; do
		origins+=("$origin")
	done < <(
		rg -o 'device_url=https?://[^[:space:]]+' "$classifier_log" |
			sed 's/^device_url=//' |
			LC_ALL=C sort -u
	)
	[[ "${#origins[@]}" == 1 ]] || fail trusted_origin_invalid
	printf 'trusted_origin=%s\n' "${origins[0]}"
	;;
read-only-system-info)
	[[ "$trusted_origin" =~ ^https?://[^/?#]+/?$ ]] || fail trusted_origin_invalid
	exec "$http_reader" \
		label=immediate \
		protected-root="$attempt_child" \
		url="$trusted_origin"
	;;
read-only-websocket)
	[[ "$trusted_origin" =~ ^https?://[^/?#]+/?$ ]] || fail trusted_origin_invalid
	exec node "$websocket_reader" \
		--device-url "$trusted_origin" \
		--path /api/ws/live \
		--out "$attempt_child/websocket.json" \
		--duration-ms 5000 \
		--max-frames 3
	;;
read-only-retained-facts)
	[[ "$trusted_origin" =~ ^https?://[^/?#]+/?$ ]] || fail trusted_origin_invalid
	exec "$report" phase36-assemble-hardware-capture \
		--attempt-child "$attempt_child" \
		--manifest "$manifest_path" \
		--manifest-digest "$manifest_digest" \
		--firmware-elf-digest "$firmware_elf_digest" \
		--executable-image-digest "$executable_image_digest" \
		--factory-image-digest "$factory_image_digest" \
		--package-identity-digest "$package_identity_digest"
	;;
cleanup)
	# All effect commands are synchronous; the broker verifies descriptor closure after return.
	:
	;;
esac
