#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly package_script="${PACKAGE_FIRMWARE_SCRIPT:-${script_dir}/package-firmware.sh}"

workspace_dir="$(git -C "${script_dir}/.." rev-parse --show-toplevel)"
readonly workspace_dir

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/package-firmware-test.XXXXXX")"
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
	local haystack="$1"
	local needle="$2"

	case "$haystack" in
	*"$needle"*) ;;
	*)
		printf 'Expected output to contain: %s\n' "$needle" >&2
		printf 'Actual output:\n%s\n' "$haystack" >&2
		exit 1
		;;
	esac
}

assert_not_contains() {
	local haystack="$1"
	local needle="$2"

	case "$haystack" in
	*"$needle"*)
		printf 'Expected output not to contain: %s\n' "$needle" >&2
		printf 'Actual output:\n%s\n' "$haystack" >&2
		exit 1
		;;
	*) ;;
	esac
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

	write_executable "${bin_dir}/python3" 'if [[ "${1:-}" == "-c" ]]; then
  output="${@: -1}"
  printf "otadata" >"$output"
  exit 0
fi
output="${@: -1}"
printf "www-image" >"$output"
'
	write_executable "${bin_dir}/espflash" 'printf "espflash %s\n" "$*" >>"${PACKAGE_FIRMWARE_TEST_LOG:?}"
output="${@: -1}"
printf "base-image" >"$output"
'
	write_executable "${bin_dir}/esptool.py" 'printf "esptool.py %s\n" "$*" >>"${PACKAGE_FIRMWARE_TEST_LOG:?}"
output=""
previous=""
for arg in "$@"; do
  if [[ "$previous" == "-o" || "$previous" == "--output" ]]; then
    output="$arg"
  fi
  previous="$arg"
done
if [[ -z "$output" ]]; then
  printf "missing esptool output\n" >&2
  exit 1
fi
printf "factory-image" >"$output"
'
	write_executable "${bin_dir}/cargo" 'printf "cargo %s\n" "$*" >>"${PACKAGE_FIRMWARE_TEST_LOG:?}"
case " $* " in
*" overlay-factory-payloads "*)
  printf "overlay-factory-payloads must not be invoked\n" >&2
  exit 1
  ;;
esac
manifest=""
previous=""
for arg in "$@"; do
  if [[ "$previous" == "--manifest" ]]; then
    manifest="$arg"
  fi
  previous="$arg"
done
if [[ -n "$manifest" ]]; then
  printf "{}\n" >"$manifest"
fi
'
}

create_reference_guard() {
	local path="$1"
	write_executable "$path" 'printf "reference clean\n"'
}

test_package_script_uses_esptool_merge_bin() {
	local bin_dir="${tmp_root}/bin"
	local idf_path="${tmp_root}/idf"
	local out_dir="${tmp_root}/out"
	local firmware_elf="${tmp_root}/firmware.elf"
	local reference_guard="${tmp_root}/verify-reference-clean.sh"
	local output_file="${tmp_root}/package.out"
	local log_file="${tmp_root}/commands.log"

	create_tool_stubs "$bin_dir"
	create_reference_guard "$reference_guard"
	mkdir -p "${idf_path}/components/spiffs" "$out_dir"
	touch "${idf_path}/components/spiffs/spiffsgen.py"
	printf 'elf' >"$firmware_elf"
	: >"$log_file"

	if ! capture_command "$output_file" env \
		HOME="${tmp_root}/home" \
		IDF_PATH="$idf_path" \
		PACKAGE_FIRMWARE_TEST_LOG="$log_file" \
		PATH="${bin_dir}:${PATH}" \
		"$BASH" "$package_script" \
		--reference-guard "$reference_guard" \
		--firmware-elf "$firmware_elf" \
		--out-dir "$out_dir" \
		--manifest "${out_dir}/bitaxe-ultra205-package.json"; then
		printf 'Package output:\n%s\n' "$(cat "$output_file")" >&2
		printf 'Command log:\n%s\n' "$(cat "$log_file")" >&2
		fail "package script failed with stubbed tools"
	fi

	local output
	output="$(cat "$output_file")"$'\n'"$(cat "$log_file")"
	assert_contains "$output" "[package-firmware] factory_base_command="
	assert_contains "$output" "[package-firmware] factory_merge_command="
	assert_contains "$output" "--skip-padding"
	assert_contains "$output" "esptool.py --chip esp32s3 merge_bin"
	assert_contains "$output" "0x0"
	assert_contains "$output" "0x410000"
	assert_contains "$output" "0xf10000"
	assert_contains "$output" "bitaxe-ultra205-factory.bin"
	assert_not_contains "$output" "overlay-factory-payloads"
}

if [[ ! -f "$package_script" ]]; then
	fail "package script missing: ${package_script}"
fi

cd "$workspace_dir"
test_package_script_uses_esptool_merge_bin

printf 'package firmware tests passed\n'
