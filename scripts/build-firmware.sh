#!/usr/bin/env bash
set -euo pipefail

readonly TARGET="xtensa-esp32s3-espidf"
readonly MCU_NAME="esp32s3"
readonly ESP_IDF_VERSION_PIN="tag:v5.5.4"
readonly PACKAGE_NAME="bitaxe-firmware"
readonly OUTPUT_SDKCONFIG_NAME="bitaxe-firmware.sdkconfig"
readonly OUTPUT_BOOTLOADER_NAME="bitaxe-firmware-bootloader.bin"
readonly OUTPUT_PARTITION_TABLE_NAME="bitaxe-firmware-partition-table.bin"
readonly OUTPUT_OTADATA_NAME="bitaxe-firmware-otadata-initial.bin"

usage() {
	printf 'usage: %s <bazel-output-dir> <build-provenance-stamp> <identity-sdkconfig-defaults>\n' "$0" >&2
}

if [[ "$#" -ne 3 ]]; then
	usage
	exit 2
fi

mkdir -p "$1"
OUTPUT_DIR="$(cd "$1" && pwd -P)"
readonly OUTPUT_DIR
BUILD_PROVENANCE_STAMP="$(cd "$(dirname "$2")" && pwd -P)/$(basename "$2")"
readonly BUILD_PROVENANCE_STAMP
IDENTITY_SDKCONFIG_DEFAULTS="$(cd "$(dirname "$3")" && pwd -P)/$(basename "$3")"
readonly IDENTITY_SDKCONFIG_DEFAULTS
if [[ -z "${HOME:-}" ]]; then
	HOME="$(cd ~ && pwd)"
	export HOME
fi

readonly ESP_EXPORT="${HOME}/export-esp.sh"

if [[ ! -f "$ESP_EXPORT" ]]; then
	printf 'error: missing ESP environment export at %s\n' "$ESP_EXPORT" >&2
	printf 'run just doctor to inspect ESP dependencies\n' >&2
	printf 'run just bootstrap-esp to install ESP Rust tooling, then source %s or open a new shell\n' "$ESP_EXPORT" >&2
	exit 1
fi

printf '[build-firmware] MCU=%s\n' "$MCU_NAME"
printf '[build-firmware] target=%s\n' "$TARGET"
printf '[build-firmware] esp_idf_version=%s\n' "$ESP_IDF_VERSION_PIN"
printf '[build-firmware] output_dir=%s\n' "$OUTPUT_DIR"

[[ -f "$BUILD_PROVENANCE_STAMP" ]] || {
	printf 'error: build provenance stamp not found: %s\n' "$BUILD_PROVENANCE_STAMP" >&2
	exit 1
}
[[ -f "$IDENTITY_SDKCONFIG_DEFAULTS" ]] || {
	printf 'error: identity sdkconfig defaults not found: %s\n' "$IDENTITY_SDKCONFIG_DEFAULTS" >&2
	exit 1
}

readonly OUTPUT_SDKCONFIG="${OUTPUT_DIR}/sdkconfig"
readonly OUTPUT_SDKCONFIG_DEFAULTS="${OUTPUT_DIR}/sdkconfig.defaults"
rm -f "$OUTPUT_SDKCONFIG" "$OUTPUT_SDKCONFIG_DEFAULTS"
cp firmware/bitaxe/sdkconfig.defaults "$OUTPUT_SDKCONFIG_DEFAULTS"
printf '\n' >>"$OUTPUT_SDKCONFIG_DEFAULTS"
while IFS= read -r line || [[ -n "$line" ]]; do
	printf '%s\n' "$line" >>"$OUTPUT_SDKCONFIG_DEFAULTS"
done <"$IDENTITY_SDKCONFIG_DEFAULTS"
export BITAXE_BUILD_PROVENANCE_STAMP="$BUILD_PROVENANCE_STAMP"

# shellcheck source=/dev/null
source "$ESP_EXPORT"

export ESP_IDF_SDKCONFIG="$OUTPUT_SDKCONFIG"
export ESP_IDF_SDKCONFIG_DEFAULTS="$OUTPUT_SDKCONFIG_DEFAULTS"
export ESP_IDF_SYS_ROOT_CRATE="$PACKAGE_NAME"
export ESP_IDF_TOOLS_INSTALL_DIR="workspace"
export ESP_IDF_VERSION="$ESP_IDF_VERSION_PIN"

if [[ -x "${HOME}/.cargo/bin/cargo" ]]; then
	PATH="${HOME}/.cargo/bin:${PATH}"
	export PATH
fi

if ! command -v cargo >/dev/null; then
	printf 'error: cargo not found after sourcing %s\n' "$ESP_EXPORT" >&2
	printf 'run just doctor to inspect ESP dependencies\n' >&2
	printf 'run just bootstrap-esp after installing Rust/Cargo with rustup\n' >&2
	exit 1
fi

cargo_cmd=(
	cargo
	build
	-p "$PACKAGE_NAME"
	--release
	--target "$TARGET"
)

printf '[build-firmware] cargo_command='
printf '%q ' "${cargo_cmd[@]}"
printf '\n'

"${cargo_cmd[@]}"

readonly SOURCE_ELF="target/${TARGET}/release/${PACKAGE_NAME}"
readonly OUTPUT_ELF="${OUTPUT_DIR}/${PACKAGE_NAME}.elf"

if [[ ! -f "$SOURCE_ELF" ]]; then
	printf 'error: expected firmware ELF was not produced: %s\n' "$SOURCE_ELF" >&2
	exit 1
fi

cp "$SOURCE_ELF" "$OUTPUT_ELF"
printf '[build-firmware] copied_elf=%s\n' "$OUTPUT_ELF"

expected_build_label=""
expected_build_label_count=0
while IFS='=' read -r key value; do
	if [[ "$key" == "build_label" ]]; then
		expected_build_label="$value"
		expected_build_label_count=$((expected_build_label_count + 1))
	fi
done <"$BUILD_PROVENANCE_STAMP"
if [[ "$expected_build_label_count" -ne 1 || -z "$expected_build_label" ]]; then
	printf 'error: build provenance stamp must contain exactly one non-empty build_label field\n' >&2
	exit 1
fi

shopt -s nullglob
generated_idf_candidates=(target/${TARGET}/release/build/esp-idf-sys-*/out)
shopt -u nullglob
generated_idf_matches=()
for candidate in "${generated_idf_candidates[@]}"; do
	if [[ ! -f "${candidate}/sdkconfig" ]]; then
		continue
	fi
	if ! grep -Fqx "CONFIG_APP_PROJECT_VER=\"${expected_build_label}\"" "${candidate}/sdkconfig"; then
		continue
	fi
	if ! grep -Fqx 'CONFIG_APP_RETRIEVE_LEN_ELF_SHA=64' "${candidate}/sdkconfig"; then
		continue
	fi
	if [[ ! -f "${candidate}/build/bootloader/bootloader.bin" || ! -f "${candidate}/build/partition_table/partition-table.bin" || ! -f "${candidate}/build/ota_data_initial.bin" ]]; then
		continue
	fi
	generated_idf_matches+=("$candidate")
done
if [[ "${#generated_idf_matches[@]}" -ne 1 ]]; then
	printf 'error: expected exactly one generated ESP-IDF build for label %s, found %s\n' "$expected_build_label" "${#generated_idf_matches[@]}" >&2
	exit 1
fi

readonly GENERATED_IDF_BUILD_DIR="${generated_idf_matches[0]}"
cp "${GENERATED_IDF_BUILD_DIR}/sdkconfig" "${OUTPUT_DIR}/${OUTPUT_SDKCONFIG_NAME}"
cp "${GENERATED_IDF_BUILD_DIR}/build/bootloader/bootloader.bin" "${OUTPUT_DIR}/${OUTPUT_BOOTLOADER_NAME}"
cp "${GENERATED_IDF_BUILD_DIR}/build/partition_table/partition-table.bin" "${OUTPUT_DIR}/${OUTPUT_PARTITION_TABLE_NAME}"
cp "${GENERATED_IDF_BUILD_DIR}/build/ota_data_initial.bin" "${OUTPUT_DIR}/${OUTPUT_OTADATA_NAME}"
printf '[build-firmware] copied_idf_outputs=%s,%s,%s,%s\n' \
	"${OUTPUT_SDKCONFIG_NAME}" \
	"${OUTPUT_BOOTLOADER_NAME}" \
	"${OUTPUT_PARTITION_TABLE_NAME}" \
	"${OUTPUT_OTADATA_NAME}"
