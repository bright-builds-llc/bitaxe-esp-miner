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

runner="$repo_root/scripts/phase28.1.1-exact-head-hardware-attempt.sh"
prevalidated_root="$temp_root/plan13-control"
prevalidated_trace="$temp_root/plan13-prevalidated.trace"
: >"$prevalidated_trace"

expect_failure env PHASE28_ACCEPTED_STATE_CALL_TRACE="$prevalidated_trace" bash "$script" --mode plan13-prevalidated --effect-id detector_board_info
[[ ! -s "$prevalidated_trace" ]] || fail "direct plan13-prevalidated entry reached an effect sentinel"
expect_failure bash "$script" --mode plan13-prevalidated --effect-id detector_board_info --port "$test_port"

prevalidated_begin="$(env \
	PHASE28_ATTEMPT_CONTROL_ROOT="$prevalidated_root" \
	PHASE28_ALLOW_DIRTY_TEST=1 \
	PHASE28_TEST_HEAD="$source_commit" \
	bash "$runner" begin-attempt --hardware-exact-head "$source_commit")"
prevalidated_handle="$(sed -n 's/^resume_handle=//p' <<<"$prevalidated_begin")"
[[ "$prevalidated_handle" =~ ^[0-9a-f]{64}$ ]] || fail "prevalidated runner did not return an opaque handle"
env \
	PHASE28_ATTEMPT_CONTROL_ROOT="$prevalidated_root" \
	PHASE28_ALLOW_DIRTY_TEST=1 \
	PHASE28_TEST_HEAD="$source_commit" \
	bash "$runner" deliver-token \
	--resume-handle "$prevalidated_handle" \
	--checkpoint-token plan13-connected-entry-v1 \
	--response-token ultra205-remains-connected >/dev/null
prevalidated_stdout="$temp_root/plan13-prevalidated.stdout"
prevalidated_stderr="$temp_root/plan13-prevalidated.stderr"
env \
	PHASE28_ATTEMPT_CONTROL_ROOT="$prevalidated_root" \
	PHASE28_ALLOW_DIRTY_TEST=1 \
	PHASE28_TEST_HEAD="$source_commit" \
	PHASE28_ADAPTER_TEST_MODE=1 \
	PHASE28_ACCEPTED_STATE_DETECT_BIN="$fake_detect" \
	PHASE28_FAKE_PORT="$test_port" \
	PHASE28_ACCEPTED_STATE_CALL_TRACE="$prevalidated_trace" \
	bash "$runner" run-validated-effect --resume-handle "$prevalidated_handle" --effect-id detector_board_info >"$prevalidated_stdout" 2>"$prevalidated_stderr"
rg -q '^effect_id=detector_board_info$' "$prevalidated_stdout"
rg -q '^effect_status=completed$' "$prevalidated_stdout"
rg -q '^plan13-prevalidated:detector_board_info$' "$prevalidated_trace"
if rg -q "$prevalidated_root|$test_port|credential|capability|resume_handle_sha256|boot_session|raw.log" "$prevalidated_stdout" "$prevalidated_stderr" "$prevalidated_trace"; then
	fail "prevalidated adapter exposed private runner state"
fi

run_adapter_boot_fixture() {
	local fixture_name="$1"
	local boot_platform="$2"
	local state_boot_raw="$3"
	local observed_boot_raw="$4"
	local fixture_root="$temp_root/adapter-boot-$fixture_name"
	local state_path="$fixture_root/state.json"
	local ack="$fixture_root/effect.ack"
	local gate="$fixture_root/effect.gate"
	local result="$fixture_root/effect-result.json"
	local trace="$fixture_root/trace"
	mkdir -m 700 "$fixture_root"
	: >"$trace"
	# shellcheck disable=SC2016
	node --input-type=module -e '
import fs from "node:fs";
const authority = await import(process.argv[1]);
const platform = process.argv[3];
const raw = process.argv[4];
const digest = platform === "darwin" ? authority.deriveMacBootSessionDigest(raw) : authority.deriveLinuxBootSessionDigest(raw);
let state = authority.createAttemptState({exactHead:process.argv[5],resumeHandleSha256:"a".repeat(64),createdMonotonicMs:1,observeBootSession:()=>digest,randomHex:()=>"b".repeat(32)});
state.attempt_state = "connected_entry_waiting";
state = authority.authorizeEffect(state, "detector_board_info", "c".repeat(32));
fs.writeFileSync(process.argv[2], `${JSON.stringify(state)}\n`, {mode:0o600});' \
		"$repo_root/scripts/phase28.1.1-hardware-attempt-state.mjs" "$state_path" "$boot_platform" "$state_boot_raw" "$source_commit"
	env \
		PHASE28_ADAPTER_TEST_MODE=1 \
		PHASE28_ADAPTER_UNIT_FIXTURE=1 \
		PHASE28_ADAPTER_TEST_BOOT_PLATFORM="$boot_platform" \
		PHASE28_ADAPTER_TEST_BOOT_RAW="$observed_boot_raw" \
		PHASE28_LIFECYCLE_ATTEMPT_DIR="$fixture_root" \
		PHASE28_EFFECT_ID=detector_board_info \
		PHASE28_EFFECT_ACK_FILE="$ack" \
		PHASE28_EFFECT_GATE_FILE="$gate" \
		PHASE28_EFFECT_RESULT_FILE="$result" \
		PHASE28_ACCEPTED_STATE_DETECT_BIN="$fake_detect" \
		PHASE28_FAKE_PORT="$test_port" \
		PHASE28_ACCEPTED_STATE_CALL_TRACE="$trace" \
		bash "$script" --mode plan13-prevalidated --effect-id detector_board_info >"$fixture_root/stdout" 2>"$fixture_root/stderr" &
	local adapter_pid=$!
	for _ in $(seq 1 100); do
		[[ -f "$ack" ]] && break
		kill -0 "$adapter_pid" 2>/dev/null || break
		sleep 0.01
	done
	if [[ "$state_boot_raw" != "$observed_boot_raw" ]]; then
		wait "$adapter_pid" 2>/dev/null && fail "mismatched boot fixture unexpectedly succeeded"
		[[ ! -f "$ack" && ! -s "$trace" ]] || fail "mismatched boot fixture crossed the adapter boundary"
		return
	fi
	[[ -f "$ack" ]] || fail "$fixture_name did not acknowledge the adapter boundary"
	jq '.effect_phase="invoked"' "$state_path" >"$state_path.next"
	mv "$state_path.next" "$state_path"
	chmod 600 "$state_path"
	: >"$gate"
	wait "$adapter_pid"
	[[ "$(jq -r '.status' "$result")" == "completed" ]] || fail "$fixture_name did not complete"
}

run_adapter_boot_fixture linux linux '11111111-1111-1111-1111-111111111111' '11111111-1111-1111-1111-111111111111'
run_adapter_boot_fixture macos darwin '{ sec = 123, usec = 456 }' '{ sec = 123, usec = 456 }'
run_adapter_boot_fixture mismatch linux '11111111-1111-1111-1111-111111111111' '22222222-2222-2222-2222-222222222222'

prevalidated_lifecycle_source="$(sed -n '/^run_prevalidated_lifecycle()/,/^}/p' "$script")"
if rg -q 'capability_file|wifi_credentials|pool_credentials|run_flash_capture|run_package|run_verify_reference' <<<"$prevalidated_lifecycle_source"; then
	fail "prevalidated lifecycle exposes credentials or duplicate preflight"
fi
rg -q 'capability="\$\(validate_credential_capability "\$capability_path"\)"' "$script" || fail "flash reinit does not validate the private credential capability"
credential_validator_source="$(sed -n '/^validate_credential_capability()/,/^}/p' "$script")"
for required_check in 'is_private_owned_file' 'exact-head-attempt-v2' 'execution_plan' 'credential_capability_status' 'credential_capability_sha256' 'wifi_credential_state' 'pool_credential_state' 'wifi_credential_binding_id' 'pool_credential_binding_id'; do
	rg -q "$required_check" <<<"$credential_validator_source" || fail "credential capability validation omits $required_check"
done

run_flash_capability_fixture() {
	local case_name="$1"
	local capability_kind="$2"
	local expected_status="$3"
	local state_status="${4:-sealed}"
	local wifi_state="${5:-present}"
	local state_digest_mode="${6:-matching}"
	local capability_mode="${7:-600}"
	local wifi_mode="${8:-600}"
	local fixture_root="$temp_root/capability-$case_name"
	local state_path="$fixture_root/state.json"
	local capability_path="$fixture_root/credential-capability.json"
	local fixture_wifi="$fixture_root/wifi.fixture.json"
	local fixture_pool="$fixture_root/pool.fixture.json"
	local ack="$fixture_root/effect.ack"
	local gate="$fixture_root/effect.gate"
	local result="$fixture_root/effect-result.json"
	local trace="$fixture_root/trace"
	local capture_args="$fixture_root/capture.args"
	local clock_sequence="$fixture_root/clock.sequence"
	local clock_cursor="$fixture_root/clock.cursor"
	local wifi_binding="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
	local pool_binding="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
	local valid_capability
	local capability
	local capability_digest
	local adapter_status

	mkdir -m 700 "$fixture_root"
	printf '%s\n' 'synthetic-wifi-not-read' >"$fixture_wifi"
	printf '%s\n' 'synthetic-pool-not-read' >"$fixture_pool"
	chmod "$wifi_mode" "$fixture_wifi"
	chmod 600 "$fixture_pool"
	valid_capability="$(jq -cn \
		--arg wifi "$fixture_wifi" \
		--arg pool "$fixture_pool" \
		--arg wifi_binding "$wifi_binding" \
		--arg pool_binding "$pool_binding" \
		'{schema_version:"plan13-credential-capability-v1",wifi_path:$wifi,pool_path:$pool,wifi_binding_id:$wifi_binding,pool_binding_id:$pool_binding}')"
	case "$capability_kind" in
	valid) capability="$valid_capability" ;;
	boolean) capability='true' ;;
	null) capability='null' ;;
	array) capability='[]' ;;
	string) capability='"scalar"' ;;
	number) capability='7' ;;
	malformed-json) capability='{' ;;
	missing) capability="$(jq -c 'del(.pool_path)' <<<"$valid_capability")" ;;
	wrong-schema) capability="$(jq -c '.schema_version="plan13-credential-capability-v0"' <<<"$valid_capability")" ;;
	wrong-binding) capability="$(jq -c '.wifi_binding_id=("c"*32)' <<<"$valid_capability")" ;;
	boolean-path) capability="$(jq -c '.wifi_path=true' <<<"$valid_capability")" ;;
	relative-path) capability="$(jq -c '.wifi_path="relative.json"' <<<"$valid_capability")" ;;
	*) fail "unknown capability fixture kind: $capability_kind" ;;
	esac
	printf '%s\n' "$capability" >"$capability_path"
	chmod "$capability_mode" "$capability_path"
	capability_digest="$(shasum -a 256 "$capability_path" | awk '{print $1}')"
	if [[ "$state_digest_mode" == "mismatch" ]]; then
		capability_digest="$(printf 'f%.0s' {1..64})"
	fi
	printf '%s\n' "$test_port" >"$fixture_root/selected-port.value"
	printf '%s\n' "$manifest" >"$fixture_root/package-manifest.path"
	chmod 600 "$fixture_root/selected-port.value" "$fixture_root/package-manifest.path"
	printf '10\n360010\n' >"$clock_sequence"
	: >"$trace"

	# shellcheck disable=SC2016
	node --input-type=module -e '
import fs from "node:fs";
const authority = await import(process.argv[1]);
let state = authority.createAttemptState({
  exactHead: process.argv[3],
  resumeHandleSha256: "d".repeat(64),
  createdMonotonicMs: 1,
  observeBootSession: () => authority.deriveLinuxBootSessionDigest("11111111-1111-1111-1111-111111111111"),
  randomHex: () => "e".repeat(32),
});
state.attempt_state = "packaged";
state.detector_attempt_count = 1;
state.selected_port_state = "one_board205";
state.selected_port_fingerprint_sha256 = "1".repeat(64);
state.wifi_credential_state = process.argv[7];
state.pool_credential_state = "present";
state.wifi_credential_binding_id = process.argv[4];
state.pool_credential_binding_id = process.argv[5];
state.credential_capability_status = process.argv[8];
state.credential_capability_sha256 = process.argv[6];
state.reference_guard_state = "pass";
state.reference_commit = "2".repeat(40);
state.reference_guard_output_sha256 = "3".repeat(64);
state.manifest_state = "pass";
state.manifest_source_commit = process.argv[3];
state.manifest_sha256 = "4".repeat(64);
state.factory_image_sha256 = "5".repeat(64);
state = authority.authorizeEffect(state, "flash_reinit_runtime", "6".repeat(32));
fs.writeFileSync(process.argv[2], `${JSON.stringify(state)}\n`, {mode:0o600});' \
		"$repo_root/scripts/phase28.1.1-hardware-attempt-state.mjs" \
		"$state_path" \
		"$source_commit" \
		"$wifi_binding" \
		"$pool_binding" \
		"$capability_digest" \
		"$wifi_state" \
		"$state_status"

	env \
		PHASE28_ADAPTER_TEST_MODE=1 \
		PHASE28_ADAPTER_UNIT_FIXTURE=1 \
		PHASE28_ADAPTER_TEST_BOOT_PLATFORM=linux \
		PHASE28_ADAPTER_TEST_BOOT_RAW=11111111-1111-1111-1111-111111111111 \
		PHASE28_LIFECYCLE_ATTEMPT_DIR="$fixture_root" \
		PHASE28_EFFECT_ID=flash_reinit_runtime \
		PHASE28_EFFECT_ACK_FILE="$ack" \
		PHASE28_EFFECT_GATE_FILE="$gate" \
		PHASE28_EFFECT_RESULT_FILE="$result" \
		PHASE28_ACCEPTED_STATE_CAPTURE_BIN="$fake_flash_capture" \
		PHASE28_ACCEPTED_STATE_CLOCK_BIN="$fake_clock" \
		PHASE28_FAKE_CAPTURE_ARGS="$capture_args" \
		PHASE28_FAKE_CLOCK_SEQUENCE="$clock_sequence" \
		PHASE28_FAKE_CLOCK_CURSOR="$clock_cursor" \
		PHASE28_ACCEPTED_STATE_CALL_TRACE="$trace" \
		bash "$script" --mode plan13-prevalidated --effect-id flash_reinit_runtime >"$fixture_root/stdout" 2>"$fixture_root/stderr" &
	local adapter_pid=$!
	for _ in $(seq 1 100); do
		[[ -f "$ack" ]] && break
		kill -0 "$adapter_pid" 2>/dev/null || break
		sleep 0.01
	done
	[[ -f "$ack" ]] || fail "$case_name did not acknowledge the adapter boundary"
	jq '.effect_phase="invoked"' "$state_path" >"$state_path.next"
	mv "$state_path.next" "$state_path"
	chmod 600 "$state_path"
	: >"$gate"
	set +e
	wait "$adapter_pid"
	adapter_status=$?
	set -e

	if rg -q 'Cannot index (boolean|null|array|string|number)|jq: error' "$fixture_root/stdout" "$fixture_root/stderr"; then
		fail "$case_name leaked a jq runtime exception"
	fi
	if rg -q 'synthetic-wifi-not-read|synthetic-pool-not-read' "$fixture_root/stdout" "$fixture_root/stderr" "$trace"; then
		fail "$case_name opened or exposed synthetic credential contents"
	fi
	if [[ "$expected_status" == "completed" ]]; then
		[[ "$adapter_status" == "0" ]] || fail "$case_name did not complete"
		[[ "$(jq -r '.status' "$result")" == "completed" ]] || fail "$case_name did not emit a completed result"
		[[ ! -e "$capability_path" ]] || fail "$case_name did not destroy the consumed capability"
		rg -q '^flash-capture$' "$trace"
		[[ "$(awk '/^--wifi-credentials$/{getline; print; exit}' "$capture_args")" == "$fixture_wifi" ]] || fail "$case_name projected the wrong Wi-Fi path"
		[[ "$(awk '/^--pool-credentials$/{getline; print; exit}' "$capture_args")" == "$fixture_pool" ]] || fail "$case_name projected the wrong pool path"
		if rg -q "$fixture_wifi|$fixture_pool|$wifi_binding|$pool_binding|$capability_digest" "$fixture_root/stdout" "$fixture_root/stderr" "$trace"; then
			fail "$case_name exposed private capability data outside the injected consumer"
		fi
		return
	fi
	[[ "$adapter_status" != "0" ]] || fail "$case_name unexpectedly completed"
	[[ "$(jq -r '.status' "$result")" == "failed" ]] || fail "$case_name did not emit a failed result"
	[[ "$(jq -r '.blocker_reason' "$result")" == "private_capability_invalid" ]] || fail "$case_name used the wrong blocker"
	[[ ! -e "$capture_args" ]] || fail "$case_name called the injected flash consumer"
	if rg -q '^flash-capture$' "$trace"; then
		fail "$case_name crossed the flash sentinel"
	fi
}

run_flash_capability_fixture boolean-projection-regression boolean failed
run_flash_capability_fixture null-capability null failed
run_flash_capability_fixture array-capability array failed
run_flash_capability_fixture string-capability string failed
run_flash_capability_fixture number-capability number failed
run_flash_capability_fixture malformed-json malformed-json failed
run_flash_capability_fixture missing-field missing failed
run_flash_capability_fixture wrong-schema wrong-schema failed
run_flash_capability_fixture wrong-binding wrong-binding failed
run_flash_capability_fixture boolean-path boolean-path failed
run_flash_capability_fixture relative-path relative-path failed
run_flash_capability_fixture wrong-state-category valid failed sealed absent
run_flash_capability_fixture wrong-sealed-state valid failed consumed present
run_flash_capability_fixture wrong-digest valid failed sealed present mismatch
run_flash_capability_fixture wrong-capability-mode valid failed sealed present matching 644
run_flash_capability_fixture wrong-credential-mode valid failed sealed present matching 600 644
run_flash_capability_fixture valid-object valid completed

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
