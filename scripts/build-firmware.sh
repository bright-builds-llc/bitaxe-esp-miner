#!/usr/bin/env bash
set -euo pipefail

readonly TARGET="xtensa-esp32s3-espidf"
readonly MCU_NAME="esp32s3"
readonly ESP_IDF_VERSION_PIN="tag:v5.5.4"
readonly PACKAGE_NAME="bitaxe-firmware"

usage() {
  printf 'usage: %s <bazel-output-dir>\n' "$0" >&2
}

if [[ "$#" -ne 1 ]]; then
  usage
  exit 2
fi

readonly OUTPUT_DIR="$1"
if [[ -z "${HOME:-}" ]]; then
  HOME="$(cd ~ && pwd)"
  export HOME
fi

readonly ESP_EXPORT="${HOME}/export-esp.sh"

if [[ ! -f "$ESP_EXPORT" ]]; then
  printf 'error: missing ESP environment export at %s\n' "$ESP_EXPORT" >&2
  printf 'run `just doctor` to inspect ESP dependencies\n' >&2
  printf 'run `just bootstrap-esp` to install ESP Rust tooling, then source %s or open a new shell\n' "$ESP_EXPORT" >&2
  exit 1
fi

mkdir -p "$OUTPUT_DIR"

printf '[build-firmware] MCU=%s\n' "$MCU_NAME"
printf '[build-firmware] target=%s\n' "$TARGET"
printf '[build-firmware] esp_idf_version=%s\n' "$ESP_IDF_VERSION_PIN"
printf '[build-firmware] output_dir=%s\n' "$OUTPUT_DIR"

source "$ESP_EXPORT"

export ESP_IDF_SDKCONFIG="firmware/bitaxe/sdkconfig"
export ESP_IDF_SDKCONFIG_DEFAULTS="firmware/bitaxe/sdkconfig.defaults"
export ESP_IDF_SYS_ROOT_CRATE="$PACKAGE_NAME"
export ESP_IDF_TOOLS_INSTALL_DIR="workspace"
export ESP_IDF_VERSION="$ESP_IDF_VERSION_PIN"

if [[ -x "${HOME}/.cargo/bin/cargo" ]]; then
  PATH="${HOME}/.cargo/bin:${PATH}"
  export PATH
fi

if ! command -v cargo >/dev/null; then
  printf 'error: cargo not found after sourcing %s\n' "$ESP_EXPORT" >&2
  printf 'run `just doctor` to inspect ESP dependencies\n' >&2
  printf 'run `just bootstrap-esp` after installing Rust/Cargo with rustup\n' >&2
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
