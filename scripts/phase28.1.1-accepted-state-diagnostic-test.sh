#!/usr/bin/env bash
set -euo pipefail

emit_complete_accepted_state_log() {
	local stage
	printf 'bitaxe-rust boot commit=test\n'
	printf 'h4_continuous_result=listener_armed\n'
	for stage in post_enumerate post_mining_ready post_max_baud post_mask_reload post_first_work; do
		local result_correlated="false"
		local submit_observed="false"
		if [[ "$stage" == "post_first_work" ]]; then
			result_correlated="true"
			submit_observed="true"
		fi
		if [[ "${PHASE28_FAKE_MONITOR_MODE:-complete}" == "missing" && "$stage" == "post_max_baud" ]]; then
			continue
		fi
		printf 'accepted_state_snapshot stage=%s observation=available chip_count_class=match readable_responses=1 error_counter_active=false domain_counter_active=false total_counter_active=true power_delta_class=flat result_correlated=%s submit_observed=%s redacted=true\n' "$stage" "$result_correlated" "$submit_observed"
	done
}

next_sequence_value() {
	local sequence_file="$1"
	local cursor_file="$2"
	local index=0
	if [[ -f "$cursor_file" ]]; then
		index="$(<"$cursor_file")"
	fi
	local value
	value="$(sed -n "$((index + 1))p" "$sequence_file")"
	if [[ -z "$value" ]]; then
		value="$(tail -1 "$sequence_file")"
	fi
	printf '%s\n' "$((index + 1))" >"$cursor_file"
	printf '%s\n' "$value"
}

fixture_name="$(basename "${BASH_SOURCE[0]}")"
case "$fixture_name" in
phase28-fake-verify)
	exit 0
	;;
phase28-fake-detect)
	printf 'port=%s\n' "${PHASE28_FAKE_PORT:?}"
	exit 0
	;;
phase28-fake-package)
	[[ "${PHASE28_FAKE_FAIL_PACKAGE:-0}" != "1" ]] || exit 91
	printf '%s\n' "$@" >"${PHASE28_FAKE_PACKAGE_ARGS:?}"
	exit 0
	;;
phase28-fake-flash-capture)
	[[ "${PHASE28_FAKE_FAIL_FLASH_CAPTURE:-0}" != "1" ]] || exit 92
	printf '%s\n' "$@" >"${PHASE28_FAKE_CAPTURE_ARGS:?}"
	evidence_root=""
	while (($#)); do
		if [[ "$1" == "--evidence-root" ]]; then
			evidence_root="$2"
			shift 2
		else
			shift
		fi
	done
	[[ -n "$evidence_root" ]]
	mkdir -p "$evidence_root/live-capture-runtime"
	emit_complete_accepted_state_log >"$evidence_root/live-capture-runtime/flash-monitor.log"
	if [[ "${PHASE28_FAKE_MONITOR_MODE:-complete}" == "hazard" ]]; then
		printf 'Guru Meditation Error: stack overflow\n' >>"$evidence_root/live-capture-runtime/flash-monitor.log"
	fi
	exit 0
	;;
phase28-fake-monitor)
	printf '%s\n' "$@" >"${PHASE28_FAKE_MONITOR_ARGS:?}"
	output_log=""
	saw_no_reset="false"
	while (($#)); do
		case "$1" in
		--out)
			output_log="$2"
			shift 2
			;;
		--no-reset)
			saw_no_reset="true"
			shift
			;;
		*) shift ;;
		esac
	done
	[[ "$saw_no_reset" == "true" && -n "$output_log" ]]
	emit_complete_accepted_state_log >"$output_log"
	if [[ "${PHASE28_FAKE_MONITOR_MODE:-complete}" == "duplicate" ]]; then
		printf 'accepted_state_snapshot stage=post_enumerate observation=available chip_count_class=match readable_responses=1 error_counter_active=false domain_counter_active=false total_counter_active=true power_delta_class=flat result_correlated=false submit_observed=false redacted=true\n' >>"$output_log"
	fi
	if [[ "${PHASE28_FAKE_MONITOR_MODE:-complete}" == "hazard" ]]; then
		printf 'Guru Meditation Error: stack overflow\n' >>"$output_log"
	fi
	if [[ "${PHASE28_FAKE_MONITOR_MODE:-complete}" == "detached-descendant" ]]; then
		perl -MPOSIX=setsid -e '
			defined(setsid()) or die "setsid failed: $!\n";
			for my $path ($ENV{PHASE28_FAKE_WATCHER_PID_FILE}, $ENV{PHASE13_MONITOR_GROUP_STATE_FILE}) {
				open my $file, ">", $path or die "open state file failed: $!\n";
				print {$file} "$$\n" or die "write state file failed: $!\n";
				close $file or die "close state file failed: $!\n";
			}
			exec {"/bin/bash"} "/bin/bash", "-c", q{trap "" INT TERM; while true; do sleep 1; done};
			die "exec failed: $!\n";
		' &
		wait "$!"
	fi
	exit 0
	;;
phase28-fake-port-present)
	state="$(next_sequence_value "${PHASE28_FAKE_PORT_SEQUENCE:?}" "${PHASE28_FAKE_PORT_CURSOR:?}")"
	if [[ "$state" == "present" ]]; then
		exit 0
	fi
	exit 1
	;;
phase28-fake-sleep)
	exit 0
	;;
phase28-fake-clock)
	next_sequence_value "${PHASE28_FAKE_CLOCK_SEQUENCE:?}" "${PHASE28_FAKE_CLOCK_CURSOR:?}"
	exit 0
	;;
phase28-fake-read)
	read_value="$(next_sequence_value "${PHASE28_FAKE_READ_SEQUENCE:?}" "${PHASE28_FAKE_READ_CURSOR:?}")"
	case "$read_value" in
	timeout) exit 142 ;;
	eof) exit 1 ;;
	*) printf '%s\n' "$read_value" ;;
	esac
	exit 0
	;;
esac

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
script="$repo_root/scripts/phase28.1.1-accepted-state-diagnostic.sh"
test_script="$repo_root/scripts/phase28.1.1-accepted-state-diagnostic-test.sh"
temp_root="$(mktemp -d "$repo_root/scratch/accepted-state-diagnostic-test.XXXXXX")"
trap 'rm -rf "$temp_root"' EXIT

fail() {
	printf 'accepted_state_diagnostic_test_error: %s\n' "$1" >&2
	exit 1
}

expect_failure() {
	if "$@" >"$temp_root/expected-failure.stdout" 2>"$temp_root/expected-failure.stderr"; then
		fail "command unexpectedly succeeded"
	fi
}

wait_for_file() {
	local path="$1"
	for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20; do
		[[ -s "$path" ]] && return 0
		sleep 0.05
	done
	fail "timed out waiting for $path"
}

assert_pid_stopped() {
	local pid="$1"
	for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20; do
		if ! kill -0 "$pid" 2>/dev/null; then
			return 0
		fi
		sleep 0.05
	done
	fail "descendant watcher $pid survived cleanup"
}

wifi="$temp_root/wifi.json"
pool="$temp_root/pool.json"
printf '%s\n' 'wifi-secret-sentinel' >"$wifi"
printf '%s\n' 'pool-secret-sentinel' >"$pool"

expect_failure bash "$script" --mode invalid --attempt accepted-state --duration-seconds 360 --evidence-out "$temp_root/invalid.md"
expect_failure bash "$script" --mode blocked --attempt invalid --duration-seconds 360 --evidence-out "$temp_root/invalid.md"
expect_failure bash "$script" --mode blocked --attempt lifecycle --duration-seconds 359 --evidence-out "$temp_root/short.md"
expect_failure bash "$script" --mode blocked --attempt lifecycle --duration-seconds 360 --reattach-timeout-seconds 90 --evidence-out "$temp_root/long-reattach.md"
expect_failure bash "$script" --mode blocked --attempt lifecycle --duration-seconds 360 --attestation-timeout-seconds 299 --evidence-out "$temp_root/short-attestation.md"
expect_failure bash "$script" --mode hardware --attempt accepted-state --duration-seconds 360 --port /dev/test --wifi-credentials "$temp_root/missing-wifi" --pool-credentials "$pool" --evidence-out "$temp_root/missing.md"

bash "$script" --mode blocked --attempt lifecycle --duration-seconds 360 --evidence-out "$temp_root/blocked.md" >"$temp_root/blocked.stdout"
rg -q '^capture_status: blocked_safe_prerequisite$' "$temp_root/blocked.md"
rg -q '^lifecycle_status: unavailable$' "$temp_root/blocked.md"
rg -q '^redacted: true$' "$temp_root/blocked.md"

fake_verify="$temp_root/phase28-fake-verify"
fake_detect="$temp_root/phase28-fake-detect"
fake_package="$temp_root/phase28-fake-package"
fake_flash_capture="$temp_root/phase28-fake-flash-capture"
fake_monitor="$temp_root/phase28-fake-monitor"
fake_port_present="$temp_root/phase28-fake-port-present"
fake_sleep="$temp_root/phase28-fake-sleep"
fake_clock="$temp_root/phase28-fake-clock"
fake_read="$temp_root/phase28-fake-read"
for fixture in "$fake_verify" "$fake_detect" "$fake_package" "$fake_flash_capture" "$fake_monitor" "$fake_port_present" "$fake_sleep" "$fake_clock" "$fake_read"; do
	ln -s "$test_script" "$fixture"
done

test_port="/dev/ultra205-test"
manifest="$temp_root/package.json"
source_commit="$(git -C "$repo_root" rev-parse HEAD)"
printf '{"source_commit":"%s"}\n' "$source_commit" >"$manifest"
upstream_log="$temp_root/upstream.log"
emit_complete_accepted_state_log >"$upstream_log"

common_env=(
	PHASE28_ACCEPTED_STATE_DETECT_BIN="$fake_detect"
	PHASE28_ACCEPTED_STATE_VERIFY_BIN="$fake_verify"
	PHASE28_ACCEPTED_STATE_MANIFEST="$manifest"
	PHASE28_ACCEPTED_STATE_SKIP_SOURCE_CLEAN_CHECK=1
	PHASE28_FAKE_PORT="$test_port"
)

expect_failure env "${common_env[@]}" PHASE28_ACCEPTED_STATE_RUN_ROOT="$temp_root/runs-mismatch" bash "$script" --mode hardware --attempt accepted-state --duration-seconds 360 --port /dev/wrong --wifi-credentials "$wifi" --pool-credentials "$pool" --evidence-out "$temp_root/mismatch.md"

package_args="$temp_root/package.args"
capture_args="$temp_root/capture.args"
env \
	"${common_env[@]}" \
	PHASE28_ACCEPTED_STATE_PACKAGE_BIN="$fake_package" \
	PHASE28_ACCEPTED_STATE_CAPTURE_BIN="$fake_flash_capture" \
	PHASE28_ACCEPTED_STATE_UPSTREAM_LOG="$upstream_log" \
	PHASE28_ACCEPTED_STATE_RUN_ROOT="$temp_root/runs-accepted" \
	PHASE28_FAKE_PACKAGE_ARGS="$package_args" \
	PHASE28_FAKE_CAPTURE_ARGS="$capture_args" \
	bash "$script" --mode hardware --attempt accepted-state --duration-seconds 360 --port "$test_port" --wifi-credentials "$wifi" --pool-credentials "$pool" --evidence-out "$temp_root/hardware.md" >"$temp_root/hardware.stdout"

rg -q '^--investigation$' "$package_args"
rg -q '^accepted_state_snapshot$' "$package_args"
rg -q '^--mode$' "$capture_args"
rg -q '^hardware$' "$capture_args"
rg -q '^--duration-seconds$' "$capture_args"
rg -q '^360$' "$capture_args"
rg -q '^--redact-evidence=true$' "$capture_args"
rg -q '^board: 205$' "$temp_root/hardware.md"
rg -q '^manifest_source_commit_match: true$' "$temp_root/hardware.md"
rg -q '^recommended_investigation: none$' "$temp_root/hardware.md"
reinit_log="$(sed -n 's/^accepted_state_reinit_log=//p' "$temp_root/hardware.stdout")"
[[ -n "$reinit_log" && -f "$reinit_log" ]] || fail "accepted-state run did not expose retained reinit log"

missing_reinit_stdout="$temp_root/missing-reinit.stdout"
expect_failure env \
	"${common_env[@]}" \
	PHASE28_ACCEPTED_STATE_PACKAGE_BIN="$fake_package" \
	PHASE28_ACCEPTED_STATE_CAPTURE_BIN="$fake_flash_capture" \
	PHASE28_ACCEPTED_STATE_UPSTREAM_LOG="$upstream_log" \
	PHASE28_ACCEPTED_STATE_RUN_ROOT="$temp_root/runs-incomplete-accepted" \
	PHASE28_FAKE_PACKAGE_ARGS="$temp_root/incomplete-package.args" \
	PHASE28_FAKE_CAPTURE_ARGS="$temp_root/incomplete-capture.args" \
	PHASE28_FAKE_MONITOR_MODE=missing \
	bash "$script" --mode hardware --attempt accepted-state --duration-seconds 360 --port "$test_port" --wifi-credentials "$wifi" --pool-credentials "$pool" --evidence-out "$temp_root/incomplete-hardware.md"
cp "$temp_root/expected-failure.stdout" "$missing_reinit_stdout"
if rg -q '^accepted_state_manifest=' "$missing_reinit_stdout"; then
	fail "incomplete accepted-state run exposed an armable manifest"
fi

if rg -q 'wifi-secret-sentinel|pool-secret-sentinel' "$temp_root/hardware.md" "$temp_root/hardware.stdout"; then
	fail "credential contents escaped into accepted-state output"
fi

lifecycle_wrapper_pid=""

run_lifecycle() {
	local case_name="$1"
	local monitor_mode="$2"
	local port_sequence="$3"
	local clock_sequence="$4"
	local read_sequence="$5"
	local selected_reinit_log="${6:-$reinit_log}"
	local read_mode="${7:-fixture}"
	local launch_mode="${8:-foreground}"
	local case_root="$temp_root/$case_name"
	mkdir -p "$case_root"
	tr ' ' '\n' <<<"$port_sequence" >"$case_root/port.sequence"
	tr ' ' '\n' <<<"$clock_sequence" >"$case_root/clock.sequence"
	tr ' ' '\n' <<<"$read_sequence" >"$case_root/read.sequence"
	local trace="$case_root/calls.trace"
	local monitor_args="$case_root/monitor.args"
	local child_pid_file="$case_root/child.pid"
	local run_status
	local -a read_env=(PHASE28_ACCEPTED_STATE_READ_BIN="$fake_read")
	if [[ "$read_mode" == "native" ]]; then
		read_env=(PHASE28_ACCEPTED_STATE_READ_BIN=)
	fi
	local -a lifecycle_command=(env
		"${common_env[@]}"
		PHASE28_ACCEPTED_STATE_PACKAGE_BIN="$fake_package"
		PHASE28_ACCEPTED_STATE_CAPTURE_BIN="$fake_flash_capture"
		PHASE28_ACCEPTED_STATE_MONITOR_BIN="$fake_monitor"
		PHASE28_ACCEPTED_STATE_PORT_PRESENT_BIN="$fake_port_present"
		PHASE28_ACCEPTED_STATE_SLEEP_BIN="$fake_sleep"
		PHASE28_ACCEPTED_STATE_CLOCK_BIN="$fake_clock"
		"${read_env[@]}"
		PHASE28_ACCEPTED_STATE_CALL_TRACE="$trace"
		PHASE28_ACCEPTED_STATE_CHILD_PID_FILE="$child_pid_file"
		PHASE28_ACCEPTED_STATE_RUN_ROOT="$case_root/runs"
		PHASE28_FAKE_FAIL_PACKAGE=1
		PHASE28_FAKE_FAIL_FLASH_CAPTURE=1
		PHASE28_FAKE_MONITOR_ARGS="$monitor_args"
		PHASE28_FAKE_MONITOR_MODE="$monitor_mode"
		PHASE28_FAKE_WATCHER_PID_FILE="$case_root/watcher.pid"
		PHASE28_FAKE_PORT_SEQUENCE="$case_root/port.sequence"
		PHASE28_FAKE_PORT_CURSOR="$case_root/port.cursor"
		PHASE28_FAKE_CLOCK_SEQUENCE="$case_root/clock.sequence"
		PHASE28_FAKE_CLOCK_CURSOR="$case_root/clock.cursor"
		PHASE28_FAKE_READ_SEQUENCE="$case_root/read.sequence"
		PHASE28_FAKE_READ_CURSOR="$case_root/read.cursor"
		bash "$script" --mode hardware --attempt lifecycle --duration-seconds 360 --reattach-timeout-seconds 60 --attestation-timeout-seconds 300 --port "$test_port" --wifi-credentials "$wifi" --pool-credentials "$pool" --manifest "$manifest" --reinit-log "$selected_reinit_log" --evidence-out "$case_root/evidence.md")
	if [[ "$launch_mode" == "background" ]]; then
		"${lifecycle_command[@]}" >"$case_root/stdout" 2>"$case_root/stderr" &
		lifecycle_wrapper_pid=$!
		return 0
	fi

	set +e
	"${lifecycle_command[@]}" >"$case_root/stdout" 2>"$case_root/stderr"
	run_status=$?
	set -e
	[[ ! -s "$child_pid_file" ]] || fail "$case_name left a live monitor child recorded"
	return "$run_status"
}

success_port_sequence='present absent absent absent present'
success_clock_sequence='0 0 0 0 4999 5000 5000 5000 60000 60000'
success_read_sequence='both-power-paths-removed barrel-then-usb-restored'
run_lifecycle lifecycle-success duplicate "$success_port_sequence" "$success_clock_sequence" "$success_read_sequence"
success_root="$temp_root/lifecycle-success"
rg -q '^accepted_state_lifecycle_checkpoint=armed$' "$success_root/stdout"
rg -q '^accepted_state_lifecycle_expected_token=both-power-paths-removed$' "$success_root/stdout"
rg -q '^accepted_state_attestation_deadline_ms=300000$' "$success_root/stdout"
rg -q '^operator_attested_both_power_paths_removed=true$' "$success_root/stdout"
rg -q '^accepted_state_usb_absence_measured_ms=5000$' "$success_root/stdout"
rg -q '^accepted_state_lifecycle_expected_token=barrel-then-usb-restored$' "$success_root/stdout"
rg -q '^accepted_state_restore_deadline_ms=60000$' "$success_root/stdout"
rg -q '^accepted_state_diagnostic_status=complete$' "$success_root/stdout"
rg -q '^lifecycle_status: match$' "$success_root/evidence.md"
rg -q '^reinit_stage_count: 5$' "$success_root/evidence.md"
rg -q '^cold_start_stage_count: 5$' "$success_root/evidence.md"
rg -q '^cold_start_marker_count: 6$' "$success_root/evidence.md"
rg -q '^cold_start_flash_performed: false$' "$success_root/evidence.md"
rg -q '^cold_start_reset_performed: false$' "$success_root/evidence.md"
rg -q '^operator_attested_both_power_paths_removed: true$' "$success_root/evidence.md"
rg -q '^usb_absence_measured_ms: 5000$' "$success_root/evidence.md"
rg -q '^barrel_removal_electronically_verified: false$' "$success_root/evidence.md"
rg -q '^reattach_deadline_ms: 60000$' "$success_root/evidence.md"
rg -q '^reappearance_elapsed_ms: 60000$' "$success_root/evidence.md"
rg -q '^monitor_start_reserve_ms: 10000$' "$success_root/evidence.md"
rg -q '^replay_interval_ms: 2000$' "$success_root/evidence.md"
rg -q '^replay_window_ms: 180000$' "$success_root/evidence.md"
rg -q '^latest_safe_replay_ms: 72000$' "$success_root/evidence.md"
rg -q '^post_capture_detector_status: passed$' "$success_root/evidence.md"
rg -q '^--no-reset$' "$success_root/monitor.args"
rg -q '^--seconds$' "$success_root/monitor.args"
rg -q '^360$' "$success_root/monitor.args"

run_lifecycle lifecycle-native-read complete "$success_port_sequence" "$success_clock_sequence" "$success_read_sequence" "$reinit_log" native <<'EOF'
both-power-paths-removed
barrel-then-usb-restored
EOF
rg -q '^accepted_state_diagnostic_status=complete$' "$temp_root/lifecycle-native-read/stdout"
if rg -n 'read[^[:cntrl:]]*-t[[:space:]]+[0-9]*\.[0-9]+' "$script"; then
	fail "diagnostic wrapper uses a fractional Bash read timeout"
fi
expect_failure run_lifecycle lifecycle-native-read-timeout complete 'present' '0 0 300000' 'unused' "$reinit_log" native < <(sleep 1)
rg -q 'timed out waiting for both-power attestation token' "$temp_root/lifecycle-native-read-timeout/stderr"
if rg -q 'invalid timeout specification' "$temp_root/lifecycle-native-read-timeout/stderr"; then
	fail "native token polling used an unsupported Bash timeout"
fi

armed_segment="$(sed -n '/^armed$/,/^capture-complete$/p' "$success_root/calls.trace")"
if printf '%s\n' "$armed_segment" | rg -q 'package|flash-capture|preflight-detector|credential-paths-checked|reference-verified'; then
	fail "lifecycle arming boundary invoked a forbidden preflight or flash action"
fi
rg -q '^post-capture-detector$' "$success_root/calls.trace"

if rg -q 'wifi-secret-sentinel|pool-secret-sentinel' "$success_root/evidence.md" "$success_root/stdout"; then
	fail "credential contents escaped into lifecycle output"
fi

expect_failure run_lifecycle lifecycle-wrong-attestation complete 'present' '0 0' 'wrong-token'
expect_failure run_lifecycle lifecycle-absent-attestation complete 'present' '0 0' 'eof'
expect_failure run_lifecycle lifecycle-abandoned-armed complete 'present' '0 0 300000' 'timeout'
[[ ! -s "$temp_root/lifecycle-abandoned-armed/child.pid" ]] || fail "abandoned armed wait left a child"
if rg -q '^monitor-no-reset$' "$temp_root/lifecycle-abandoned-armed/calls.trace"; then
	fail "abandoned armed wait started a monitor child"
fi

expect_failure run_lifecycle lifecycle-token-crossed-deadline complete 'present' '0 299999 300000' 'both-power-paths-removed'
rg -q 'timed out waiting for both-power attestation token' "$temp_root/lifecycle-token-crossed-deadline/stderr"
if rg -q '^operator_attested_both_power_paths_removed=true$' "$temp_root/lifecycle-token-crossed-deadline/stdout"; then
	fail "token crossing the deadline was accepted"
fi

expect_failure run_lifecycle lifecycle-present-at-attestation complete 'present present' '0 0 0 0' 'both-power-paths-removed'
expect_failure run_lifecycle lifecycle-absence-4999 complete 'present absent absent present' '0 0 0 0 4999' 'both-power-paths-removed'
expect_failure run_lifecycle lifecycle-wrong-restore complete 'present absent absent' '0 0 0 0 5000 5000 5000' 'both-power-paths-removed wrong-token'
expect_failure run_lifecycle lifecycle-absent-restore complete 'present absent absent' '0 0 0 0 5000 5000' 'both-power-paths-removed eof'
expect_failure run_lifecycle lifecycle-abandoned-restore complete 'present absent absent' '0 0 0 0 5000 5000 60000' 'both-power-paths-removed timeout'
[[ ! -s "$temp_root/lifecycle-abandoned-restore/child.pid" ]] || fail "abandoned restore wait left a child"
if rg -q '^monitor-no-reset$' "$temp_root/lifecycle-abandoned-restore/calls.trace"; then
	fail "abandoned restore wait started a monitor child"
fi

expect_failure run_lifecycle lifecycle-late-restore complete 'present absent absent' '0 0 0 0 5000 60000' 'both-power-paths-removed barrel-then-usb-restored'
expect_failure run_lifecycle lifecycle-absent-at-60000 complete 'present absent absent absent' '0 0 0 0 5000 5000 5000 60000' "$success_read_sequence"
expect_failure run_lifecycle lifecycle-present-at-60001 complete 'present absent absent present' '0 0 0 0 5000 5000 5000 60001' "$success_read_sequence"

run_lifecycle lifecycle-cancel-descendant detached-descendant "$success_port_sequence" "$success_clock_sequence" "$success_read_sequence" "$reinit_log" fixture background
wait_for_file "$temp_root/lifecycle-cancel-descendant/watcher.pid"
kill -TERM "$lifecycle_wrapper_pid"
set +e
wait "$lifecycle_wrapper_pid"
lifecycle_cancel_status=$?
set -e
[[ "$lifecycle_cancel_status" -ne 0 ]] || fail "cancelled lifecycle unexpectedly succeeded"
assert_pid_stopped "$(<"$temp_root/lifecycle-cancel-descendant/watcher.pid")"
[[ ! -s "$temp_root/lifecycle-cancel-descendant/child.pid" ]] || fail "cancelled lifecycle falsely retained active child state"

incomplete_reinit="$temp_root/incomplete-reinit.log"
PHASE28_FAKE_MONITOR_MODE=missing emit_complete_accepted_state_log >"$incomplete_reinit"
expect_failure run_lifecycle lifecycle-incomplete-reinit complete 'present' '0' 'both-power-paths-removed' "$incomplete_reinit"
if rg -q '^armed$' "$temp_root/lifecycle-incomplete-reinit/calls.trace"; then
	fail "incomplete reinit reached lifecycle arming"
fi

expect_failure run_lifecycle lifecycle-hazard hazard "$success_port_sequence" "$success_clock_sequence" "$success_read_sequence"

if rg -q 'post_max_baud_delay_2000|match_upstream_register_read_poll|upstream_like_long_block_receive|ticket_mask_asic_difficulty' "$script"; then
	fail "closed diagnostic wrapper contains a banned hypothesis"
fi

fixture_source="$temp_root/fixture/esp-miner"
mkdir -p "$fixture_source/components/asic/include" "$fixture_source/components/asic"
printf '#define BM1366_SERIALTX_DEBUG false\n#define BM1366_SERIALRX_DEBUG false\n' >"$fixture_source/components/asic/include/bm1366.h"
printf 'void BM1366_read_registers(void) {}\n' >"$fixture_source/components/asic/bm1366.c"
BM1366_UPSTREAM_SOURCE="$fixture_source" \
	BM1366_UPSTREAM_SCRATCH="$temp_root/upstream-scratch" \
	PHASE28_VERIFY_REFERENCE_BIN="$fake_verify" \
	bash "$repo_root/scripts/phase28.1.1-upstream-wire-capture.sh" >"$temp_root/upstream.stdout"
rg -q '^#define BM1366_SERIALTX_DEBUG true$' "$temp_root/upstream-scratch/esp-miner/components/asic/include/bm1366.h"
rg -q '^#define BM1366_SERIALRX_DEBUG true$' "$temp_root/upstream-scratch/esp-miner/components/asic/include/bm1366.h"
rg -q 'accepted_state_snapshot stage=post_first_work' "$temp_root/upstream-scratch/captures/accepted-state-template.log"

printf 'accepted_state_diagnostic_test: passed\n'
