#!/usr/bin/env bash
# Detector-gated Phase 28.1.1 accepted-state/lifecycle evidence wrapper.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
mode=""
attempt=""
duration_seconds="360"
port=""
wifi_credentials=""
pool_credentials=""
evidence_out=""
board="205"

die() {
	printf 'accepted_state_diagnostic_error: %s\n' "$1" >&2
	exit 1
}

usage() {
	printf 'usage: %s --mode blocked|hardware --attempt accepted-state|lifecycle --duration-seconds N --port PATH --wifi-credentials PATH --pool-credentials PATH --evidence-out PATH\n' "$(basename "$0")"
}

while (($#)); do
	case "$1" in
	--mode | --attempt | --duration-seconds | --port | --wifi-credentials | --pool-credentials | --evidence-out)
		[[ $# -ge 2 ]] || die "missing option value"
		name="${1#--}"
		case "$name" in
		mode) mode="$2" ;;
		attempt) attempt="$2" ;;
		duration-seconds) duration_seconds="$2" ;;
		port) port="$2" ;;
		wifi-credentials) wifi_credentials="$2" ;;
		pool-credentials) pool_credentials="$2" ;;
		evidence-out) evidence_out="$2" ;;
		esac
		shift 2
		;;
	-h | --help)
		usage
		exit 0
		;;
	*) die "unknown argument" ;;
	esac
done

[[ "$mode" == "blocked" || "$mode" == "hardware" ]] || die "invalid --mode"
[[ "$attempt" == "accepted-state" || "$attempt" == "lifecycle" ]] || die "invalid --attempt"
[[ "$duration_seconds" =~ ^[0-9]+$ ]] || die "--duration-seconds must be an integer"
((duration_seconds >= 360)) || die "--duration-seconds must be at least 360"
[[ -n "$evidence_out" ]] || die "--evidence-out is required"

run_root="${PHASE28_ACCEPTED_STATE_RUN_ROOT:-$repo_root/scratch/phase28.1.1-accepted-state}"
run_dir="$run_root/$(date -u +%Y%m%dT%H%M%SZ)-${attempt}"
mkdir -p "$run_dir" "$(dirname "$evidence_out")"

write_unavailable_markers() {
	local stage
	for stage in post_enumerate post_mining_ready post_max_baud post_mask_reload post_first_work; do
		printf 'accepted_state_snapshot stage=%s observation=unavailable chip_count_class=unavailable readable_responses=0 error_counter_active=false domain_counter_active=false total_counter_active=false power_delta_class=unavailable result_correlated=false submit_observed=false redacted=true\n' "$stage"
	done
}

normalize_markers() {
	local source_log="$1"
	local normalized_log="$2"
	if [[ -f "$source_log" ]] && rg 'accepted_state_snapshot' "$source_log" >"$normalized_log"; then
		return
	fi
	write_unavailable_markers >"$normalized_log"
}

run_detector() {
	if [[ -n "${PHASE28_ACCEPTED_STATE_DETECT_BIN:-}" ]]; then
		"$PHASE28_ACCEPTED_STATE_DETECT_BIN"
	else
		just detect-ultra205
	fi
}

run_verify_reference() {
	if [[ -n "${PHASE28_ACCEPTED_STATE_VERIFY_BIN:-}" ]]; then
		"$PHASE28_ACCEPTED_STATE_VERIFY_BIN"
	else
		just verify-reference
	fi
}

run_package() {
	local -a args=()
	if [[ "$attempt" == "accepted-state" ]]; then
		args+=(--investigation accepted_state_snapshot)
	fi
	if [[ -n "${PHASE28_ACCEPTED_STATE_PACKAGE_BIN:-}" ]]; then
		if ((${#args[@]})); then
			"$PHASE28_ACCEPTED_STATE_PACKAGE_BIN" "${args[@]}"
		else
			"$PHASE28_ACCEPTED_STATE_PACKAGE_BIN"
		fi
	else
		if ((${#args[@]})); then
			bash scripts/phase27-live-hardware-bridge-package.sh "${args[@]}"
		else
			bash scripts/phase27-live-hardware-bridge-package.sh
		fi
	fi
}

run_capture() {
	local manifest="$1"
	local evidence_root="$run_dir/hardware"
	local -a args=(
		--evidence-root "$evidence_root"
		--manifest "$manifest"
		--mode hardware
		--pool-credentials "$pool_credentials"
		--wifi-credentials "$wifi_credentials"
		--duration-seconds "$duration_seconds"
		--port "$port"
		--redact-evidence=true
	)
	if [[ -n "${PHASE28_ACCEPTED_STATE_CAPTURE_BIN:-}" ]]; then
		"$PHASE28_ACCEPTED_STATE_CAPTURE_BIN" "${args[@]}"
	else
		bash scripts/phase27-live-hardware-bridge-evidence.sh "${args[@]}"
	fi
}

cd "$repo_root"
detector_status="not_run"
capture_status="blocked_safe_prerequisite"
rust_raw_log=""

if [[ "$mode" == "hardware" ]]; then
	[[ -n "$port" ]] || die "--port is required in hardware mode"
	[[ -f "$wifi_credentials" ]] || die "Wi-Fi credential file is missing"
	[[ -f "$pool_credentials" ]] || die "pool credential file is missing"

	run_verify_reference >"$run_dir/verify-reference.raw.log" 2>&1
	run_detector >"$run_dir/detect-ultra205.raw.log" 2>&1
	detected_port="$(sed -n 's/^port=//p' "$run_dir/detect-ultra205.raw.log" | tail -1)"
	[[ -n "$detected_port" ]] || die "detect-ultra205 did not report one port"
	[[ "$detected_port" == "$port" ]] || die "detected port does not match --port"
	detector_status="passed"

	run_package >"$run_dir/package.raw.log" 2>&1
	manifest="${PHASE28_ACCEPTED_STATE_MANIFEST:-$repo_root/bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json}"
	[[ -f "$manifest" ]] || die "phase-correct package manifest is missing"
	run_capture "$manifest" >"$run_dir/capture.raw.log" 2>&1
	capture_status="complete"
	rust_raw_log="$run_dir/hardware/live-capture-runtime/flash-monitor.log"
fi

upstream_raw_log="${PHASE28_ACCEPTED_STATE_UPSTREAM_LOG:-$repo_root/scratch/upstream-wire-capture/captures/upstream-monitor.raw.log}"
normalize_markers "$upstream_raw_log" "$run_dir/upstream-markers.redacted.log"
normalize_markers "$rust_raw_log" "$run_dir/rust-markers.redacted.log"

node scripts/phase28.1.1-accepted-state-compare.mjs \
	--upstream-log "$run_dir/upstream-markers.redacted.log" \
	--rust-log "$run_dir/rust-markers.redacted.log" \
	--out "$run_dir/comparison.redacted.txt"

{
	printf 'mode: %s\n' "$mode"
	printf 'attempt: %s\n' "$attempt"
	printf 'board: %s\n' "$board"
	printf 'detector_status: %s\n' "$detector_status"
	printf 'capture_status: %s\n' "$capture_status"
	printf 'duration_seconds: %s\n' "$duration_seconds"
	printf 'credential_contents_read: false\n'
	printf 'raw_register_values_promoted: false\n'
	printf 'redacted: true\n'
	cat "$run_dir/comparison.redacted.txt"
} >"$evidence_out"

printf 'accepted_state_diagnostic_status=%s\n' "$capture_status"
printf 'accepted_state_diagnostic_redacted=true\n'
