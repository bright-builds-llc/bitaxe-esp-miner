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
readonly WWW_SPIFFS_OBJ_NAME_LEN="64"
readonly OTA_PARTITION_SIZE_BYTES="$((0x400000))"

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
	printf 'usage: %s --firmware-elf <path> --build-provenance-stamp <path> --out-dir <path> [--manifest <path>] [--reference-guard <path>]\n' "$0" >&2
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

	printf 'error: spiffsgen.py not found; run just doctor to inspect ESP dependencies\n' >&2
	printf 'advanced override: set IDF_PATH so IDF_PATH/components/spiffs/spiffsgen.py exists\n' >&2
	return 1
}

find_esptool() {
	if command -v esptool.py >/dev/null 2>&1; then
		command -v esptool.py
		return 0
	fi

	local candidate
	for candidate in \
		"${workspace_dir}"/.embuild/espressif/python_env/idf5.5*_env/bin/esptool.py \
		"${workspace_dir}/.embuild/espressif/esp-idf/v5.5.4/components/esptool_py/esptool/esptool.py"; do
		if [[ -f "$candidate" ]]; then
			printf '%s\n' "$candidate"
			return 0
		fi
	done

	printf 'error: esptool.py not found; run just doctor to inspect ESP dependencies\n' >&2
	printf 'expected managed Esptool under .embuild/espressif after the first ESP-IDF firmware build\n' >&2
	return 1
}

read_stamp_field() {
	local stamp="$1"
	local expected_key="$2"
	local value=""
	local count=0
	local key
	local candidate_value
	while IFS='=' read -r key candidate_value; do
		if [[ "$key" == "$expected_key" ]]; then
			value="$candidate_value"
			count=$((count + 1))
		fi
	done <"$stamp"
	if [[ "$count" -ne 1 || -z "$value" ]]; then
		printf 'error: provenance stamp must contain exactly one non-empty %s field\n' "$expected_key" >&2
		return 1
	fi
	printf '%s\n' "$value"
}

find_generated_idf_build_dir() {
	local expected_label="$1"
	local candidates=()
	if [[ -n "${ESP_IDF_BUILD_DIR:-}" ]]; then
		candidates+=("$ESP_IDF_BUILD_DIR")
	else
		candidates+=(target/xtensa-esp32s3-espidf/release/build/esp-idf-sys-*/out)
	fi

	local matches=()
	local candidate
	for candidate in "${candidates[@]}"; do
		if [[ ! -f "${candidate}/sdkconfig" ]]; then
			continue
		fi
		if ! grep -Fqx "CONFIG_APP_PROJECT_VER=\"${expected_label}\"" "${candidate}/sdkconfig"; then
			continue
		fi
		if ! grep -Fqx 'CONFIG_APP_RETRIEVE_LEN_ELF_SHA=64' "${candidate}/sdkconfig"; then
			continue
		fi
		if [[ ! -f "${candidate}/build/bootloader/bootloader.bin" || ! -f "${candidate}/build/partition_table/partition-table.bin" || ! -f "${candidate}/build/ota_data_initial.bin" ]]; then
			continue
		fi
		matches+=("$candidate")
	done

	if [[ "${#matches[@]}" -ne 1 ]]; then
		printf 'error: expected exactly one generated ESP-IDF build for label %s, found %s\n' "$expected_label" "${#matches[@]}" >&2
		return 1
	fi
	printf '%s\n' "${matches[0]}"
}

absolute_existing_path() {
	local path="$1"
	absolute_path "$path"
}

absolute_path() {
	local path="$1"
	if [[ "$path" == /* ]]; then
		printf '%s\n' "$path"
		return 0
	fi

	local dir
	dir="$(dirname "$path")"
	local base
	base="$(basename "$path")"
	printf '%s/%s\n' "$(cd "$dir" && pwd)" "$base"
}

reference_guard="$DEFAULT_REFERENCE_GUARD"
firmware_elf=""
build_provenance_stamp=""
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
	--build-provenance-stamp)
		if [[ "$#" -lt 2 ]]; then
			usage
			exit 2
		fi
		build_provenance_stamp="$2"
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

if [[ -z "$build_provenance_stamp" ]]; then
	printf 'error: missing --build-provenance-stamp <path>\n' >&2
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
reference_guard="$(absolute_existing_path "$reference_guard")"

if [[ -z "$manifest" ]]; then
	manifest="${out_dir}/${MANIFEST_NAME}"
fi

firmware_elf="$(absolute_existing_path "$firmware_elf")"
build_provenance_stamp="$(absolute_existing_path "$build_provenance_stamp")"
mkdir -p "$out_dir"
out_dir="$(absolute_existing_path "$out_dir")"
manifest="$(absolute_path "$manifest")"

workspace_dir="$(detect_workspace_dir)"
export BUILD_WORKSPACE_DIRECTORY="$workspace_dir"
cd "$workspace_dir"

"$reference_guard"

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
esptool="$(find_esptool)"
expected_build_label="$(read_stamp_field "$build_provenance_stamp" build_label)"
generated_idf_build_dir="$(find_generated_idf_build_dir "$expected_build_label")"
generated_bootloader="${generated_idf_build_dir}/build/bootloader/bootloader.bin"
generated_partition_table="${generated_idf_build_dir}/build/partition_table/partition-table.bin"
generated_otadata="${generated_idf_build_dir}/build/ota_data_initial.bin"
spiffs_cmd=(
	python3
	"$spiffsgen"
	--obj-name-len
	"$WWW_SPIFFS_OBJ_NAME_LEN"
	"$WWW_SPIFFS_SIZE"
	"$WWW_SOURCE_DIR"
	"$www_image"
)

printf '[package-firmware] spiffs_command='
printf '%q ' "${spiffs_cmd[@]}"
printf '\n'

"${spiffs_cmd[@]}"

cp "$generated_otadata" "$otadata_initial"
otadata_source="$generated_otadata"
printf '[package-firmware] otadata_source=%s\n' "$otadata_source"

firmware_ota_cmd=(
	"$esptool"
	--chip
	esp32s3
	elf2image
	--version
	2
	--flash_size
	16MB
	--flash_mode
	dio
	--flash_freq
	80m
	--elf-sha256-offset
	0xb0
	--min-rev-full
	0
	--max-rev-full
	99
	-o
	"$firmware_ota_image"
	"$package_elf"
)

printf '[package-firmware] firmware_ota_command='
printf '%q ' "${firmware_ota_cmd[@]}"
printf '\n'

if ! "${firmware_ota_cmd[@]}"; then
	printf 'error: esptool.py elf2image failed for %s; run just doctor to inspect managed .embuild ESP tools\n' "$FIRMWARE_OTA_IMAGE_NAME" >&2
	exit 1
fi

firmware_ota_size="$(wc -c <"$firmware_ota_image")"
if [[ "$firmware_ota_size" -gt "$OTA_PARTITION_SIZE_BYTES" ]]; then
	printf 'error: firmware OTA image is %s bytes, exceeding ota_0 size %s\n' "$firmware_ota_size" "$OTA_PARTITION_SIZE_BYTES" >&2
	exit 1
fi
app_descriptor_info="$("$esptool" image_info --version 2 "$firmware_ota_image")"
app_descriptor_version=""
app_elf_sha256=""
while IFS= read -r line; do
	case "$line" in
	"App version: "*) app_descriptor_version="${line#App version: }" ;;
	"ELF file SHA256: "*) app_elf_sha256="${line#ELF file SHA256: }" ;;
	esac
done <<<"$app_descriptor_info"

if [[ -z "$app_descriptor_version" ]]; then
	printf 'error: ESP application descriptor did not contain App version\n' >&2
	exit 1
fi
if [[ ! "$app_elf_sha256" =~ ^[0-9a-f]{64}$ || "$app_elf_sha256" =~ ^0+$ ]]; then
	printf 'error: ESP application descriptor did not contain a nonzero lowercase ELF SHA-256\n' >&2
	exit 1
fi

factory_merge_cmd=(
	"$esptool"
	--chip
	esp32s3
	merge_bin
	--flash_mode
	dio
	--flash_size
	16MB
	--flash_freq
	80m
	0x0
	"$generated_bootloader"
	0x8000
	"$generated_partition_table"
	0x10000
	"$firmware_ota_image"
	0x410000
	"$www_image"
	0xf10000
	"$otadata_initial"
	-o
	"$factory_image"
)

printf '[package-firmware] factory_merge_command='
printf '%q ' "${factory_merge_cmd[@]}"
printf '\n'

if ! "${factory_merge_cmd[@]}"; then
	printf 'error: esptool.py merge_bin failed for %s; run just doctor to inspect managed .embuild ESP tools\n' "$FACTORY_IMAGE_NAME" >&2
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
	--build-provenance-stamp
	"$build_provenance_stamp"
	--app-descriptor-version
	"$app_descriptor_version"
	--app-elf-sha256
	"$app_elf_sha256"
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
