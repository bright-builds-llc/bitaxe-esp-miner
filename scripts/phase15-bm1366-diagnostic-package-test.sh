#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly wrapper="${PHASE15_DIAGNOSTIC_PACKAGE_SCRIPT:-${script_dir}/phase15-bm1366-diagnostic-package.sh}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase15-diagnostic-package-test.XXXXXX")"
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

	write_executable "${bin_dir}/bazel" 'printf "bazel %s\n" "$*" >>"${PHASE15_DIAGNOSTIC_TEST_LOG:?}"
case "$1" in
build)
  mkdir -p "${PHASE15_FAKE_BAZEL_BIN:?}/firmware/bitaxe"
  printf "elf" >"${PHASE15_FAKE_BAZEL_BIN}/firmware/bitaxe/bitaxe-firmware.elf"
  ;;
info)
  if [[ "${2:-}" != "bazel-bin" ]]; then
    printf "unexpected bazel info args: %s\n" "$*" >&2
    exit 2
  fi
  printf "%s\n" "${PHASE15_FAKE_BAZEL_BIN:?}"
  ;;
*)
  printf "unexpected bazel command: %s\n" "$*" >&2
  exit 2
  ;;
esac
'

	write_executable "${bin_dir}/package-firmware.sh" 'printf "package-firmware.sh %s\n" "$*" >>"${PHASE15_DIAGNOSTIC_TEST_LOG:?}"
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

run_wrapper() {
	local mode="$1"
	local out_dir="$2"
	local output_file="$3"
	local log_file="$4"
	local bin_dir="${tmp_root}/bin"
	local bazel_bin="${tmp_root}/bazel-bin"

	create_tool_stubs "$bin_dir"
	: >"$log_file"

	capture_command "$output_file" env \
		PHASE15_DIAGNOSTIC_TEST_LOG="$log_file" \
		PHASE15_FAKE_BAZEL_BIN="$bazel_bin" \
		PHASE15_PACKAGE_FIRMWARE_SCRIPT="package-firmware.sh" \
		PATH="${bin_dir}:${PATH}" \
		"$BASH" "$wrapper" --mode "$mode" --out-dir "$out_dir"
}

test_chip_detect_mode_builds_and_packages() {
	local out_dir="${tmp_root}/chip-detect"
	local output_file="${tmp_root}/chip-detect.out"
	local log_file="${tmp_root}/chip-detect.log"

	if ! run_wrapper "chip-detect" "$out_dir" "$output_file" "$log_file"; then
		printf 'Wrapper output:\n%s\n' "$(cat "$output_file")" >&2
		printf 'Command log:\n%s\n' "$(cat "$log_file")" >&2
		fail "chip-detect mode failed"
	fi

	assert_contains "$log_file" "bazel build --action_env=BITAXE_ASIC_DIAGNOSTIC=chip-detect --action_env=BITAXE_HARDWARE_EVIDENCE_ACK=ultra205-chip-detect-safe-bench //firmware/bitaxe:firmware"
	assert_contains "$log_file" "package-firmware.sh --reference-guard scripts/verify-reference-clean.sh --firmware-elf ${tmp_root}/bazel-bin/firmware/bitaxe/bitaxe-firmware.elf --out-dir ${out_dir}/package --manifest ${out_dir}/package/bitaxe-ultra205-package.json"
	assert_contains "${out_dir}/diagnostic-package-summary.json" '"mode": "chip-detect"'
	assert_contains "${out_dir}/diagnostic-package-summary.json" '"diagnostic_env": "BITAXE_ASIC_DIAGNOSTIC=chip-detect"'
	assert_contains "${out_dir}/diagnostic-package-summary.json" '"hardware_evidence_ack": "BITAXE_HARDWARE_EVIDENCE_ACK=ultra205-chip-detect-safe-bench"'
	assert_contains "${out_dir}/diagnostic-package-summary.json" '"package_manifest":'
	assert_contains "${out_dir}/diagnostic-package-summary.json" '"flash_monitor_command_shape": "bazel run //tools/flash:flash -- flash-monitor --board 205 --port <port> --manifest <out-dir>/package/bitaxe-ultra205-package.json --evidence-dir <evidence-dir> --capture-timeout-seconds <seconds>"'
}

test_work_result_mode_builds_and_packages() {
	local out_dir="${tmp_root}/work-result"
	local output_file="${tmp_root}/work-result.out"
	local log_file="${tmp_root}/work-result.log"

	if ! run_wrapper "work-result" "$out_dir" "$output_file" "$log_file"; then
		printf 'Wrapper output:\n%s\n' "$(cat "$output_file")" >&2
		printf 'Command log:\n%s\n' "$(cat "$log_file")" >&2
		fail "work-result mode failed"
	fi

	assert_contains "$log_file" "bazel build --action_env=BITAXE_ASIC_DIAGNOSTIC=work-result --action_env=BITAXE_HARDWARE_EVIDENCE_ACK=ultra205-work-result-safe-bench //firmware/bitaxe:firmware"
	assert_contains "${out_dir}/diagnostic-package-summary.json" '"mode": "work-result"'
	assert_contains "${out_dir}/diagnostic-package-summary.json" '"diagnostic_env": "BITAXE_ASIC_DIAGNOSTIC=work-result"'
	assert_contains "${out_dir}/diagnostic-package-summary.json" '"hardware_evidence_ack": "BITAXE_HARDWARE_EVIDENCE_ACK=ultra205-work-result-safe-bench"'
}

test_unknown_mode_fails_before_commands() {
	local output_file="${tmp_root}/unknown.out"
	local log_file="${tmp_root}/unknown.log"
	local out_dir="${tmp_root}/unknown"
	local bin_dir="${tmp_root}/bin-unknown"
	local bazel_bin="${tmp_root}/bazel-bin-unknown"

	create_tool_stubs "$bin_dir"
	: >"$log_file"

	set +e
	env \
		PHASE15_DIAGNOSTIC_TEST_LOG="$log_file" \
		PHASE15_FAKE_BAZEL_BIN="$bazel_bin" \
		PHASE15_PACKAGE_FIRMWARE_SCRIPT="package-firmware.sh" \
		PATH="${bin_dir}:${PATH}" \
		"$BASH" "$wrapper" --mode "invalid" --out-dir "$out_dir" >"$output_file" 2>&1
	local status=$?
	set -e

	if [[ "$status" -ne 2 ]]; then
		printf 'Expected exit 2, got %s\n' "$status" >&2
		printf 'Output:\n%s\n' "$(cat "$output_file")" >&2
		exit 1
	fi
	if [[ -s "$log_file" ]]; then
		printf 'Expected no commands for unknown mode, got:\n%s\n' "$(cat "$log_file")" >&2
		exit 1
	fi
}

test_wrapper_avoids_hardware_and_stress_commands() {
	for forbidden in espflash serial erase rollback voltage fan mining-stress raw-write; do
		assert_not_contains "$wrapper" "$forbidden"
	done
}

if [[ ! -f "$wrapper" ]]; then
	fail "wrapper script missing: ${wrapper}"
fi

test_chip_detect_mode_builds_and_packages
test_work_result_mode_builds_and_packages
test_unknown_mode_fails_before_commands
test_wrapper_avoids_hardware_and_stress_commands

printf 'phase15 BM1366 diagnostic package tests passed\n'
