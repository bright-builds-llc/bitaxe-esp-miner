#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
workspace_dir="$(cd "${script_dir}/.." && pwd)"
# shellcheck source=scripts/process-group.sh
source "${script_dir}/process-group.sh"
# shellcheck source=scripts/flash-durability-log-contract.sh
source "${script_dir}/flash-durability-log-contract.sh"
umask 077

board="205"
cycles="20"
port=""
manifest=""
wifi_credentials=""
protected_root=""

usage() {
	printf 'usage: just verify-flash-durability board=205 cycles=20 port=PATH manifest=PATH wifi-credentials=PATH protected-root=scratch/PATH\n' >&2
}

for argument in "$@"; do
	case "$argument" in
	board=*) board="${argument#*=}" ;;
	cycles=*) cycles="${argument#*=}" ;;
	port=*) port="${argument#*=}" ;;
	manifest=*) manifest="${argument#*=}" ;;
	wifi-credentials=* | wifi_credentials=*) wifi_credentials="${argument#*=}" ;;
	protected-root=* | protected_root=*) protected_root="${argument#*=}" ;;
	*)
		usage
		exit 2
		;;
	esac
done

[[ "$board" == "205" && "$cycles" == "20" && -n "$port" && -n "$manifest" &&
	-n "$wifi_credentials" && -n "$protected_root" ]] || {
	usage
	exit 2
}
[[ "$port" == /dev/* ]] || {
	printf 'failure_category=transport_absent\n' >&2
	exit 2
}

case "$protected_root" in
"${workspace_dir}"/scratch/*) ;;
scratch/*) protected_root="${workspace_dir}/${protected_root}" ;;
*)
	printf 'failure_category=cleanup_failed\n' >&2
	exit 2
	;;
esac
case "$protected_root" in
*/../* | */.. | */./*)
	printf 'failure_category=cleanup_failed\n' >&2
	exit 2
	;;
esac

[[ ! -e "$protected_root" ]] || {
	printf 'failure_category=cleanup_failed\n' >&2
	exit 1
}
[[ -z "$(git -C "$workspace_dir" status --porcelain --untracked-files=all)" ]] || {
	printf 'failure_category=flash_failed_before_transfer\n' >&2
	exit 1
}
existing_ancestor="$(dirname "$protected_root")"
while [[ ! -e "$existing_ancestor" ]]; do
	existing_ancestor="$(dirname "$existing_ancestor")"
done
canonical_ancestor="$(perl -MCwd=realpath -e 'print realpath($ARGV[0])' "$existing_ancestor")"
case "$canonical_ancestor" in
"${workspace_dir}/scratch" | "${workspace_dir}/scratch/"*) ;;
*)
	printf 'failure_category=cleanup_failed\n' >&2
	exit 2
	;;
esac
[[ -s "$wifi_credentials" || -s "${workspace_dir}/${wifi_credentials}" ]] || {
	printf 'failure_category=flash_failed_before_transfer\n' >&2
	exit 1
}

mkdir -p "$protected_root"
chmod 700 "$protected_root"
bazel build \
	//tools/flash:flash //firmware/bitaxe:firmware_image \
	>"${protected_root}/build.log" 2>&1
chmod 600 "${protected_root}/build.log"
[[ -s "$manifest" || -s "${workspace_dir}/${manifest}" ]] || {
	printf 'failure_category=flash_failed_before_transfer\n' >&2
	exit 1
}

flash_bin="$(bazel info bazel-bin)/tools/flash/flash"
[[ -x "$flash_bin" ]] || {
	printf 'failure_category=flash_failed_before_transfer\n' >&2
	exit 1
}

common_args=(
	"board=${board}"
	"port=${port}"
	"manifest=${manifest}"
	"wifi-credentials=${wifi_credentials}"
)

run_private() {
	local label="$1"
	shift
	local log="${protected_root}/${label}.log"
	if ! "$@" >"$log" 2>&1; then
		chmod 600 "$log"
		local category
		if ! category="$(rg -o 'concurrent_repo_session|foreign_holder|transport_absent|identity_drift|bootloader_connect_failed|flash_failed_before_transfer|flash_failed_after_transfer|monitor_failed|cleanup_failed|recovery_not_observed|repeated_boundary' "$log" | sed -n '1p')"; then
			category=""
		fi
		if [[ "$category" == recovery_not_observed ]]; then
			local signature
			if ! signature="$(durability_log_recovery_signature "$log")"; then
				signature="unavailable"
			fi
			printf 'failure_category=%s failure_signature=%s cycle=%s\n' \
				"$category" "$signature" "$label" >&2
		else
			printf 'failure_category=%s cycle=%s\n' "${category:-cleanup_failed}" "$label" >&2
		fi
		return 1
	fi
	chmod 600 "$log"
	durability_log_has_terminal_ready "$log" || {
		printf 'failure_category=cleanup_failed cycle=%s\n' "$label" >&2
		return 1
	}
}

run_interrupted_flash_monitor() {
	local label="$1"
	local log="${protected_root}/${label}.log"
	local ready="${protected_root}/${label}.group-ready"
	phase_process_group_start "$ready" "$flash_bin" flash-monitor \
		"${common_args[@]}" capture-timeout-seconds=360 >"$log" 2>&1
	local pid="$PHASE_PROCESS_GROUP_PID"
	local admitted=false
	for _ in $(seq 1 4200); do
		if grep -q '^usb_reader: admitted$' "$log" 2>/dev/null; then
			admitted=true
			break
		fi
		phase_process_is_alive "$pid" || break
		sleep 0.1
	done
	if [[ "$admitted" != true ]]; then
		if ! phase_process_group_terminate "$pid" "$label" >/dev/null 2>&1; then
			chmod 600 "$log"
			printf 'failure_category=cleanup_failed cycle=%s\n' "$label" >&2
			return 1
		fi
		chmod 600 "$log"
		printf 'failure_category=monitor_failed cycle=%s\n' "$label" >&2
		return 1
	fi
	kill -INT "$pid"
	if ! wait "$pid"; then
		chmod 600 "$log"
		printf 'failure_category=cleanup_failed cycle=%s\n' "$label" >&2
		return 1
	fi
	chmod 600 "$log"
	durability_log_has_terminal_ready "$log" || {
		printf 'failure_category=cleanup_failed cycle=%s\n' "$label" >&2
		return 1
	}
}

for cycle in $(seq 1 5); do
	run_private "cycle-${cycle}-detect" "$flash_bin" detect "board=${board}" "port=${port}"
	run_private "cycle-${cycle}-flash" "$flash_bin" flash "${common_args[@]}"
done

for cycle in $(seq 6 10); do
	run_private "cycle-${cycle}-monitor" "$flash_bin" monitor \
		"board=${board}" "port=${port}" capture-timeout-seconds=360
	run_private "cycle-${cycle}-flash" "$flash_bin" flash "${common_args[@]}"
done

for cycle in $(seq 11 15); do
	run_private "cycle-${cycle}-flash-monitor" "$flash_bin" flash-monitor \
		"${common_args[@]}" capture-timeout-seconds=360
	run_private "cycle-${cycle}-reflash" "$flash_bin" flash "${common_args[@]}"
done

for cycle in $(seq 16 20); do
	run_interrupted_flash_monitor "cycle-${cycle}-flash-monitor-interrupt"
	run_private "cycle-${cycle}-reflash" "$flash_bin" flash "${common_args[@]}"
done

printf 'durability_result=ready cycles=20\n'
