#!/usr/bin/env bash
set -euo pipefail

readonly DEFAULT_REFERENCE_GUARD="scripts/verify-reference-clean.sh"
readonly PACKAGE_ELF_NAME="bitaxe-ultra205.elf"
readonly FIRMWARE_OTA_IMAGE_NAME="esp-miner.bin"
readonly WWW_IMAGE_NAME="www.bin"
readonly OTADATA_INITIAL_NAME="otadata-initial.bin"
readonly FACTORY_IMAGE_NAME="bitaxe-ultra205-factory.bin"
readonly MANIFEST_NAME="bitaxe-ultra205-package.json"
readonly ULTRA205_PARTITION_TABLE="firmware/bitaxe/partitions-ultra205.csv"
readonly WWW_SOURCE_DIR="firmware/bitaxe/static/www"
readonly WWW_SMOKE_ASSET="firmware/bitaxe/static/www/assets/app.css.gz"
readonly WWW_SPIFFS_SIZE="0x300000"

detect_workspace_dir() {
	local maybe_git_dir
	maybe_git_dir="$(git rev-parse --git-dir)"

	local maybe_git_target
	if maybe_git_target="$(readlink "$maybe_git_dir")"; then
		case "$maybe_git_target" in
		*/.git)
			dirname "$maybe_git_target"
			return 0
			;;
		esac
	fi

	git rev-parse --show-toplevel
}

usage() {
	printf 'usage: %s --firmware-elf <path> --out-dir <path> [--manifest <path>] [--reference-guard <path>]\n' "$0" >&2
}

find_spiffsgen() {
	local idf_spiffsgen="${IDF_PATH:-}/components/spiffs/spiffsgen.py"
	if [[ -n "${IDF_PATH:-}" && -f "$idf_spiffsgen" ]]; then
		printf '%s\n' "$idf_spiffsgen"
		return 0
	fi

	local candidate
	for candidate in \
		"${workspace_dir}/.embuild/espressif/esp-idf/v5.5.4/components/spiffs/spiffsgen.py" \
		"${workspace_dir}/.embuild/espressif/esp-idf/v5.2.3/components/spiffs/spiffsgen.py"; do
		if [[ -f "$candidate" ]]; then
			printf '%s\n' "$candidate"
			return 0
		fi
	done

	printf 'error: spiffsgen.py not found; set IDF_PATH so IDF_PATH/components/spiffs/spiffsgen.py exists\n' >&2
	return 1
}

find_generated_otadata() {
	local candidate
	for candidate in \
		target/xtensa-esp32s3-espidf/release/build/esp-idf-sys-*/out/build/ota_data_initial.bin \
		target/xtensa-esp32s3-espidf/debug/build/esp-idf-sys-*/out/build/ota_data_initial.bin; do
		if [[ -f "$candidate" ]]; then
			printf '%s\n' "$candidate"
			return 0
		fi
	done

	return 1
}

write_erased_otadata() {
	local output="$1"
	python3 -c 'from pathlib import Path; import sys; Path(sys.argv[1]).write_bytes(b"\xff" * 8192)' "$output"
}

reference_guard="$DEFAULT_REFERENCE_GUARD"
firmware_elf=""
out_dir=""
manifest=""

if [[ -z "${HOME:-}" ]]; then
	HOME="$(cd ~ && pwd)"
	export HOME
fi

if [[ -d "${HOME}/.cargo/bin" ]]; then
	PATH="${HOME}/.cargo/bin:${PATH}"
	export PATH
fi

while [[ "$#" -gt 0 ]]; do
	case "$1" in
	--reference-guard)
		if [[ "$#" -lt 2 ]]; then
			usage
			exit 2
		fi
		reference_guard="$2"
		shift 2
		;;
	--firmware-elf)
		if [[ "$#" -lt 2 ]]; then
			usage
			exit 2
		fi
		firmware_elf="$2"
		shift 2
		;;
	--out-dir)
		if [[ "$#" -lt 2 ]]; then
			usage
			exit 2
		fi
		out_dir="$2"
		shift 2
		;;
	--manifest)
		if [[ "$#" -lt 2 ]]; then
			usage
			exit 2
		fi
		manifest="$2"
		shift 2
		;;
	-h | --help)
		usage
		exit 0
		;;
	*)
		if [[ -z "$firmware_elf" ]]; then
			firmware_elf="$1"
			shift
			continue
		fi
		printf 'error: unexpected argument: %s\n' "$1" >&2
		usage
		exit 2
		;;
	esac
done

if [[ -z "$firmware_elf" ]]; then
	printf 'error: missing --firmware-elf <path>\n' >&2
	usage
	exit 2
fi

if [[ -z "$out_dir" ]]; then
	printf 'error: missing --out-dir <path>\n' >&2
	usage
	exit 2
fi

if [[ ! -f "$firmware_elf" ]]; then
	printf 'error: firmware ELF path does not exist: %s\n' "$firmware_elf" >&2
	exit 1
fi

if [[ ! -x "$reference_guard" && ! -f "$reference_guard" ]]; then
	printf 'error: reference guard not found: %s\n' "$reference_guard" >&2
	exit 1
fi

if [[ -z "$manifest" ]]; then
	manifest="${out_dir}/${MANIFEST_NAME}"
fi

workspace_dir="$(detect_workspace_dir)"
export BUILD_WORKSPACE_DIRECTORY="$workspace_dir"
cd "$workspace_dir"

"$reference_guard"

if ! command -v espflash >/dev/null; then
	printf 'error: espflash not found; install or upgrade it with: cargo install espflash --locked\n' >&2
	exit 1
fi

mkdir -p "$out_dir"

package_elf="${out_dir}/${PACKAGE_ELF_NAME}"
firmware_ota_image="${out_dir}/${FIRMWARE_OTA_IMAGE_NAME}"
www_image="${out_dir}/${WWW_IMAGE_NAME}"
otadata_initial="${out_dir}/${OTADATA_INITIAL_NAME}"
factory_image="${out_dir}/${FACTORY_IMAGE_NAME}"

cp "$firmware_elf" "$package_elf"

if [[ ! -f "$ULTRA205_PARTITION_TABLE" ]]; then
	printf 'error: Ultra 205 partition table not found: %s\n' "$ULTRA205_PARTITION_TABLE" >&2
	exit 1
fi

if [[ ! -d "$WWW_SOURCE_DIR" ]]; then
	printf 'error: static www source directory not found: %s\n' "$WWW_SOURCE_DIR" >&2
	exit 1
fi

if [[ ! -f "$WWW_SMOKE_ASSET" ]]; then
	printf 'error: representative gzip static smoke asset not found: %s\n' "$WWW_SMOKE_ASSET" >&2
	exit 1
fi

spiffsgen="$(find_spiffsgen)"
spiffs_cmd=(
	python3
	"$spiffsgen"
	"$WWW_SPIFFS_SIZE"
	"$WWW_SOURCE_DIR"
	"$www_image"
)

printf '[package-firmware] spiffs_command='
printf '%q ' "${spiffs_cmd[@]}"
printf '\n'

"${spiffs_cmd[@]}"

if generated_otadata="$(find_generated_otadata)"; then
	cp "$generated_otadata" "$otadata_initial"
	otadata_source="$generated_otadata"
else
	write_erased_otadata "$otadata_initial"
	otadata_source="generated-erased-flash"
fi
printf '[package-firmware] otadata_source=%s\n' "$otadata_source"

firmware_ota_cmd=(
	espflash
	save-image
	--chip
	esp32s3
	--flash-size
	16mb
	--flash-mode
	dio
	--flash-freq
	80mhz
	--partition-table
	"$ULTRA205_PARTITION_TABLE"
	--target-app-partition
	ota_0
	"$package_elf"
	"$firmware_ota_image"
)

printf '[package-firmware] firmware_ota_command='
printf '%q ' "${firmware_ota_cmd[@]}"
printf '\n'

if ! "${firmware_ota_cmd[@]}"; then
	printf 'error: espflash save-image failed for %s; install or upgrade espflash with: cargo install espflash --locked\n' "$FIRMWARE_OTA_IMAGE_NAME" >&2
	exit 1
fi

espflash_cmd=(
	espflash
	save-image
	--chip
	esp32s3
	--merge
	"$package_elf"
	"$factory_image"
)

printf '[package-firmware] espflash_command='
printf '%q ' "${espflash_cmd[@]}"
printf '\n'

if ! "${espflash_cmd[@]}"; then
	printf 'error: espflash save-image failed; install or upgrade espflash with: cargo install espflash --locked\n' >&2
	exit 1
fi

cargo_cmd=(
	cargo
	run
	-p
	xtask
	--
	package-firmware
	--board
	205
	--firmware-elf
	"$package_elf"
	--firmware-ota-image
	"$firmware_ota_image"
	--www-bin
	"$www_image"
	--partition-table
	"$ULTRA205_PARTITION_TABLE"
	--otadata-initial
	"$otadata_initial"
	--default-flash-image
	"$package_elf"
	--out-dir
	"$out_dir"
	--manifest
	"$manifest"
	--factory-image
	"$factory_image"
	--release-name
	"bitaxe-ultra205"
	--install-notes
	"docs/release/ultra-205.md"
	--license-inventory
	"docs/release/license-inventory.md"
	--provenance-manifest
	"docs/release/provenance-manifest.md"
	--otadata-source
	"$otadata_source"
)

printf '[package-firmware] cargo_command='
printf '%q ' "${cargo_cmd[@]}"
printf '\n'

"${cargo_cmd[@]}"
printf '[package-firmware] manifest=%s\n' "$manifest"
