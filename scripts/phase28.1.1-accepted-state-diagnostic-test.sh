#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
script="$repo_root/scripts/phase28.1.1-accepted-state-diagnostic.sh"
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
expect_failure bash "$script" --mode hardware --attempt accepted-state --duration-seconds 360 --port /dev/test --wifi-credentials "$temp_root/missing-wifi" --pool-credentials "$pool" --evidence-out "$temp_root/missing.md"

bash "$script" --mode blocked --attempt lifecycle --duration-seconds 360 --evidence-out "$temp_root/blocked.md" >"$temp_root/blocked.stdout"
rg -q '^capture_status: blocked_safe_prerequisite$' "$temp_root/blocked.md"
rg -q '^accepted_state_status: unavailable$' "$temp_root/blocked.md"

fake_verify="$temp_root/fake-verify.sh"
fake_detect="$temp_root/fake-detect.sh"
fake_package="$temp_root/fake-package.sh"
fake_capture="$temp_root/fake-capture.sh"
manifest="$temp_root/package.json"
printf '#!/usr/bin/env bash\nexit 0\n' >"$fake_verify"
printf '#!/usr/bin/env bash\nprintf "port=/dev/ultra205-test\\n"\n' >"$fake_detect"
printf '#!/usr/bin/env bash\nprintf "%%s\\n" "$@" >"%s"\n' "$temp_root/package.args" >"$fake_package"
{
	printf '#!/usr/bin/env bash\nset -euo pipefail\n'
	printf 'printf "%%s\\n" "$@" >"%s"\n' "$temp_root/capture.args"
	# shellcheck disable=SC2016
	printf 'while (($#)); do if [[ "$1" == "--evidence-root" ]]; then root="$2"; shift 2; else shift; fi; done\n'
	# shellcheck disable=SC2016
	printf 'mkdir -p "$root/live-capture-runtime"\n'
	# shellcheck disable=SC2016
	printf 'printf "accepted_state_snapshot stage=post_first_work observation=available chip_count_class=match readable_responses=7 error_counter_active=false domain_counter_active=false total_counter_active=true power_delta_class=rising_hashing result_correlated=true submit_observed=true redacted=true\\n" >"$root/live-capture-runtime/flash-monitor.log"\n'
	printf 'exit 0\n'
} >"$fake_capture"
chmod +x "$fake_verify" "$fake_detect" "$fake_package" "$fake_capture"
printf '{}\n' >"$manifest"

upstream_log="$temp_root/upstream.log"
printf 'accepted_state_snapshot stage=post_first_work observation=available chip_count_class=match readable_responses=7 error_counter_active=false domain_counter_active=false total_counter_active=true power_delta_class=rising_hashing result_correlated=false submit_observed=false redacted=true\n' >"$upstream_log"

expect_failure env PHASE28_ACCEPTED_STATE_DETECT_BIN="$fake_detect" PHASE28_ACCEPTED_STATE_VERIFY_BIN="$fake_verify" PHASE28_ACCEPTED_STATE_RUN_ROOT="$temp_root/runs-mismatch" bash "$script" --mode hardware --attempt accepted-state --duration-seconds 360 --port /dev/wrong --wifi-credentials "$wifi" --pool-credentials "$pool" --evidence-out "$temp_root/mismatch.md"

env \
	PHASE28_ACCEPTED_STATE_DETECT_BIN="$fake_detect" \
	PHASE28_ACCEPTED_STATE_VERIFY_BIN="$fake_verify" \
	PHASE28_ACCEPTED_STATE_PACKAGE_BIN="$fake_package" \
	PHASE28_ACCEPTED_STATE_CAPTURE_BIN="$fake_capture" \
	PHASE28_ACCEPTED_STATE_MANIFEST="$manifest" \
	PHASE28_ACCEPTED_STATE_UPSTREAM_LOG="$upstream_log" \
	PHASE28_ACCEPTED_STATE_RUN_ROOT="$temp_root/runs" \
	bash "$script" --mode hardware --attempt accepted-state --duration-seconds 360 --port /dev/ultra205-test --wifi-credentials "$wifi" --pool-credentials "$pool" --evidence-out "$temp_root/hardware.md" >"$temp_root/hardware.stdout"

rg -q '^accepted_state_snapshot$' "$temp_root/package.args"
rg -q '^--mode$' "$temp_root/capture.args"
rg -q '^hardware$' "$temp_root/capture.args"
rg -q '^--duration-seconds$' "$temp_root/capture.args"
rg -q '^360$' "$temp_root/capture.args"
rg -q '^--redact-evidence=true$' "$temp_root/capture.args"
rg -q '^board: 205$' "$temp_root/hardware.md"
rg -q '^recommended_investigation: none$' "$temp_root/hardware.md"

if rg -q 'wifi-secret-sentinel|pool-secret-sentinel' "$temp_root/hardware.md" "$temp_root/hardware.stdout"; then
	fail "credential contents escaped into redacted output"
fi
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
