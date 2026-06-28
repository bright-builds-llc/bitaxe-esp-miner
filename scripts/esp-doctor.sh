#!/usr/bin/env bash
set -euo pipefail

readonly EXPECTED_TARGET="xtensa-esp32s3-espidf"
readonly EXPECTED_MCU="esp32s3"
readonly EXPECTED_IDF_VERSION="tag:v5.5.4"
readonly EXPECTED_IDF_DIR="workspace"
readonly EXPECTED_IDF_SOURCE_VERSION="v5.5.4"

failures=0
warnings=0

usage() {
	printf 'usage: %s\n' "$0" >&2
	printf 'Checks local ESP-IDF/Rust firmware prerequisites without installing anything.\n' >&2
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
	usage
	exit 0
fi

if [[ "$#" -ne 0 ]]; then
	usage
	exit 2
fi

detect_workspace_dir() {
	if [[ -n "${WORKSPACE_DIR:-}" ]]; then
		printf '%s\n' "$WORKSPACE_DIR"
		return 0
	fi

	local maybe_root
	if maybe_root="$(git rev-parse --show-toplevel 2>/dev/null)"; then
		printf '%s\n' "$maybe_root"
		return 0
	fi

	cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd
}

if [[ -z "${HOME:-}" ]]; then
	HOME="$(cd ~ && pwd)"
	export HOME
fi

if [[ -d "${HOME}/.cargo/bin" ]]; then
	PATH="${HOME}/.cargo/bin:${PATH}"
	export PATH
fi

readonly workspace_dir="$(detect_workspace_dir)"
readonly cargo_config="${workspace_dir}/.cargo/config.toml"
readonly firmware_manifest="${workspace_dir}/firmware/bitaxe/Cargo.toml"
readonly esp_export="${HOME}/export-esp.sh"

pass() {
	printf 'ok: %s\n' "$1"
}

info() {
	printf 'info: %s\n' "$1"
}

warn() {
	warnings=$((warnings + 1))
	printf 'warning: %s\n' "$1" >&2
}

fail() {
	failures=$((failures + 1))
	printf 'missing: %s\n' "$1" >&2
	printf '  fix: %s\n' "$2" >&2
}

tool_version() {
	local tool="$1"
	local output
	if output="$("$tool" --version 2>/dev/null)"; then
		output="${output%%$'\n'*}"
		printf '%s\n' "$output"
		return 0
	fi

	return 1
}

check_tool() {
	local tool="$1"
	local fix="$2"

	if ! command -v "$tool" >/dev/null 2>&1; then
		fail "$tool" "$fix"
		return
	fi

	local maybe_version
	if maybe_version="$(tool_version "$tool")"; then
		pass "${tool}: ${maybe_version}"
		return
	fi

	pass "$tool found"
}

file_contains() {
	local path="$1"
	local expected="$2"

	[[ -f "$path" ]] || return 1
	local content
	content="$(<"$path")"
	[[ "$content" == *"$expected"* ]]
}

check_file_contains() {
	local path="$1"
	local expected="$2"
	local label="$3"
	local fix="$4"

	if file_contains "$path" "$expected"; then
		pass "$label"
		return
	fi

	fail "$label" "$fix"
}

check_esp_toolchain() {
	if ! command -v rustup >/dev/null 2>&1; then
		fail "rustup" "Install Rust with rustup, then run: just bootstrap-esp"
		return
	fi

	local toolchains
	if ! toolchains="$(rustup toolchain list 2>/dev/null)"; then
		fail "rustup toolchain list" "Repair rustup, then run: just bootstrap-esp"
		return
	fi

	case "$toolchains" in
	*esp*) pass "rustup esp toolchain installed" ;;
	*) fail "rustup esp toolchain" "Run: just bootstrap-esp" ;;
	esac
}

check_esp_export() {
	if [[ -f "$esp_export" ]]; then
		pass "ESP export script: ${esp_export}"
		return
	fi

	fail "ESP export script: ${esp_export}" "Run: just bootstrap-esp"
}

check_optional_idf_path() {
	if [[ -z "${IDF_PATH:-}" ]]; then
		info "IDF_PATH is not set; canonical path is esp-idf-sys managed ${EXPECTED_IDF_VERSION}"
		return
	fi

	if [[ -f "${IDF_PATH}/components/spiffs/spiffsgen.py" ]]; then
		pass "optional IDF_PATH has spiffsgen.py"
		return
	fi

	warn "IDF_PATH is set but ${IDF_PATH}/components/spiffs/spiffsgen.py is missing"
}

check_managed_embuild_file() {
	local label="$1"
	local fix="$2"
	shift 2

	local candidate
	for candidate in "$@"; do
		if [[ -f "$candidate" ]]; then
			pass "${label}: ${candidate}"
			return
		fi
	done

	warn "${label} not found under .embuild; ${fix}"
}

check_managed_embuild_tools() {
	local embuild_dir="${workspace_dir}/.embuild"
	if [[ ! -d "$embuild_dir" ]]; then
		info ".embuild is not present yet; the first ESP-IDF firmware build creates the local managed tool workspace"
		return
	fi

	check_managed_embuild_file \
		"managed spiffsgen.py" \
		"run just build or just package to populate ESP-IDF managed tools" \
		"${embuild_dir}/espressif/esp-idf/${EXPECTED_IDF_SOURCE_VERSION}/components/spiffs/spiffsgen.py"
	check_managed_embuild_file \
		"managed gen_esp32part.py" \
		"run just build or just package to populate ESP-IDF managed tools" \
		"${embuild_dir}/espressif/esp-idf/${EXPECTED_IDF_SOURCE_VERSION}/components/partition_table/gen_esp32part.py"
	check_managed_embuild_file \
		"managed esptool.py" \
		"run just build or just package to populate ESP-IDF managed tools" \
		"${embuild_dir}"/espressif/python_env/idf5.5*_env/bin/esptool.py \
		"${embuild_dir}/espressif/esp-idf/${EXPECTED_IDF_SOURCE_VERSION}/components/esptool_py/esptool/esptool.py"
}

printf 'ESP-IDF contributor dependency doctor\n'
printf 'workspace: %s\n' "$workspace_dir"

check_tool cargo "Install Rust/Cargo from https://rustup.rs, then run: just bootstrap-esp"
check_tool rustup "Install Rust with rustup from https://rustup.rs"
check_esp_toolchain
check_esp_export
check_tool ldproxy "Run: just bootstrap-esp"
check_tool espflash "Run: just bootstrap-esp"
check_tool python3 "Install Python 3 with your system package manager"
check_tool bazel "Install Bazelisk/Bazel so the repo's bazel command is available"

check_file_contains "$cargo_config" "[target.${EXPECTED_TARGET}]" ".cargo target ${EXPECTED_TARGET}" "Restore .cargo/config.toml"
check_file_contains "$cargo_config" 'linker = "ldproxy"' ".cargo linker uses ldproxy" "Restore .cargo/config.toml"
check_file_contains "$cargo_config" "MCU = \"${EXPECTED_MCU}\"" ".cargo MCU ${EXPECTED_MCU}" "Restore .cargo/config.toml"
check_file_contains "$cargo_config" "ESP_IDF_VERSION = \"${EXPECTED_IDF_VERSION}\"" ".cargo ESP_IDF_VERSION ${EXPECTED_IDF_VERSION}" "Restore .cargo/config.toml"
check_file_contains "$cargo_config" "ESP_IDF_TOOLS_INSTALL_DIR = \"${EXPECTED_IDF_DIR}\"" ".cargo ESP_IDF_TOOLS_INSTALL_DIR ${EXPECTED_IDF_DIR}" "Restore .cargo/config.toml"

check_file_contains "$firmware_manifest" "[package.metadata.esp-idf-sys]" "firmware esp-idf-sys metadata exists" "Restore firmware/bitaxe/Cargo.toml"
check_file_contains "$firmware_manifest" "esp_idf_version = \"${EXPECTED_IDF_VERSION}\"" "firmware pins ESP-IDF ${EXPECTED_IDF_VERSION}" "Restore firmware/bitaxe/Cargo.toml"
check_file_contains "$firmware_manifest" "esp_idf_tools_install_dir = \"${EXPECTED_IDF_DIR}\"" "firmware uses workspace ESP-IDF tools dir" "Restore firmware/bitaxe/Cargo.toml"

check_managed_embuild_tools
check_optional_idf_path

if [[ "$failures" -gt 0 ]]; then
	printf 'ESP-IDF dependency check failed: %s missing or misconfigured item(s), %s warning(s).\n' "$failures" "$warnings" >&2
	printf 'Run `just bootstrap-esp`, then source "%s" or open a new shell, and rerun `just doctor`.\n' "$esp_export" >&2
	exit 1
fi

printf 'ESP-IDF dependency check passed with %s warning(s).\n' "$warnings"
