#!/usr/bin/env bash
set -euo pipefail

readonly FIRMWARE_TARGET="//firmware/bitaxe:firmware_image"
readonly ENABLEMENT_MODE="phase27-live-hardware-asic-stratum-bridge"
readonly HARDWARE_EVIDENCE_ACK="ultra205-phase27-live-hardware-bridge-safe-stop"

usage() {
	printf 'usage: %s [--out-dir PATH]\n' "$(basename "$0")" >&2
}

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
out_dir="${repo_root}/bazel-bin/firmware/bitaxe"

while [[ "$#" -gt 0 ]]; do
	case "$1" in
	--out-dir)
		if [[ "$#" -lt 2 ]]; then
			usage
			exit 2
		fi
		out_dir="${2%/}"
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

package_manifest="${out_dir}/bitaxe-ultra205-package.json"
enablement_ledger="${out_dir}/phase27-live-hardware-bridge-enablement.md"

build_cmd=(
	bazel
	build
	"--action_env=BITAXE_MINING_EVIDENCE_MODE=${ENABLEMENT_MODE}"
	"--action_env=BITAXE_HARDWARE_EVIDENCE_ACK=${HARDWARE_EVIDENCE_ACK}"
	"$FIRMWARE_TARGET"
)

printf '[phase27-live-hardware-bridge-package] build_command='
printf '%q ' "${build_cmd[@]}"
printf '\n'

(
	cd "$repo_root"
	"${build_cmd[@]}"
)

if [[ ! -f "$package_manifest" ]]; then
	bazel_bin="$(cd "$repo_root" && bazel info bazel-bin)"
	package_manifest="${bazel_bin}/firmware/bitaxe/bitaxe-ultra205-package.json"
fi

if [[ ! -f "$package_manifest" ]]; then
	printf 'error: package manifest missing after build: %s\n' "$package_manifest" >&2
	exit 1
fi

source_commit="$(git -C "$repo_root" rev-parse HEAD 2>/dev/null || printf 'unknown-source')"
reference_commit="$(git -C "$repo_root/reference/esp-miner" rev-parse HEAD 2>/dev/null || printf 'unknown-reference')"

cat >"$enablement_ledger" <<LEDGER
# Phase 27 Live Hardware Bridge Enablement

phase27_live_hardware_bridge_package_status: ready
enablement_mode: ${ENABLEMENT_MODE}
hardware_evidence_ack: ${HARDWARE_EVIDENCE_ACK}
expected_boot_markers: phase27_safety_bring_up=complete,safety_power_status=observed,safety_thermal_status=observed,safety_fan_status=startup_duty,asic_enable_status=active,asic_production_status=initialized
package_manifest: ${package_manifest}
source_commit: ${source_commit}
reference_commit: ${reference_commit}
LEDGER

printf '[phase27-live-hardware-bridge-package] manifest=%s\n' "$package_manifest"
printf '[phase27-live-hardware-bridge-package] ledger=%s\n' "$enablement_ledger"
