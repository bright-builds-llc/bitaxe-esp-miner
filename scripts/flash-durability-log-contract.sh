#!/usr/bin/env bash

durability_log_has_terminal_ready() {
	[[ $# -eq 1 ]] || return 2
	local log_path="$1"
	[[ -f "$log_path" ]] || return 1
	local final_line
	final_line="$(tail -n 1 -- "$log_path")"
	[[ "$final_line" == "usb_session: ready" ]]
}

durability_log_recovery_signature() {
	[[ $# -eq 1 ]] || return 2
	local log_path="$1"
	[[ -f "$log_path" ]] || return 1
	local pattern
	pattern='^Error: recovery_not_observed: phase=(post_flash|post_probe|retry_admission|monitor_admission|final_cleanup),deadline_seconds=(30|60),same_device_seen=(true|false),accessible_seen=(true|false),holder_free_seen=(true|false),stable_samples_max=[0-3],enumeration_changed=(true|false),final_state=(absent|inaccessible|stabilizing),trace_recorded=(true|false)$'
	local signature_line
	if ! signature_line="$(LC_ALL=C grep -aE "$pattern" "$log_path" | tail -n 1)"; then
		return 1
	fi
	[[ -n "$signature_line" ]] || return 1
	printf '%s\n' "${signature_line#Error: recovery_not_observed: }"
}
