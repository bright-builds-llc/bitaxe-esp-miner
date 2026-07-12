#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
boot_evidence="$repo_root/firmware/bitaxe/src/boot_evidence.rs"
main="$repo_root/firmware/bitaxe/src/main.rs"
runtime_uptime="$repo_root/firmware/bitaxe/src/runtime_uptime.rs"
live_runtime="$repo_root/firmware/bitaxe/src/live_stratum_runtime.rs"
log_buffer="$repo_root/firmware/bitaxe/src/log_buffer.rs"

# Arrange: the Plan 13 replay implementation must be present in the firmware shell.
[[ -f "$boot_evidence" ]]
[[ -f "$runtime_uptime" ]]
[[ -f "$live_runtime" ]]

# Act: inspect the replay owner and its bounded timing contract.
rg -q 'ACCEPTED_STATE_REPLAY_INTERVAL_MS' "$boot_evidence"
rg -q 'ACCEPTED_STATE_REPLAY_WINDOW_MS' "$boot_evidence"
if rg -q '^const REPLAY_(INTERVAL|WINDOW)_MS' "$boot_evidence"; then
	printf 'phase28_boot_replay_test_error=duplicate_timing_authority\n' >&2
	exit 1
fi
rg -q 'RuntimeHeartbeatModel' "$boot_evidence"
rg -q 'runtime_uptime::millis' "$boot_evidence"
rg -q 'log::info!\("\{marker\}"\)' "$boot_evidence"
rg -q 'initialize_observer\(\);' "$main"
[[ "$(rg -c '\.spawn\(' "$boot_evidence")" == "1" ]]
rg -q 'esp_timer_get_time' "$runtime_uptime"
if rg -q 'esp_timer_get_time' "$repo_root/firmware/bitaxe/src/runtime_snapshot.rs" "$repo_root/firmware/bitaxe/src/http_api.rs"; then
	printf 'phase28_boot_replay_test_error=duplicate_uptime_authority\n' >&2
	exit 1
fi
rg -q 'record_listener_armed\(\);' "$live_runtime"
if rg -q 'runtime_heartbeat' "$log_buffer"; then
	printf 'phase28_boot_replay_test_error=heartbeat_retained\n' >&2
	exit 1
fi

# Assert: boot-lifetime evidence owns replay; Stratum progress cannot gate it.
if rg -q 'maybe_replay_accepted_state_snapshot|AcceptedStateReplayCadence' "$live_runtime"; then
	printf 'phase28_boot_replay_test_error=stratum_owned_replay\n' >&2
	exit 1
fi

printf 'phase28 boot evidence replay ownership test: pass\n'
