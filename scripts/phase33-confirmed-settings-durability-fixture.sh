#!/usr/bin/env bash
set -euo pipefail

role="$(basename "$0")"
readonly role
readonly state_root="${PHASE33_FAKE_STATE_ROOT:?PHASE33_FAKE_STATE_ROOT is required}"
readonly scenario="${PHASE33_TEST_SCENARIO:-success}"
readonly calls_log="${state_root}/calls.log"
mkdir -p "$state_root"
chmod 700 "$state_root"
touch "$calls_log"
chmod 600 "$calls_log"
printf '%s %s\n' "$role" "$*" >>"$calls_log"

write_classification() {
	local status="$1"
	local category="$2"
	local session="${3:-session-a}"
	local ordinal="${4:-7}"
	jq -cn \
		--arg status "$status" \
		--arg category "$category" \
		--arg session "$session" \
		--argjson boot_ordinal "$ordinal" \
		--arg device_url 'http://device.invalid' \
		'{status:$status,category:$category,session:$session,boot_ordinal:$boot_ordinal,device_url:$device_url}'
}

run_just() {
	local command="${1:-}"
	shift || true
	case "$command" in
	detect-ultra205)
		case "$scenario" in
		detector_ambiguous)
			printf 'port=/dev/fake-a\nport=/dev/fake-b\n'
			;;
		board_info_failure)
			printf 'failure_category=board_info_failure\n' >&2
			exit 1
			;;
		*)
			printf 'port=/dev/fake-ultra205\n'
			;;
		esac
		;;
	flash-monitor)
		[[ "$scenario" != "failed_flash" ]] || exit 1
		local evidence_dir=""
		local argument
		for argument in "$@"; do
			case "$argument" in
			evidence-dir=*) evidence_dir="${argument#evidence-dir=}" ;;
			esac
		done
		[[ -n "$evidence_dir" ]] || exit 1
		mkdir -p "$evidence_dir"
		printf 'fake setup capture\n' >"${evidence_dir}/flash-monitor.log"
		;;
	*)
		exit 2
		;;
	esac
}

run_classifier() {
	[[ "${1:-}" == "phase33-classify" ]] || exit 2
	shift
	local classification_mode=""
	while (($#)); do
		case "$1" in
		--mode)
			classification_mode="$2"
			shift 2
			;;
		*) shift ;;
		esac
	done

	case "$classification_mode" in
	baseline)
		case "$scenario" in
		zero_origin) write_classification failed runtime_origin_missing ;;
		multiple_origin) write_classification failed runtime_origin_multiple ;;
		*) write_classification passed none ;;
		esac
		;;
	delivery)
		if [[ "$scenario" == "timeout" ]]; then
			write_classification failed proof_timeout
		else
			write_classification passed none
		fi
		;;
	post-restart)
		case "$scenario" in
		unchanged_session) write_classification failed post_restart_session_unchanged ;;
		multiple_session | extra_reset) write_classification failed post_restart_multiple_sessions ;;
		ordinal_n_plus_two) write_classification failed post_restart_ordinal_nonmonotonic ;;
		wrong_reset) write_classification failed post_restart_reset_reason_wrong ;;
		wrong_session_origin) write_classification failed runtime_origin_wrong_session ;;
		response_reversal) write_classification failed response_before_effect_unproved ;;
		*) write_classification passed none session-b 8 ;;
		esac
		;;
	*)
		exit 2
		;;
	esac
}

run_identity() {
	local count_file="${state_root}/identity-count"
	local count=0
	[[ ! -f "$count_file" ]] || count="$(<"$count_file")"
	count=$((count + 1))
	printf '%s\n' "$count" >"$count_file"
	if [[ "$scenario" == "identity_change" && "$count" -ge 2 ]]; then
		printf 'physical-identity-b\n'
		return
	fi
	printf 'physical-identity-a\n'
}

parse_curl_args() {
	request_method="GET"
	output_path=""
	data_path=""
	request_url=""
	while (($#)); do
		case "$1" in
		--request)
			request_method="$2"
			shift 2
			;;
		--output)
			output_path="$2"
			shift 2
			;;
		--data-binary)
			data_path="${2#@}"
			shift 2
			;;
		--max-time | --write-out | --header)
			shift 2
			;;
		--silent | --show-error)
			shift
			;;
		*)
			request_url="$1"
			shift
			;;
		esac
	done
}

write_http_hostname() {
	local hostname="$1"
	[[ -n "$output_path" && "$output_path" != "/dev/null" ]] || return 0
	mkdir -p "$(dirname "$output_path")"
	jq -cn --arg hostname "$hostname" '{hostname:$hostname}' >"$output_path"
}

run_curl() {
	parse_curl_args "$@"
	local hostname_file="${state_root}/hostname"
	[[ -f "$hostname_file" ]] || printf 'factory-host\n' >"$hostname_file"

	if [[ "$request_method" == "PATCH" ]]; then
		local hostname
		hostname="$(jq -er '.hostname' "$data_path")"
		if [[ "$hostname" == "factory-host" ]]; then
			printf 'curl PATCH restore\n' >>"$calls_log"
			[[ "$scenario" != "restore_failure" ]] || exit 7
		else
			printf 'curl PATCH proof\n' >>"$calls_log"
		fi
		printf '%s\n' "$hostname" >"$hostname_file"
		: >"$output_path"
		printf '200'
		return
	fi

	if [[ "$request_method" == "POST" && "$request_url" == */api/system/restart ]]; then
		printf 'curl POST restart\n' >>"$calls_log"
		printf '{}\n' >"$output_path"
		touch "${state_root}/restart-requested"
		printf '200'
		return
	fi

	if [[ -f "${state_root}/restart-requested" && "$output_path" == "/dev/null" ]]; then
		exit 7
	fi

	local hostname
	hostname="$(<"$hostname_file")"
	case "$(basename "$output_path")" in
	immediate.json)
		[[ "$scenario" != "immediate_missing" ]] || exit 7
		[[ "$scenario" != "immediate_mismatch" ]] || hostname="mismatched-host"
		;;
	post-reboot.json)
		[[ "$scenario" != "post_missing" ]] || exit 7
		[[ "$scenario" != "post_mismatch" ]] || hostname="mismatched-host"
		;;
	esac
	write_http_hostname "$hostname"
	printf '200'
}

monitor_cleanup() {
	if [[ "$scenario" != "process_leak" ]]; then
		: >"${PHASE13_MONITOR_GROUP_STATE_FILE:?monitor state file is required}"
	fi
	exit 143
}

run_monitor() {
	local out=""
	local raw_out=""
	while (($#)); do
		case "$1" in
		--out)
			out="$2"
			shift 2
			;;
		--raw-out)
			raw_out="$2"
			shift 2
			;;
		--port | --seconds | --reader)
			shift 2
			;;
		--no-reset)
			shift
			;;
		*) exit 2 ;;
		esac
	done
	[[ -n "$out" && -n "$raw_out" ]] || exit 2
	trap monitor_cleanup INT TERM
	printf '%s\n' "$$" >"${PHASE13_MONITOR_GROUP_STATE_FILE:?monitor state file is required}"
	printf 'ready\n' >"${PHASE13_MONITOR_ACTIVE_READY_FILE:?monitor ready file is required}"
	printf 'baseline-session-bytes\n' >>"$raw_out"

	for _ in $(seq 1 5000); do
		[[ ! -f "${state_root}/restart-requested" ]] || break
		sleep 0.01
	done
	[[ -f "${state_root}/restart-requested" ]] || exit 1
	printf 'post-restart-session-bytes\n' >>"$raw_out"
	if [[ "$scenario" != "holder_leak" ]]; then
		printf 'serial_trace_post_readiness=ready\n' >>"$out"
	fi
	printf 'serial_trace_active_owner_verified=true\n' >>"$out"
	if [[ "$scenario" != "process_leak" ]]; then
		: >"$PHASE13_MONITOR_GROUP_STATE_FILE"
	fi
}

run_checkpoint() {
	local checkpoint="${1:-}"
	local output="${2:-}"
	case "${checkpoint}:${scenario}" in
	after_hostname_patch:cancel_after_patch)
		kill -TERM "$PPID"
		;;
	after_hostname_patch:errexit_after_patch)
		exit 73
		;;
	before_shareable_validation:sensitive_output)
		printf 'raw_origin=http://192.0.2.1 password=fixture-only\n' >>"$output"
		;;
	esac
}

case "$role" in
just) run_just "$@" ;;
classifier) run_classifier "$@" ;;
identity) run_identity "$@" ;;
curl) run_curl "$@" ;;
monitor) run_monitor "$@" ;;
checkpoint) run_checkpoint "$@" ;;
*) exit 2 ;;
esac
