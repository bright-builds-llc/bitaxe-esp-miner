#!/usr/bin/env bash
set -euo pipefail

usage() {
	printf 'usage: %s --log <path> --out <path>\n' "$0" >&2
	printf 'Classifies a wrapper-captured Phase 12 firmware log before any live mining smoke or soak.\n' >&2
}

log_path=""
out_path=""

while [[ "$#" -gt 0 ]]; do
	case "$1" in
	--log)
		if [[ "$#" -lt 2 || -z "${2:-}" ]]; then
			printf 'error: --log requires a path\n' >&2
			usage
			exit 2
		fi
		log_path="$2"
		shift 2
		;;
	--out)
		if [[ "$#" -lt 2 || -z "${2:-}" ]]; then
			printf 'error: --out requires a path\n' >&2
			usage
			exit 2
		fi
		out_path="$2"
		shift 2
		;;
	-h | --help)
		usage
		exit 0
		;;
	*)
		printf 'error: unexpected argument: %s\n' "$1" >&2
		usage
		exit 2
		;;
	esac
done

if [[ -z "$log_path" ]]; then
	printf 'error: missing required --log path\n' >&2
	usage
	exit 2
fi

if [[ -z "$out_path" ]]; then
	printf 'error: missing required --out path\n' >&2
	usage
	exit 2
fi

if [[ ! -f "$log_path" ]]; then
	printf 'error: input log does not exist: %s\n' "$log_path" >&2
	exit 1
fi

mkdir -p "$(dirname "$out_path")"

write_blocked_output() {
	local reason="$1"
	local work_submission="$2"

	{
		printf 'phase12_mining_smoke_preflight=blocked\n'
		printf 'reason=%s\n' "$reason"
		printf 'controlled_mining_smoke=not_run\n'
		printf 'bounded_mining_soak=not_run\n'
		printf 'work_submission=%s\n' "$work_submission"
		printf 'source_log=%s\n' "$log_path"
	} >"$out_path"
}

if grep -Eiq 'mining_loop_status=active|work_submission=ready|accepted share|rejected share|share accepted|share rejected|mining_activity=active' "$log_path"; then
	write_blocked_output "bounded_probe_absent" "unknown"
	printf 'error: active mining/share marker found without a bounded Phase 12 mining probe\n' >&2
	exit 1
fi

if ! grep -Fq 'mining_loop_status=blocked' "$log_path"; then
	write_blocked_output "safe_blocked_marker_missing" "unknown"
	printf 'error: required mining_loop_status=blocked marker was not found in %s\n' "$log_path" >&2
	exit 1
fi

if ! grep -Fq 'work_submission=disabled' "$log_path"; then
	write_blocked_output "work_submission_disabled_marker_missing" "unknown"
	printf 'error: required work_submission=disabled marker was not found in %s\n' "$log_path" >&2
	exit 1
fi

write_blocked_output "firmware_fail_closed" "disabled"
printf 'phase12_mining_smoke_preflight=blocked reason=firmware_fail_closed work_submission=disabled\n'
