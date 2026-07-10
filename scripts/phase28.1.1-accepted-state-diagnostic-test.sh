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
	exit 0
	;;
phase28-fake-port-present)
	cursor_file="${PHASE28_FAKE_PORT_CURSOR:?}"
	sequence_file="${PHASE28_FAKE_PORT_SEQUENCE:?}"
	index=0
	if [[ -f "$cursor_file" ]]; then
		index="$(<"$cursor_file")"
	fi
	state="$(sed -n "$((index + 1))p" "$sequence_file")"
	if [[ -z "$state" ]]; then
		state="$(tail -1 "$sequence_file")"
	fi
	printf '%s\n' "$((index + 1))" >"$cursor_file"
	if [[ "$state" == "present" ]]; then
		exit 0
	fi
	exit 1
	;;
phase28-fake-sleep)
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

wifi="$temp_root/wifi.json"
pool="$temp_root/pool.json"
printf '%s\n' 'wifi-secret-sentinel' >"$wifi"
printf '%s\n' 'pool-secret-sentinel' >"$pool"

expect_failure bash "$script" --mode invalid --attempt accepted-state --duration-seconds 360 --evidence-out "$temp_root/invalid.md"
expect_failure bash "$script" --mode blocked --attempt invalid --duration-seconds 360 --evidence-out "$temp_root/invalid.md"
expect_failure bash "$script" --mode blocked --attempt lifecycle --duration-seconds 359 --evidence-out "$temp_root/short.md"
expect_failure bash "$script" --mode blocked --attempt lifecycle --duration-seconds 360 --reattach-timeout-seconds 90 --evidence-out "$temp_root/long-reattach.md"
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
for fixture in "$fake_verify" "$fake_detect" "$fake_package" "$fake_flash_capture" "$fake_monitor" "$fake_port_present" "$fake_sleep"; do
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

if rg -q 'wifi-secret-sentinel|pool-secret-sentinel' "$temp_root/hardware.md" "$temp_root/hardware.stdout"; then
	fail "credential contents escaped into accepted-state output"
fi

run_lifecycle() {
	local case_name="$1"
	local monitor_mode="$2"
	local sequence="$3"
	local timeout="$4"
	local case_root="$temp_root/$case_name"
	mkdir -p "$case_root"
	tr ' ' '\n' <<<"$sequence" >"$case_root/port.sequence"
	local trace="$case_root/calls.trace"
	local monitor_args="$case_root/monitor.args"
	env \
		"${common_env[@]}" \
		PHASE28_ACCEPTED_STATE_PACKAGE_BIN="$fake_package" \
		PHASE28_ACCEPTED_STATE_CAPTURE_BIN="$fake_flash_capture" \
		PHASE28_ACCEPTED_STATE_MONITOR_BIN="$fake_monitor" \
		PHASE28_ACCEPTED_STATE_PORT_PRESENT_BIN="$fake_port_present" \
		PHASE28_ACCEPTED_STATE_SLEEP_BIN="$fake_sleep" \
		PHASE28_ACCEPTED_STATE_CALL_TRACE="$trace" \
		PHASE28_ACCEPTED_STATE_RUN_ROOT="$case_root/runs" \
		PHASE28_FAKE_FAIL_PACKAGE=1 \
		PHASE28_FAKE_FAIL_FLASH_CAPTURE=1 \
		PHASE28_FAKE_MONITOR_ARGS="$monitor_args" \
		PHASE28_FAKE_MONITOR_MODE="$monitor_mode" \
		PHASE28_FAKE_PORT_SEQUENCE="$case_root/port.sequence" \
		PHASE28_FAKE_PORT_CURSOR="$case_root/port.cursor" \
		bash "$script" --mode hardware --attempt lifecycle --duration-seconds 360 --reattach-timeout-seconds "$timeout" --port "$test_port" --wifi-credentials "$wifi" --pool-credentials "$pool" --manifest "$manifest" --reinit-log "$reinit_log" --evidence-out "$case_root/evidence.md" >"$case_root/stdout"
}

run_lifecycle lifecycle-success duplicate 'present absent present' 5
success_root="$temp_root/lifecycle-success"
rg -q '^accepted_state_lifecycle_checkpoint=armed$' "$success_root/stdout"
rg -q '^accepted_state_diagnostic_status=complete$' "$success_root/stdout"
rg -q '^lifecycle_status: match$' "$success_root/evidence.md"
rg -q '^reinit_stage_count: 5$' "$success_root/evidence.md"
rg -q '^cold_start_stage_count: 5$' "$success_root/evidence.md"
rg -q '^cold_start_marker_count: 6$' "$success_root/evidence.md"
rg -q '^cold_start_flash_performed: false$' "$success_root/evidence.md"
rg -q '^cold_start_reset_performed: false$' "$success_root/evidence.md"
rg -q '^post_capture_detector_status: passed$' "$success_root/evidence.md"
rg -q '^--no-reset$' "$success_root/monitor.args"
rg -q '^--seconds$' "$success_root/monitor.args"
rg -q '^360$' "$success_root/monitor.args"

armed_segment="$(sed -n '/^armed$/,/^capture-complete$/p' "$success_root/calls.trace")"
if printf '%s\n' "$armed_segment" | rg -q 'package|flash-capture|preflight-detector|credential-paths-checked|reference-verified'; then
	fail "lifecycle arming boundary invoked a forbidden preflight or flash action"
fi
rg -q '^post-capture-detector$' "$success_root/calls.trace"

if rg -q 'wifi-secret-sentinel|pool-secret-sentinel' "$success_root/evidence.md" "$success_root/stdout"; then
	fail "credential contents escaped into lifecycle output"
fi

expect_failure run_lifecycle lifecycle-no-disappear complete 'present present present' 2
expect_failure run_lifecycle lifecycle-no-reappear complete 'present absent absent absent' 2
expect_failure run_lifecycle lifecycle-missing missing 'present absent present' 5
expect_failure run_lifecycle lifecycle-hazard hazard 'present absent present' 5

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
