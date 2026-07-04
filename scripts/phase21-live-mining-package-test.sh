#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly wrapper="${PHASE21_LIVE_MINING_PACKAGE_SCRIPT:-${script_dir}/phase21-live-mining-package.sh}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase21-live-mining-package-test.XXXXXX")"
readonly tmp_root

cleanup() {
	rm -rf "$tmp_root"
}
trap cleanup EXIT

fail() {
	printf 'FAIL: %s\n' "$1" >&2
	exit 1
}

assert_contains() {
	local path="$1"
	local needle="$2"

	if ! grep -Fq -- "$needle" "$path"; then
		printf 'Expected %s to contain: %s\n' "$path" "$needle" >&2
		printf 'Actual content:\n%s\n' "$(cat "$path")" >&2
		exit 1
	fi
}

assert_not_contains() {
	local path="$1"
	local needle="$2"

	if grep -Fq -- "$needle" "$path"; then
		printf 'Expected %s not to contain: %s\n' "$path" "$needle" >&2
		printf 'Actual content:\n%s\n' "$(cat "$path")" >&2
		exit 1
	fi
}

capture_command() {
	local output_file="$1"
	shift

	set +e
	"$@" >"$output_file" 2>&1
	local status=$?
	set -e

	return "$status"
}

write_executable() {
	local path="$1"
	local body="$2"

	printf '#!%s\n%s\n' "$BASH" "$body" >"$path"
	chmod +x "$path"
}

create_tool_stubs() {
	local bin_dir="$1"
	mkdir -p "$bin_dir"

	write_executable "${bin_dir}/cargo" 'printf "cargo %s\n" "$*" >>"${PHASE21_PACKAGE_TEST_LOG:?}"
'

	write_executable "${bin_dir}/bazel" 'printf "bazel %s\n" "$*" >>"${PHASE21_PACKAGE_TEST_LOG:?}"
case "$1" in
build)
  mkdir -p "${PHASE21_FAKE_BAZEL_BIN:?}/firmware/bitaxe"
  printf "elf" >"${PHASE21_FAKE_BAZEL_BIN}/firmware/bitaxe/bitaxe-firmware.elf"
  ;;
info)
  if [[ "${2:-}" != "bazel-bin" ]]; then
    printf "unexpected bazel info args: %s\n" "$*" >&2
    exit 2
  fi
  printf "%s\n" "${PHASE21_FAKE_BAZEL_BIN:?}"
  ;;
*)
  printf "unexpected bazel command: %s\n" "$*" >&2
  exit 2
  ;;
esac
'

	write_executable "${bin_dir}/package-firmware.sh" 'printf "package-firmware.sh %s\n" "$*" >>"${PHASE21_PACKAGE_TEST_LOG:?}"
manifest=""
previous=""
for arg in "$@"; do
  if [[ "$previous" == "--manifest" ]]; then
    manifest="$arg"
  fi
  previous="$arg"
done
if [[ -z "$manifest" ]]; then
  printf "missing manifest argument\n" >&2
  exit 2
fi
mkdir -p "$(dirname "$manifest")"
printf "{\"source_commit\":\"source-abc\",\"reference_commit\":\"reference-def\"}\n" >"$manifest"
'
}

write_valid_readiness_audit() {
	local path="$1"

	cat >"$path" <<'AUDIT'
firmware_live_mining_status: blocked_by_default
controlled_enablement_required: true
AUDIT
}

write_invalid_readiness_audit() {
	local path="$1"

	cat >"$path" <<'AUDIT'
firmware_live_mining_status: ready_without_blocker
controlled_enablement_required: false
AUDIT
}

run_wrapper() {
	local out_dir="$1"
	local readiness_audit="$2"
	local output_file="$3"
	local log_file="$4"
	local bin_dir="${tmp_root}/bin"
	local bazel_bin="${tmp_root}/bazel-bin"

	create_tool_stubs "$bin_dir"
	: >"$log_file"

	capture_command "$output_file" env \
		PHASE21_PACKAGE_TEST_LOG="$log_file" \
		PHASE21_FAKE_BAZEL_BIN="$bazel_bin" \
		PHASE21_PACKAGE_FIRMWARE_SCRIPT="package-firmware.sh" \
		PATH="${bin_dir}:${PATH}" \
		"$BASH" "$wrapper" --out-dir "$out_dir" --readiness-audit "$readiness_audit"
}

test_missing_readiness_audit_blocks_before_bazel() {
	local out_dir="${tmp_root}/missing-readiness"
	local output_file="${tmp_root}/missing-readiness.out"
	local log_file="${tmp_root}/missing-readiness.log"
	local readiness_audit="${tmp_root}/missing-readiness.md"

	if run_wrapper "$out_dir" "$readiness_audit" "$output_file" "$log_file"; then
		fail "missing readiness audit should block"
	fi

	assert_contains "${out_dir}.md" "controlled_live_mining_package_status: blocked"
	assert_contains "${out_dir}.md" "controlled_runtime_harness_status: blocked"
	assert_contains "${out_dir}.md" "blocker: missing readiness audit"
	if [[ -s "$log_file" ]]; then
		printf 'Expected no commands before readiness gate, got:\n%s\n' "$(cat "$log_file")" >&2
		exit 1
	fi
}

test_invalid_readiness_audit_blocks_before_bazel() {
	local out_dir="${tmp_root}/invalid-readiness"
	local output_file="${tmp_root}/invalid-readiness.out"
	local log_file="${tmp_root}/invalid-readiness.log"
	local readiness_audit="${tmp_root}/invalid-readiness.md"

	write_invalid_readiness_audit "$readiness_audit"

	if run_wrapper "$out_dir" "$readiness_audit" "$output_file" "$log_file"; then
		fail "invalid readiness audit should block"
	fi

	assert_contains "${out_dir}.md" "controlled_live_mining_package_status: blocked"
	assert_contains "${out_dir}.md" "controlled_runtime_harness_status: blocked"
	assert_contains "${out_dir}.md" "blocker: readiness audit missing controlled enablement markers"
	if [[ -s "$log_file" ]]; then
		printf 'Expected no commands before readiness gate, got:\n%s\n' "$(cat "$log_file")" >&2
		exit 1
	fi
}

test_valid_readiness_builds_packages_and_writes_ready_ledger() {
	local out_dir="${tmp_root}/ready"
	local output_file="${tmp_root}/ready.out"
	local log_file="${tmp_root}/ready.log"
	local readiness_audit="${tmp_root}/ready-readiness.md"

	write_valid_readiness_audit "$readiness_audit"

	if ! run_wrapper "$out_dir" "$readiness_audit" "$output_file" "$log_file"; then
		printf 'Wrapper output:\n%s\n' "$(cat "$output_file")" >&2
		printf 'Command log:\n%s\n' "$(cat "$log_file")" >&2
		fail "valid readiness path failed"
	fi

	assert_contains "$log_file" "cargo test -p bitaxe-stratum --all-features controlled_runtime"
	assert_contains "$log_file" "bazel build --action_env=BITAXE_MINING_EVIDENCE_MODE=live-mining-runtime --action_env=BITAXE_HARDWARE_EVIDENCE_ACK=ultra205-live-mining-runtime-safe-bench //firmware/bitaxe:firmware"
	assert_contains "$log_file" "package-firmware.sh --reference-guard scripts/verify-reference-clean.sh --firmware-elf ${tmp_root}/bazel-bin/firmware/bitaxe/bitaxe-firmware.elf --out-dir ${out_dir}/package --manifest ${out_dir}/package/bitaxe-ultra205-package.json"
	assert_contains "${out_dir}.md" "controlled_live_mining_package_status: ready"
	assert_contains "${out_dir}.md" "controlled_runtime_harness_status: ready"
	assert_contains "${out_dir}.md" "controlled_runtime_contract_tests: passed"
	assert_contains "${out_dir}.md" "runtime_required_log_markers: phase21_controlled_runtime_status, stratum_subscribe_status, stratum_authorize_status, stratum_notify_status, bm1366_work_dispatch_status, result_receive_status, share_submission_status, runtime_snapshot_status, api_websocket_telemetry_update_status, safe_stop_status"
	assert_contains "${out_dir}.md" "redaction_reviewer: required-before-citation"
	assert_contains "${out_dir}.md" "source_commit: source-abc"
	assert_contains "${out_dir}.md" "reference_commit: reference-def"
}

test_enablement_ledger_excludes_raw_secret_or_target_values() {
	local out_dir="${tmp_root}/redaction"
	local output_file="${tmp_root}/redaction.out"
	local log_file="${tmp_root}/redaction.log"
	local readiness_audit="${tmp_root}/redaction-readiness.md"
	local ledger="${out_dir}.md"

	write_valid_readiness_audit "$readiness_audit"

	if ! run_wrapper "$out_dir" "$readiness_audit" "$output_file" "$log_file"; then
		fail "valid readiness path failed for redaction test"
	fi

	for forbidden in password credential token device_url "192.168." "10.0." "aa:bb:cc" "private."; do
		assert_not_contains "$ledger" "$forbidden"
	done
}

if [[ ! -f "$wrapper" ]]; then
	fail "wrapper script missing: ${wrapper}"
fi

test_missing_readiness_audit_blocks_before_bazel
test_invalid_readiness_audit_blocks_before_bazel
test_valid_readiness_builds_packages_and_writes_ready_ledger
test_enablement_ledger_excludes_raw_secret_or_target_values

printf 'phase21 live mining package tests passed\n'
