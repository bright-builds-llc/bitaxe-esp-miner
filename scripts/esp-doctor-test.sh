#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly doctor_script="${ESP_DOCTOR_SCRIPT:-${script_dir}/esp-doctor.sh}"
readonly bootstrap_script="${BOOTSTRAP_ESP_SCRIPT:-${script_dir}/bootstrap-esp.sh}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/esp-doctor-test.XXXXXX")"
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

create_workspace() {
	local workspace="$1"

	mkdir -p "${workspace}/.cargo" "${workspace}/firmware/bitaxe"
	cat >"${workspace}/.cargo/config.toml" <<'CONFIG'
[target.xtensa-esp32s3-espidf]
linker = "ldproxy"

[env]
MCU = "esp32s3"
ESP_IDF_TOOLS_INSTALL_DIR = "workspace"
ESP_IDF_VERSION = "tag:v5.5.4"
CONFIG

	cat >"${workspace}/firmware/bitaxe/Cargo.toml" <<'CARGO'
[package]
name = "bitaxe-firmware"

[package.metadata.esp-idf-sys]
esp_idf_version = "tag:v5.5.4"
esp_idf_tools_install_dir = "workspace"
CARGO
}

create_common_tool_stubs() {
	local bin_dir="$1"

	mkdir -p "$bin_dir"
	write_executable "${bin_dir}/cargo" 'if [[ "${1:-}" == "--version" ]]; then
  echo "cargo 1.0.0"
  exit 0
fi
echo "cargo stub $*"
'
	write_executable "${bin_dir}/rustup" 'if [[ "${1:-}" == "--version" ]]; then
  echo "rustup 1.0.0"
  exit 0
fi
if [[ "${1:-}" == "toolchain" && "${2:-}" == "list" ]]; then
  echo "esp"
  exit 0
fi
echo "rustup stub $*"
'
	write_executable "${bin_dir}/ldproxy" 'echo "ldproxy 1.0.0"
'
	write_executable "${bin_dir}/espflash" 'echo "espflash 1.0.0"
'
	write_executable "${bin_dir}/python3" 'echo "Python 3.12.0"
'
	write_executable "${bin_dir}/bazel" 'echo "bazel 9.1.1"
'
}

test_doctor_missing_tools_fails_with_bootstrap_hint() {
	local workspace="${tmp_root}/missing-workspace"
	local home_dir="${tmp_root}/missing-home"
	local bin_dir="${tmp_root}/missing-bin"
	local output_file="${tmp_root}/missing.out"

	create_workspace "$workspace"
	mkdir -p "$home_dir" "$bin_dir"

	if capture_command "$output_file" env HOME="$home_dir" PATH="$bin_dir" WORKSPACE_DIR="$workspace" "$BASH" "$doctor_script"; then
		fail "doctor unexpectedly passed with missing tools"
	fi

	local output
	output="$(cat "$output_file")"
	assert_contains "$output" "missing: cargo"
	assert_contains "$output" "just bootstrap-esp"
}

test_doctor_passes_with_stubbed_tools() {
	local workspace="${tmp_root}/clean-workspace"
	local home_dir="${tmp_root}/clean-home"
	local bin_dir="${tmp_root}/clean-bin"
	local output_file="${tmp_root}/clean.out"

	create_workspace "$workspace"
	create_common_tool_stubs "$bin_dir"
	mkdir -p "$home_dir"
	printf 'export ESP_STUB=1\n' >"${home_dir}/export-esp.sh"

	if ! capture_command "$output_file" env HOME="$home_dir" PATH="$bin_dir" WORKSPACE_DIR="$workspace" "$BASH" "$doctor_script"; then
		printf 'Doctor output:\n%s\n' "$(cat "$output_file")" >&2
		fail "doctor failed with stubbed tools"
	fi

	assert_contains "$(cat "$output_file")" "ESP-IDF dependency check passed"
}

test_doctor_reports_managed_embuild_tools() {
	local workspace="${tmp_root}/embuild-workspace"
	local home_dir="${tmp_root}/embuild-home"
	local bin_dir="${tmp_root}/embuild-bin"
	local output_file="${tmp_root}/embuild.out"

	create_workspace "$workspace"
	create_common_tool_stubs "$bin_dir"
	mkdir -p "$home_dir"
	printf 'export ESP_STUB=1\n' >"${home_dir}/export-esp.sh"
	mkdir -p \
		"${workspace}/.embuild/espressif/esp-idf/v5.5.4/components/spiffs" \
		"${workspace}/.embuild/espressif/esp-idf/v5.5.4/components/partition_table" \
		"${workspace}/.embuild/espressif/python_env/idf5.5_py3.12_env/bin"
	touch \
		"${workspace}/.embuild/espressif/esp-idf/v5.5.4/components/spiffs/spiffsgen.py" \
		"${workspace}/.embuild/espressif/esp-idf/v5.5.4/components/partition_table/gen_esp32part.py" \
		"${workspace}/.embuild/espressif/python_env/idf5.5_py3.12_env/bin/esptool.py"

	if ! capture_command "$output_file" env HOME="$home_dir" PATH="$bin_dir" WORKSPACE_DIR="$workspace" "$BASH" "$doctor_script"; then
		printf 'Doctor output:\n%s\n' "$(cat "$output_file")" >&2
		fail "doctor failed with managed .embuild tool stubs"
	fi

	local output
	output="$(cat "$output_file")"
	assert_contains "$output" "managed spiffsgen.py"
	assert_contains "$output" "managed gen_esp32part.py"
	assert_contains "$output" "managed esptool.py"
}

test_bootstrap_dry_run_prints_install_actions() {
	local home_dir="${tmp_root}/bootstrap-home"
	local bin_dir="${tmp_root}/bootstrap-bin"
	local output_file="${tmp_root}/bootstrap.out"

	mkdir -p "$home_dir" "$bin_dir"
	write_executable "${bin_dir}/cargo" 'if [[ "${1:-}" == "--version" ]]; then
  echo "cargo 1.0.0"
  exit 0
fi
echo "cargo stub $*"
'
	write_executable "${bin_dir}/rustup" 'if [[ "${1:-}" == "toolchain" && "${2:-}" == "list" ]]; then
  echo "stable-aarch64-apple-darwin"
  exit 0
fi
echo "rustup stub $*"
'

	if ! capture_command "$output_file" env HOME="$home_dir" PATH="$bin_dir" "$BASH" "$bootstrap_script" --dry-run; then
		printf 'Bootstrap output:\n%s\n' "$(cat "$output_file")" >&2
		fail "bootstrap dry-run failed"
	fi

	local output
	output="$(cat "$output_file")"
	assert_contains "$output" "cargo install espup --locked"
	assert_contains "$output" "espup install --targets esp32s3 --std"
	assert_contains "$output" "cargo install espflash --locked"
}

if [[ ! -f "$doctor_script" ]]; then
	fail "doctor script missing: ${doctor_script}"
fi

if [[ ! -f "$bootstrap_script" ]]; then
	fail "bootstrap script missing: ${bootstrap_script}"
fi

test_doctor_missing_tools_fails_with_bootstrap_hint
test_doctor_passes_with_stubbed_tools
test_doctor_reports_managed_embuild_tools
test_bootstrap_dry_run_prints_install_actions

printf 'esp doctor tests passed\n'
