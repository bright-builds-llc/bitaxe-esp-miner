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
result_path=""

fail() {
	printf 'category=%s\n' "$1" >&2
	exit 2
}

file_mode() {
	stat -f '%Lp' "$1" 2>/dev/null || stat -c '%a' "$1"
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
	result-path) assign_once result_path "$value" ;;
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
[[ "$result_path" == "$attempt_child/effect-result-${operation}.json" &&
	! -e "$result_path" ]] || fail result_path_invalid

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

json_operation() {
	printf '%s\n' "${operation//-/_}"
}

write_effect_result() {
	local status="$1"
	local failure="${2:-}"
	[[ ! -e "$result_path" ]] || fail result_path_exists
	umask 077
	jq -cn \
		--arg operation "$(json_operation)" \
		--arg status "$status" \
		--arg failure "$failure" \
		--arg package_identity_digest "$package_identity_digest" \
		--arg factory_image_digest "$factory_image_digest" \
		'{schema_version:"phase36-effect-result-v1",operation:$operation,status:$status,
		failure:(if $failure == "" then null else $failure end),
		package_identity_digest:$package_identity_digest,
		factory_image_digest:$factory_image_digest}' >"$result_path"
	chmod 600 "$result_path"
}

run_flash_operation() {
	local child_status=0
	if PHASE36_EFFECT_RESULT_PATH="$result_path" \
		PHASE36_EFFECT_OPERATION="$(json_operation)" \
		PHASE36_EFFECT_PACKAGE_IDENTITY_DIGEST="$package_identity_digest" \
		PHASE36_EFFECT_FACTORY_IMAGE_DIGEST="$factory_image_digest" \
		"$flash_tool" "${arguments[@]}"; then
		:
	else
		child_status=$?
	fi
	if [[ -e "$result_path" ]]; then
		[[ -f "$result_path" && ! -L "$result_path" &&
			"$(file_mode "$result_path")" == 600 ]] || fail effect_result_invalid
	fi
	return "$child_status"
}

case "$operation" in
exact-package-flash | typed-recovery)
	[[ -f "$manifest_path" && -f "$factory_image_path" ]] || fail package_input_missing
	arguments=(
		flash
		--board 205
		--port "$port"
		--manifest "$manifest_path"
		--redact-evidence
		--evidence-dir "$attempt_child/${operation}"
	)
	if [[ "$operation" == exact-package-flash ]]; then
		[[ -n "$wifi_credentials" ]] || fail wifi_credentials_missing
		arguments+=(--wifi-credentials "$wifi_credentials")
	fi
	run_flash_operation
	;;
passive-serial-observation)
	monitor_dir="$attempt_child/passive-serial-observation"
	[[ ! -e "$monitor_dir" ]] || fail passive_serial_root_exists
	mkdir "$monitor_dir"
	chmod 700 "$monitor_dir"
	classifier_log="$monitor_dir/monitor.classifier-input.log"
	diagnostic_log="$monitor_dir/monitor.stderr.log"
	umask 077
	if ! "$flash_tool" monitor \
		--board 205 \
		--port "$port" \
		--capture-timeout-seconds "$capture_timeout_seconds" \
		>"$classifier_log" 2>"$diagnostic_log"; then
		write_effect_result failed_no_device_effect capture_failed
		exit 2
	fi
	[[ -f "$classifier_log" && ! -L "$classifier_log" &&
		"$(file_mode "$classifier_log")" == 600 &&
		-f "$diagnostic_log" && ! -L "$diagnostic_log" &&
		"$(file_mode "$diagnostic_log")" == 600 ]] || fail passive_serial_log_invalid
	origins=()
	while IFS= read -r origin; do
		origins+=("$origin")
	done < <(
		rg -o 'device_url=https?://[^[:space:]]+' "$classifier_log" |
			sed 's/^device_url=//' |
			LC_ALL=C sort -u
	)
	[[ "${#origins[@]}" == 1 ]] || fail trusted_origin_invalid
	write_effect_result completed
	printf 'trusted_origin=%s\n' "${origins[0]}"
	;;
read-only-system-info)
	[[ "$trusted_origin" =~ ^https?://[^/?#]+/?$ ]] || fail trusted_origin_invalid
	if ! "$http_reader" \
		label=immediate \
		protected-root="$attempt_child" \
		url="$trusted_origin"; then
		write_effect_result failed_no_device_effect capture_failed
		exit 2
	fi
	write_effect_result completed
	;;
read-only-websocket)
	[[ "$trusted_origin" =~ ^https?://[^/?#]+/?$ ]] || fail trusted_origin_invalid
	if ! node "$websocket_reader" \
		--device-url "$trusted_origin" \
		--path /api/ws/live \
		--out "$attempt_child/websocket.json" \
		--duration-ms 5000 \
		--max-frames 3; then
		write_effect_result failed_no_device_effect capture_failed
		exit 2
	fi
	write_effect_result completed
	;;
read-only-retained-facts)
	[[ "$trusted_origin" =~ ^https?://[^/?#]+/?$ ]] || fail trusted_origin_invalid
	if ! "$report" phase36-assemble-hardware-capture \
		--attempt-child "$attempt_child" \
		--manifest "$manifest_path" \
		--manifest-digest "$manifest_digest" \
		--firmware-elf-digest "$firmware_elf_digest" \
		--executable-image-digest "$executable_image_digest" \
		--factory-image-digest "$factory_image_digest" \
		--package-identity-digest "$package_identity_digest"; then
		write_effect_result failed_no_device_effect capture_failed
		exit 2
	fi
	write_effect_result completed
	;;
cleanup)
	# All effect commands are synchronous; the broker verifies descriptor closure after return.
	write_effect_result completed
	;;
esac
