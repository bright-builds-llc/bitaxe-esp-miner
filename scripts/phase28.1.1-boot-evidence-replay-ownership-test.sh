#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
boot_evidence="$repo_root/firmware/bitaxe/src/boot_evidence.rs"
live_runtime="$repo_root/firmware/bitaxe/src/live_stratum_runtime.rs"

# Arrange: the Plan 13 replay implementation must be present in the firmware shell.
[[ -f "$boot_evidence" ]]
[[ -f "$live_runtime" ]]

# Act: inspect the replay owner and its bounded timing contract.
rg -q 'ACCEPTED_STATE_REPLAY_INTERVAL_MS, ACCEPTED_STATE_REPLAY_WINDOW_MS' "$boot_evidence"
if rg -q '^const REPLAY_(INTERVAL|WINDOW)_MS' "$boot_evidence"; then
	printf 'phase28_boot_replay_test_error=duplicate_timing_authority\n' >&2
	exit 1
fi
rg -q '\.spawn\(replay_until_window_end\)' "$boot_evidence"
rg -q 'start_replay_task\(\);' "$boot_evidence"

# Assert: boot-lifetime evidence owns replay; Stratum progress cannot gate it.
if rg -q 'maybe_replay_accepted_state_snapshot|AcceptedStateReplayCadence' "$live_runtime"; then
	printf 'phase28_boot_replay_test_error=stratum_owned_replay\n' >&2
	exit 1
fi

printf 'phase28 boot evidence replay ownership test: pass\n'
