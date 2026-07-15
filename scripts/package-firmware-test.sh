#!/usr/bin/env bash
# shellcheck disable=SC2016 # Stub bodies must expand only when the generated scripts execute.
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
if [[ " $* " == *" image_info "* ]]; then
  printf "App version: 0123456789ab-dev\n"
  printf "ELF file SHA256: 1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef\n"
  exit 0
fi
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

test_package_script_uses_managed_esptool_images() {
	local bin_dir="${tmp_root}/bin"
	local idf_path="${tmp_root}/idf"
	local out_dir="${tmp_root}/out"
	local firmware_elf="${tmp_root}/firmware.elf"
	local build_provenance_stamp="${tmp_root}/build-provenance.stamp"
	local generated_idf_build_dir="${tmp_root}/generated-idf"
	local reference_guard="${tmp_root}/verify-reference-clean.sh"
	local output_file="${tmp_root}/package.out"
	local log_file="${tmp_root}/commands.log"

	create_tool_stubs "$bin_dir"
	create_reference_guard "$reference_guard"
	mkdir -p \
		"${idf_path}/components/spiffs" \
		"${generated_idf_build_dir}/build/bootloader" \
		"${generated_idf_build_dir}/build/partition_table" \
		"$out_dir"
	touch "${idf_path}/components/spiffs/spiffsgen.py"
	printf 'elf' >"$firmware_elf"
	printf 'build_label=0123456789ab-dev\n' >"$build_provenance_stamp"
	printf 'CONFIG_APP_PROJECT_VER="0123456789ab-dev"\nCONFIG_APP_RETRIEVE_LEN_ELF_SHA=64\n' >"${generated_idf_build_dir}/sdkconfig"
	printf 'bootloader' >"${generated_idf_build_dir}/build/bootloader/bootloader.bin"
	printf 'partitions' >"${generated_idf_build_dir}/build/partition_table/partition-table.bin"
	printf 'otadata' >"${generated_idf_build_dir}/build/ota_data_initial.bin"
	: >"$log_file"

	if ! capture_command "$output_file" env \
		HOME="${tmp_root}/home" \
		IDF_PATH="$idf_path" \
		ESP_IDF_BUILD_DIR="$generated_idf_build_dir" \
		PACKAGE_FIRMWARE_TEST_LOG="$log_file" \
		PATH="${bin_dir}:${PATH}" \
		"$BASH" "$package_script" \
		--reference-guard "$reference_guard" \
		--firmware-elf "$firmware_elf" \
		--build-provenance-stamp "$build_provenance_stamp" \
		--out-dir "$out_dir" \
		--manifest "${out_dir}/bitaxe-ultra205-package.json"; then
		printf 'Package output:\n%s\n' "$(cat "$output_file")" >&2
		printf 'Command log:\n%s\n' "$(cat "$log_file")" >&2
		fail "package script failed with stubbed tools"
	fi

	local output
	output="$(cat "$output_file")"$'\n'"$(cat "$log_file")"
	assert_contains "$output" "[package-firmware] firmware_ota_command="
	assert_contains "$output" "[package-firmware] factory_merge_command="
	assert_contains "$output" "--obj-name-len 64"
	assert_contains "$output" "esptool.py --chip esp32s3 merge_bin"
	assert_contains "$output" "esptool.py image_info --version 2"
	assert_contains "$output" "esptool.py --chip esp32s3 elf2image"
	assert_contains "$output" "--elf-sha256-offset 0xb0"
	assert_contains "$output" "--build-provenance-stamp"
	assert_contains "$output" "--app-descriptor-version 0123456789ab-dev"
	assert_contains "$output" "--app-elf-sha256 1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
	assert_contains "$output" "0x0"
	assert_contains "$output" "0x8000"
	assert_contains "$output" "0x10000"
	assert_contains "$output" "0x410000"
	assert_contains "$output" "0xf10000"
	assert_contains "$output" "bitaxe-ultra205-factory.bin"
	assert_not_contains "$output" "overlay-factory-payloads"
}

if [[ ! -f "$package_script" ]]; then
	fail "package script missing: ${package_script}"
fi

cd "$workspace_dir"
test_package_script_uses_managed_esptool_images

printf 'package firmware tests passed\n'
