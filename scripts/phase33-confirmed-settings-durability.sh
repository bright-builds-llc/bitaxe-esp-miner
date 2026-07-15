#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/serial-session-trace.sh
# shellcheck disable=SC1091
source "${script_dir}/serial-session-trace.sh"
readonly passive_monitor_contract="--chip esp32s3 --before no-reset-no-sync --after no-reset --no-reset --non-interactive"

mode="hardware"
scenario="success"
capture_seconds=360
manifest="bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json"
wifi_credentials=""
shareable_out="docs/evidence/phase-33/hardware-summary.md"
local_root=""
passive_pid=""
device_url=""
original_hostname=""
hostname_changed=0
restore_complete=0
finalizer_ran=0
finalizer_cleanup_failed=0
finalizer_restoration_failed=0
phase33_classifier="${PHASE33_CLASSIFIER:-bazel-bin/tools/parity/report}"
just_command="${PHASE33_JUST_COMMAND:-just}"
curl_command="${PHASE33_CURL_COMMAND:-curl}"
identity_command="${PHASE33_IDENTITY_COMMAND:-}"
passive_monitor_command="${PHASE33_PASSIVE_MONITOR_COMMAND:-${script_dir}/phase13-monitor-capture.sh}"
checkpoint_command="${PHASE33_CHECKPOINT_COMMAND:-}"
poll_interval_seconds="${PHASE33_POLL_INTERVAL_SECONDS:-0.25}"

usage() {
	printf 'usage: %s [--mode hardware|simulate] [--scenario NAME] [--capture-seconds N] [--manifest PATH] [--wifi-credentials PATH] [--shareable-out PATH] [--local-root PATH]\n' "$(basename "$0")" >&2
}

while (($#)); do
	case "$1" in
	--mode | --scenario | --capture-seconds | --manifest | --wifi-credentials | --shareable-out | --local-root)
		[[ $# -ge 2 ]] || {
			usage
			exit 2
		}
		case "$1" in
		--mode) mode="$2" ;;
		--scenario) scenario="$2" ;;
		--capture-seconds) capture_seconds="$2" ;;
		--manifest) manifest="$2" ;;
		--wifi-credentials) wifi_credentials="$2" ;;
		--shareable-out) shareable_out="$2" ;;
		--local-root) local_root="$2" ;;
		esac
		shift 2
		;;
	-h | --help)
		usage
		exit 0
		;;
	*)
		usage
		exit 2
		;;
	esac
done

[[ "$mode" == "hardware" || "$mode" == "simulate" ]] || {
	printf 'failure_category=invalid_mode\n' >&2
	exit 2
}
[[ "$capture_seconds" =~ ^[0-9]+$ ]] || {
	printf 'failure_category=invalid_capture_timeout\n' >&2
	exit 2
}

hash_text() {
	printf '%s' "$1" | shasum -a 256 | awk '{print $1}'
}

run_checkpoint() {
	[[ -n "$checkpoint_command" ]] || return 0
	"$checkpoint_command" "$@"
}

normalize_absolute_path() {
	local path="$1"
	local normalized=""
	local component
	local -a components
	IFS='/' read -r -a components <<<"$path"
	for component in "${components[@]}"; do
		case "$component" in
		'' | .) continue ;;
		..)
			return 1
			;;
		esac
		normalized="${normalized}/${component}"
	done
	[[ -n "$normalized" ]] || normalized="/"
	printf '%s\n' "$normalized"
}

path_has_symlink_component() {
	local path="$1"
	local current=""
	local component
	local -a components
	IFS='/' read -r -a components <<<"$path"
	for component in "${components[@]}"; do
		[[ -n "$component" ]] || continue
		current="${current}/${component}"
		[[ ! -L "$current" ]] || return 0
	done
	return 1
}

canonicalize_local_root() {
	local requested="$1"
	local repo_root="$2"
	local absolute
	if [[ "$requested" == /* ]]; then
		absolute="$requested"
	else
		absolute="${repo_root}/${requested}"
	fi
	absolute="$(normalize_absolute_path "$absolute")" || return 1
	if [[ "$absolute" == "$repo_root" || "$absolute" == "${repo_root}/"* ]]; then
		path_has_symlink_component "$absolute" && return 1
	elif [[ -L "$absolute" ]]; then
		return 1
	fi

	local cursor="$absolute"
	local suffix=""
	while [[ ! -e "$cursor" && ! -L "$cursor" ]]; do
		suffix="/$(basename "$cursor")${suffix}"
		cursor="$(dirname "$cursor")"
	done
	[[ -d "$cursor" && ! -L "$cursor" ]] || return 1
	local canonical_existing
	canonical_existing="$(realpath "$cursor")" || return 1
	printf '%s%s\n' "${canonical_existing%/}" "$suffix"
}

repo_local_root_is_untracked_and_ignored() {
	local repo_root="$1"
	local relative="$2"
	local tracked_entries
	tracked_entries="$(git -C "$repo_root" ls-files --cached --stage -- ":(literal)${relative}")" || return 1
	[[ -z "$tracked_entries" ]] || return 1
	git -C "$repo_root" check-ignore -q -- "$relative"
}

prepare_local_root() {
	local requested="$1"
	local repo_root
	repo_root="$(git rev-parse --show-toplevel)" || return 1
	repo_root="$(realpath "$repo_root")" || return 1

	local canonical
	canonical="$(canonicalize_local_root "$requested" "$repo_root")" || return 1
	if [[ "$canonical" == "$repo_root" ]]; then
		return 1
	fi
	if [[ "$canonical" == "${repo_root}/"* ]]; then
		local relative="${canonical#"${repo_root}/"}"
		repo_local_root_is_untracked_and_ignored "$repo_root" "$relative" || return 1
	elif [[ -e "$canonical" ]]; then
		[[ "$(stat -f '%Lp' "$canonical")" == "700" ]] || return 1
	fi

	mkdir -p "$canonical" || return 1
	chmod 700 "$canonical" || return 1
	[[ "$(realpath "$canonical")" == "$canonical" ]] || return 1
	local_root="$canonical"
}

physical_identity() {
	local port="$1"
	if [[ -n "$identity_command" ]]; then
		"$identity_command" "$port"
		return
	fi
	serial_session_usb_physical_identity "$port"
}

detector_failure_category() {
	local maybe_category
	maybe_category="$(sed -n 's/^failure_category=\([a-z0-9_]*\)$/\1/p' "$detector_log" | tail -1)"
	if [[ -n "$maybe_category" ]]; then
		printf '%s\n' "$maybe_category"
		return
	fi
	printf 'detector_failed\n'
}

if [[ "$mode" == "simulate" ]]; then
	[[ "${PHASE33_ALLOW_TEST_MODE:-}" == "1" ]] || {
		printf 'failure_category=simulation_not_authorized\n' >&2
		exit 2
	}
	[[ -n "${PHASE33_FAKE_STATE_ROOT:-}" ]] || {
		printf 'failure_category=simulation_fixture_missing\n' >&2
		exit 2
	}
	export PHASE33_TEST_SCENARIO="$scenario"
fi

umask 077
if [[ -z "$local_root" ]]; then
	local_root="scratch/phase33-settings-durability/$(date -u '+%Y%m%dT%H%M%SZ')-$$"
fi
prepare_local_root "$local_root" || {
	printf 'failure_category=local_raw_root_unsafe\n' >&2
	exit 2
}

[[ "$mode" == "simulate" ]] || ((capture_seconds >= 360)) || {
	printf 'failure_category=capture_timeout_too_short\n' >&2
	exit 2
}
[[ -f "$manifest" ]] || {
	printf 'failure_category=required_package_missing\n' >&2
	exit 1
}
if [[ "$mode" == "simulate" ]]; then
	current_source_commit="${PHASE33_TEST_SOURCE_COMMIT:-test-source-commit}"
	current_reference_commit="${PHASE33_TEST_REFERENCE_COMMIT:-test-reference-commit}"
else
	current_source_commit="$(git rev-parse HEAD)"
	current_reference_commit="$(git -C reference/esp-miner rev-parse HEAD)"
fi
manifest_source_commit="$(jq -er '.source_commit | select(type == "string")' "$manifest")" || {
	printf 'failure_category=package_source_commit_missing\n' >&2
	exit 1
}
manifest_reference_commit="$(jq -er '.reference_commit | select(type == "string")' "$manifest")" || {
	printf 'failure_category=package_reference_commit_missing\n' >&2
	exit 1
}
[[ "$manifest_source_commit" == "$current_source_commit" ]] || {
	printf 'failure_category=package_source_commit_stale\n' >&2
	exit 1
}
[[ "$manifest_reference_commit" == "$current_reference_commit" ]] || {
	printf 'failure_category=package_reference_commit_stale\n' >&2
	exit 1
}
if [[ -n "$wifi_credentials" && ! -f "$wifi_credentials" ]]; then
	printf 'failure_category=wifi_credentials_path_missing\n' >&2
	exit 1
fi

detector_log="${local_root}/detector.log"
flash_dir="${local_root}/flash"
passive_log="${local_root}/passive.log"
passive_raw="${local_root}/passive.raw"
passive_ready="${local_root}/passive.ready"
passive_state="${local_root}/passive.state"
http_dir="${local_root}/http"
mkdir -p "$flash_dir" "$http_dir"
chmod 700 "$flash_dir" "$http_dir"
: >"$detector_log"
: >"$passive_log"
: >"$passive_raw"
: >"$passive_state"
chmod 600 "$detector_log" "$passive_log" "$passive_raw" "$passive_state"

cleanup_passive() {
	[[ -n "$passive_pid" ]] || return 0
	if kill -0 "$passive_pid" >/dev/null 2>&1; then
		kill -TERM "$passive_pid"
	fi
	set +e
	wait "$passive_pid"
	local status=$?
	set -e
	passive_pid=""
	if ((status != 0 && status != 130 && status != 143)); then
		return 1
	fi
}

classify_trace() {
	local output="$1"
	shift
	"$phase33_classifier" phase33-classify "$@" >"$output"
	chmod 600 "$output"
}

http_json_field() {
	local origin="$1"
	local field="$2"
	local body="$3"
	local code
	set +e
	code="$("$curl_command" --silent --show-error --max-time 10 --output "$body" --write-out '%{http_code}' "${origin}/api/system/info")"
	local status=$?
	set -e
	((status == 0)) || return 1
	[[ "$code" == "200" ]] || return 1
	jq -er --arg field "$field" '.[$field] | select(type == "string")' "$body"
}

patch_hostname() {
	local origin="$1"
	local hostname="$2"
	local label="$3"
	local payload="${http_dir}/${label}-request.json"
	local body="${http_dir}/${label}-response.txt"
	jq -cn --arg hostname "$hostname" '{hostname:$hostname}' >"$payload"
	chmod 600 "$payload"
	local code
	set +e
	code="$("$curl_command" --silent --show-error --max-time 15 --request PATCH --header 'Content-Type: application/json' --data-binary "@${payload}" --output "$body" --write-out '%{http_code}' "${origin}/api/system")"
	local status=$?
	set -e
	((status == 0)) || return 1
	[[ "$code" == "200" && ! -s "$body" ]]
}

restore_hostname() {
	((hostname_changed == 1)) || return 0
	[[ -n "$device_url" && -n "$original_hostname" ]] || return 1
	patch_hostname "$device_url" "$original_hostname" restore || return 1
	local restored
	restored="$(http_json_field "$device_url" hostname "${http_dir}/restore-readback.json")" || return 1
	[[ "$restored" == "$original_hostname" ]] || return 1
	restore_complete=1
	hostname_changed=0
}

finalize_once() {
	((finalizer_ran == 0)) || return 0
	finalizer_ran=1
	set +e
	if ! cleanup_passive; then
		finalizer_cleanup_failed=1
		printf 'cleanup_category=monitor_process_cleanup_failed\n' >&2
	fi
	if ((hostname_changed == 1)); then
		if restore_hostname; then
			printf 'restoration_category=confirmed_restored\n' >&2
		else
			finalizer_restoration_failed=1
			printf 'restoration_category=restoration_failed\n' >&2
		fi
	fi
	set -e
	return 0
}

on_exit() {
	local status=$?
	trap - EXIT
	finalize_once
	exit "$status"
}

on_signal() {
	local signal="$1"
	trap - EXIT INT TERM
	finalize_once
	trap - "$signal"
	kill -s "$signal" "$$"
}

trap 'on_exit' EXIT
trap 'on_signal INT' INT
trap 'on_signal TERM' TERM

fail_proof() {
	local category="$1"
	finalize_once
	((finalizer_cleanup_failed == 0)) || category="monitor_process_cleanup_failed"
	((finalizer_restoration_failed == 0)) || category="restoration_failed"
	printf 'failure_category=%s\n' "$category" >&2
	exit 1
}

if [[ "$phase33_classifier" == "bazel-bin/tools/parity/report" ]]; then
	bazel build //tools/parity:report >/dev/null || fail_proof phase33_classifier_build_failed
fi
[[ -x "$phase33_classifier" ]] || fail_proof phase33_classifier_unavailable

printf '[phase33] detector preflight starting; this is the only detector invocation\n'
set +e
"$just_command" detect-ultra205 >"$detector_log" 2>&1
detector_status=$?
set -e
((detector_status == 0)) || fail_proof "$(detector_failure_category)"
port_lines="$(rg '^port=' "$detector_log")"
port_count="$(printf '%s\n' "$port_lines" | sed '/^$/d' | wc -l | tr -d ' ')"
[[ "$port_count" == "1" ]] || fail_proof detector_ambiguous
port="${port_lines#port=}"
[[ -n "$port" ]] || fail_proof detector_ambiguous
physical_identity_before="$(physical_identity "$port")" || fail_proof physical_identity_unavailable

flash_args=(board=205 "port=${port}" "manifest=${manifest}" "evidence-dir=${flash_dir}" "capture-timeout-seconds=${capture_seconds}")
if [[ -n "$wifi_credentials" ]]; then
	flash_args+=("wifi-credentials=${wifi_credentials}")
fi
printf '[phase33] required Phase 33 package flash and %ss setup capture starting\n' "$capture_seconds"
set +e
"$just_command" flash-monitor "${flash_args[@]}" >"${local_root}/flash-command.log" 2>&1
flash_status=$?
set -e
((flash_status == 0)) || fail_proof required_package_flash_failed
monitor_log="${flash_dir}/flash-monitor.log"
[[ -s "$monitor_log" ]] || fail_proof flash_monitor_log_missing
baseline_json="${local_root}/baseline-classification.json"
classify_trace "$baseline_json" --trace "$monitor_log" --mode baseline || fail_proof baseline_classifier_failed
[[ "$(jq -er '.status' "$baseline_json")" == "passed" ]] || fail_proof "$(jq -er '.category' "$baseline_json")"
baseline_session="$(jq -er '.session' "$baseline_json")"
baseline_ordinal="$(jq -er '.boot_ordinal' "$baseline_json")"
device_url="$(jq -er '.device_url' "$baseline_json")"
printf '[phase33] setup capture complete; starting confirmed PATCH and passive proof\n'

original_hostname="$(http_json_field "$device_url" hostname "${http_dir}/original.json")" || fail_proof original_hostname_unavailable
source_short="${current_source_commit:0:6}"
test_hostname="phase33-$(date -u '+%H%M%S')-${source_short}"
patch_hostname "$device_url" "$test_hostname" proof || fail_proof hostname_patch_failed
hostname_changed=1
run_checkpoint after_hostname_patch "$shareable_out"
immediate_hostname="$(http_json_field "$device_url" hostname "${http_dir}/immediate.json")" || fail_proof immediate_readback_missing
[[ "$immediate_hostname" == "$test_hostname" ]] || fail_proof immediate_readback_mismatch

physical_identity_proof="$(physical_identity "$port")" || fail_proof physical_identity_unavailable
[[ "$physical_identity_proof" == "$physical_identity_before" ]] || fail_proof physical_identity_changed
rm -f "$passive_ready"
printf 'required_contract=%s\n' "$passive_monitor_contract" >>"$passive_log"
PHASE13_MONITOR_ACTIVE_READY_FILE="$passive_ready" \
	PHASE13_MONITOR_GROUP_STATE_FILE="$passive_state" \
	SERIAL_SESSION_TRACE_ROOT="$local_root" \
	bash "$passive_monitor_command" \
	--port "$port" --out "$passive_log" --raw-out "$passive_raw" \
	--seconds "$capture_seconds" --reader espflash --no-reset &
passive_pid=$!
for _ in $(seq 1 80); do
	[[ -s "$passive_ready" ]] && break
	kill -0 "$passive_pid" >/dev/null 2>&1 || fail_proof passive_monitor_start_failed
	sleep "$poll_interval_seconds"
done
[[ -s "$passive_ready" ]] || fail_proof passive_monitor_not_ready
delivery_offset="$(wc -c <"$passive_raw" | tr -d ' ')"
delivery_json="${local_root}/delivery-classification.json"
delivery_proved=0
for _ in $(seq 1 160); do
	classify_trace "$delivery_json" --trace "$passive_raw" --mode delivery --start-byte "$delivery_offset" --expected-session "$baseline_session" --expected-ordinal "$baseline_ordinal" || fail_proof passive_delivery_classifier_failed
	if [[ "$(jq -er '.status' "$delivery_json")" == "passed" ]]; then
		delivery_proved=1
		break
	fi
	kill -0 "$passive_pid" >/dev/null 2>&1 || fail_proof passive_monitor_exited_before_delivery
	sleep "$poll_interval_seconds"
done
if ((delivery_proved == 0)); then
	delivery_category="$(jq -er '.category' "$delivery_json")" || delivery_category="passive_byte_delivery_unproved"
	fail_proof "$delivery_category"
fi

restart_body="${http_dir}/restart-response.json"
set +e
restart_code="$("$curl_command" --silent --show-error --max-time 15 --request POST --output "$restart_body" --write-out '%{http_code}' "${device_url}/api/system/restart")"
restart_status=$?
set -e
((restart_status == 0)) || fail_proof restart_response_missing
[[ "$restart_code" == "200" ]] || fail_proof restart_response_invalid

service_lost=0
for _ in $(seq 1 80); do
	if ! "$curl_command" --silent --max-time 1 --output /dev/null "${device_url}/api/system/info"; then
		service_lost=1
		break
	fi
	sleep "$poll_interval_seconds"
done
((service_lost == 1)) || fail_proof service_loss_unproved
proof_offset="$(wc -c <"$passive_raw" | tr -d ' ')"

printf '[phase33] application restart response received; passive %ss capture is in progress\n' "$capture_seconds"
set +e
wait "$passive_pid"
passive_status=$?
set -e
passive_pid=""
((passive_status == 0)) || fail_proof passive_capture_failed
rg -q '^serial_trace_post_readiness=ready$' "$passive_log" || fail_proof serial_holder_cleanup_failed
rg -q '^serial_trace_active_owner_verified=true$' "$passive_log" || fail_proof serial_ownership_unproved
[[ ! -s "$passive_state" ]] || fail_proof monitor_process_cleanup_failed

post_json="${local_root}/post-restart-classification.json"
classify_trace "$post_json" --trace "$passive_raw" --mode post-restart --start-byte "$proof_offset" --expected-session "$baseline_session" --expected-ordinal "$baseline_ordinal" || fail_proof post_restart_classifier_failed
[[ "$(jq -er '.status' "$post_json")" == "passed" ]] || fail_proof "$(jq -er '.category' "$post_json")"
device_url="$(jq -er '.device_url' "$post_json")"
post_ordinal="$(jq -er '.boot_ordinal' "$post_json")"
physical_identity_after="$(physical_identity "$port")" || fail_proof physical_identity_unavailable
[[ "$physical_identity_after" == "$physical_identity_before" ]] || fail_proof physical_identity_changed

post_hostname="$(http_json_field "$device_url" hostname "${http_dir}/post-reboot.json")" || fail_proof post_reboot_readback_missing
[[ "$post_hostname" == "$test_hostname" ]] || fail_proof post_reboot_readback_mismatch
restore_hostname || fail_proof restoration_failed

source_commit="$current_source_commit"
reference_commit="$current_reference_commit"
package_digest="$(shasum -a 256 "$manifest" | awk '{print $1}')"
trace_digest="$(shasum -a 256 "$passive_raw" | awk '{print $1}')"
test_digest="$(hash_text "$test_hostname")"
original_digest="$(hash_text "$original_hostname")"
evidence_title="Phase 33 Confirmed Settings Durability Evidence"
evidence_conclusion="passed durability evidence only; Phase 35 admission and parity promotion remain unclaimed"
if [[ "$mode" == "simulate" ]]; then
	evidence_title="Phase 33 Confirmed Settings Durability Simulation"
	evidence_conclusion="simulation-only; no hardware or parity claim"
fi
mkdir -p "$(dirname "$shareable_out")"
cat >"$shareable_out" <<EOF
# ${evidence_title}

- status: passed
- board_category: Ultra 205
- source_commit_sha256_input: ${source_commit}
- reference_commit_sha256_input: ${reference_commit}
- package_manifest_sha256: ${package_digest}
- command_categories: detector, package-flash-monitor, settings-patch, system-info-readback, passive-monitor, application-restart, settings-restore
- setup_capture_seconds: ${capture_seconds}
- passive_capture_seconds: ${capture_seconds}
- detector_count: 1
- package_flash_required: true
- package_flash_complete: true
- immediate_hostname_digest_sha256: ${test_digest}
- post_reboot_hostname_digest_sha256: ${test_digest}
- restored_hostname_digest_sha256: ${original_digest}
- immediate_post_reboot_match: true
- application_restart_count: 1
- baseline_boot_ordinal: ${baseline_ordinal}
- post_restart_boot_ordinal: ${post_ordinal}
- passive_byte_delivery_before_post: true
- post_restart_reset_reason: software_cpu
- post_restart_origin_binding: unique
- response_before_effect: true
- same_physical_identity: true
- passive_monitor_contract_complete: true
- process_cleanup_complete: true
- serial_holder_cleanup_complete: true
- restoration_complete: ${restore_complete}
- protected_trace_sha256: ${trace_digest}
- conclusion: ${evidence_conclusion}
EOF
chmod 644 "$shareable_out"

run_checkpoint before_shareable_validation "$shareable_out"
if rg -n -i 'https?://|([0-9]{1,3}\.){3}[0-9]{1,3}|([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}|/dev/|ssid|password|credential|worker|pool[_ -]?(url|user|password)|device_url|nvs[_ -]?secret|api[_ -]?(token|key)' "$shareable_out"; then
	printf 'failure_category=sensitive_output_detected\n' >&2
	exit 1
fi
printf '[phase33] proof passed; redacted evidence written to %s\n' "$shareable_out"
