#!/usr/bin/env bash
set -euo pipefail

readonly DEFAULT_REFERENCE_GUARD="scripts/verify-reference-clean.sh"
readonly PACKAGE_ELF_NAME="bitaxe-gamma601.elf"
readonly FACTORY_IMAGE_NAME="bitaxe-gamma601-factory.bin"
readonly MANIFEST_NAME="bitaxe-gamma601-package.json"

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
factory_image="${out_dir}/${FACTORY_IMAGE_NAME}"

cp "$firmware_elf" "$package_elf"

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
	601
	--firmware-elf
	"$package_elf"
	--default-flash-image
	"$package_elf"
	--out-dir
	"$out_dir"
	--manifest
	"$manifest"
	--factory-image
	"$factory_image"
)

printf '[package-firmware] cargo_command='
printf '%q ' "${cargo_cmd[@]}"
printf '\n'

"${cargo_cmd[@]}"
printf '[package-firmware] manifest=%s\n' "$manifest"
