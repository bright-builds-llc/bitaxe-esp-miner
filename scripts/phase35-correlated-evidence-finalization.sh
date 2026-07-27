#!/usr/bin/env bash
# Restoration, cleanup, sealing, and terminal failure helpers for Phase 35.

restore_setting_once() {
	((mutation_started == 1)) || return 0
	if ((restoration_attempted == 1)); then
		((restoration_complete == 1))
		return
	fi
	restoration_attempted=1
	if [[ -n "$fixture_command" ]]; then
		fixture restore "$target_token" "$original_setting" || {
			restoration_secondary_category="restoration_action_failed"
			return 1
		}
	else
		patch_setting "$original_setting" || {
			restoration_secondary_category="restoration_action_failed"
			return 1
		}
	fi
	local restored=""
	read_setting_into restoration restored || {
		restoration_secondary_category="${last_http_category:-restoration_read_failed}"
		return 1
	}
	[[ "$restored" == "$original_setting" ]] || {
		restoration_secondary_category="restoration_value_mismatch"
		return 1
	}
	restoration_complete=1
	record_checkpoint restoration_confirmed "$(hash_fields phase35-restoration-v1 true)"
}

cleanup_resources_once() {
	if ((cleanup_attempted == 1)); then
		((cleanup_complete == 1))
		return
	fi
	cleanup_attempted=1
	if [[ -n "$passive_monitor_pid" ]]; then
		local passive_cleanup_status=0
		set +e
		if kill -0 "$passive_monitor_pid" >/dev/null 2>&1; then
			kill -TERM "$passive_monitor_pid" >/dev/null 2>&1
		fi
		wait "$passive_monitor_pid" >/dev/null 2>&1
		passive_cleanup_status=$?
		set -e
		passive_monitor_pid=""
		if ((passive_cleanup_status != 0 && passive_cleanup_status != 130 && passive_cleanup_status != 143)); then
			cleanup_secondary_category="cleanup_passive_monitor_failed"
			return 1
		fi
	fi
	if [[ -n "$fixture_command" ]]; then
		fixture cleanup || {
			cleanup_secondary_category="cleanup_resource_failure"
			return 1
		}
	elif [[ -n "$port" ]]; then
		local maybe_holders=""
		maybe_holders="$(serial_session_holder_pids "$port")" || {
			cleanup_secondary_category="cleanup_holder_check_failed"
			return 1
		}
		[[ -z "$maybe_holders" ]] || {
			cleanup_secondary_category="cleanup_holder_present"
			return 1
		}
	fi
	cleanup_complete=1
	record_checkpoint cleanup_confirmed "$(hash_fields phase35-cleanup-v1 true)"
}

seal_non_promotion() {
	local category="$1"
	[[ -f "$local_root/non-promotion.seal" ]] && return 0
	write_private "$local_root/non-promotion.seal" \
		"status=non_promotion" \
		"category=${category}" \
		"boundary_schema=${flash_boundary_schema:-none}" \
		"flash_stage=${flash_stage:-none}" \
		"flash_boundary=${flash_boundary:-none}" \
		"device_session_schema=${device_session_schema:-none}" \
		"device_session_category=${device_session_category:-none}" \
		"restoration_secondary_category=${restoration_secondary_category:-none}" \
		"cleanup_secondary_category=${cleanup_secondary_category:-none}" \
		"root_reusable=false"
}

capture_primary_failure() {
	local category="$1"
	[[ -n "$primary_failure_category" ]] || primary_failure_category="$category"
	failure_category="$primary_failure_category"
}

finalize_resources_once() {
	local restoration_status=0
	local cleanup_status=0
	set +e
	restore_setting_once
	restoration_status=$?
	cleanup_resources_once
	cleanup_status=$?
	set -e
	((restoration_status == 0 && cleanup_status == 0))
}

finalize_once() {
	local incoming_status="$1"
	((finalizer_ran == 0)) || return "$incoming_status"
	finalizer_ran=1
	local finalization_status=0
	finalize_resources_once || finalization_status=$?
	if ((incoming_status != 0)); then
		capture_primary_failure "${primary_failure_category:-${failure_category:-supervisor_failed}}"
		seal_non_promotion "$primary_failure_category"
		return "$incoming_status"
	fi
	if ((finalization_status != 0)); then
		capture_primary_failure supervisor_finalization_failed
		seal_non_promotion "$primary_failure_category"
		return 1
	fi
	return 0
}

on_exit() {
	local status=$?
	trap - EXIT
	if ! finalize_once "$status"; then
		status=1
	fi
	exit "$status"
}

fail() {
	capture_primary_failure "$1"
	printf 'failure_category=%s\n' "$primary_failure_category" >&2
	exit 1
}
