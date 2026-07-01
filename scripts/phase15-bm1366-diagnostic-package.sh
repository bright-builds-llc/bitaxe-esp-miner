#!/usr/bin/env bash
set -euo pipefail

readonly FIRMWARE_TARGET="//firmware/bitaxe:firmware"
readonly DEFAULT_PACKAGE_FIRMWARE_SCRIPT="scripts/package-firmware.sh"
readonly REFERENCE_GUARD="scripts/verify-reference-clean.sh"
readonly FLASH_MONITOR_COMMAND_SHAPE="bazel run //tools/flash:flash -- flash-monitor --board 205 --port <port> --manifest <out-dir>/package/bitaxe-ultra205-package.json --evidence-dir <evidence-dir> --capture-timeout-seconds <seconds>"

usage() {
	printf 'usage: %s --mode chip-detect|work-result --out-dir PATH\n' "$0" >&2
}

mode=""
out_dir=""

while [[ "$#" -gt 0 ]]; do
	case "$1" in
	--mode)
		if [[ "$#" -lt 2 ]]; then
			usage
			exit 2
		fi
		mode="$2"
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
	-h | --help)
		usage
		exit 0
		;;
	*)
		printf 'error: unexpected argument: %s\n' "$1" >&2
		usage
		exit 2
		;;
	esac
done

if [[ -z "$mode" || -z "$out_dir" ]]; then
	usage
	exit 2
fi

diagnostic_env=""
hardware_evidence_ack=""

case "$mode" in
chip-detect)
	diagnostic_env="BITAXE_ASIC_DIAGNOSTIC=chip-detect"
	hardware_evidence_ack="BITAXE_HARDWARE_EVIDENCE_ACK=ultra205-chip-detect-safe-bench"
	;;
work-result)
	diagnostic_env="BITAXE_ASIC_DIAGNOSTIC=work-result"
	hardware_evidence_ack="BITAXE_HARDWARE_EVIDENCE_ACK=ultra205-work-result-safe-bench"
	;;
*)
	printf 'error: unsupported diagnostic mode: %s\n' "$mode" >&2
	usage
	exit 2
	;;
esac

json_escape() {
	local value="$1"
	value="${value//\\/\\\\}"
	value="${value//\"/\\\"}"
	value="${value//$'\n'/\\n}"
	value="${value//$'\r'/\\r}"
	value="${value//$'\t'/\\t}"
	printf '%s' "$value"
}

write_summary() {
	local summary_path="$1"
	local package_manifest="$2"

	cat >"$summary_path" <<JSON
{
  "mode": "$(json_escape "$mode")",
  "diagnostic_env": "$(json_escape "$diagnostic_env")",
  "hardware_evidence_ack": "$(json_escape "$hardware_evidence_ack")",
  "package_manifest": "$(json_escape "$package_manifest")",
  "flash_monitor_command_shape": "$(json_escape "$FLASH_MONITOR_COMMAND_SHAPE")"
}
JSON
}

mkdir -p "$out_dir"

build_cmd=(
	bazel
	build
	"--action_env=${diagnostic_env}"
	"--action_env=${hardware_evidence_ack}"
	"$FIRMWARE_TARGET"
)

printf '[phase15-diagnostic-package] build_command='
printf '%q ' "${build_cmd[@]}"
printf '\n'
"${build_cmd[@]}"

bazel_bin="$(bazel info bazel-bin)"
if [[ -z "$bazel_bin" ]]; then
	printf 'error: bazel info bazel-bin returned no path\n' >&2
	exit 1
fi

package_dir="${out_dir}/package"
package_manifest="${package_dir}/bitaxe-ultra205-package.json"
firmware_elf="${bazel_bin}/firmware/bitaxe/bitaxe-firmware.elf"
package_firmware_script="${PHASE15_PACKAGE_FIRMWARE_SCRIPT:-$DEFAULT_PACKAGE_FIRMWARE_SCRIPT}"

package_cmd=(
	"$package_firmware_script"
	--reference-guard
	"$REFERENCE_GUARD"
	--firmware-elf
	"$firmware_elf"
	--out-dir
	"$package_dir"
	--manifest
	"$package_manifest"
)

printf '[phase15-diagnostic-package] package_command='
printf '%q ' "${package_cmd[@]}"
printf '\n'
"${package_cmd[@]}"

write_summary "${out_dir}/diagnostic-package-summary.json" "$package_manifest"
printf '[phase15-diagnostic-package] summary=%s\n' "${out_dir}/diagnostic-package-summary.json"
