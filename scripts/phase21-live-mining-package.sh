#!/usr/bin/env bash
set -euo pipefail

readonly FIRMWARE_TARGET="//firmware/bitaxe:firmware"
readonly DEFAULT_PACKAGE_FIRMWARE_SCRIPT="scripts/package-firmware.sh"
readonly REFERENCE_GUARD="scripts/verify-reference-clean.sh"
readonly ENABLEMENT_MODE="live-mining-runtime"
readonly HARDWARE_EVIDENCE_ACK="ultra205-live-mining-runtime-safe-bench"
readonly RUNTIME_REQUIRED_LOG_MARKERS="phase21_controlled_runtime_status, stratum_subscribe_status, stratum_authorize_status, stratum_notify_status, bm1366_work_dispatch_status, result_receive_status, share_submission_status, runtime_snapshot_status, api_websocket_telemetry_update_status, safe_stop_status"
readonly RECOVERY_STEPS="stop wrapper, power-cycle board if serial stalls, reflash default safe package if live mode does not safe-stop"

usage() {
	printf 'usage: %s --out-dir PATH --readiness-audit PATH\n' "$0" >&2
}

out_dir=""
readiness_audit=""

while [[ "$#" -gt 0 ]]; do
	case "$1" in
	--out-dir)
		if [[ "$#" -lt 2 ]]; then
			usage
			exit 2
		fi
		out_dir="$2"
		shift 2
		;;
	--readiness-audit)
		if [[ "$#" -lt 2 ]]; then
			usage
			exit 2
		fi
		readiness_audit="$2"
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

if [[ -z "$out_dir" || -z "$readiness_audit" ]]; then
	usage
	exit 2
fi

out_dir="${out_dir%/}"
package_dir="${out_dir}/package"
package_manifest="${package_dir}/bitaxe-ultra205-package.json"
enablement_ledger="${out_dir}.md"

write_ledger() {
	local package_status="$1"
	local harness_status="$2"
	local contract_tests="$3"
	local readiness_status="$4"
	local blocker="$5"
	local source_commit="$6"
	local reference_commit="$7"

	mkdir -p "$out_dir" "$(dirname "$enablement_ledger")"
	cat >"$enablement_ledger" <<LEDGER
# Phase 21 Live Mining Enablement

controlled_live_mining_package_status: ${package_status}
controlled_runtime_harness_status: ${harness_status}
controlled_runtime_contract_tests: ${contract_tests}
runtime_required_log_markers: ${RUNTIME_REQUIRED_LOG_MARKERS}
readiness_status: ${readiness_status}
enablement_mode: ${ENABLEMENT_MODE}
hardware_evidence_ack: ${HARDWARE_EVIDENCE_ACK}
package_manifest: ${package_manifest}
source_commit: ${source_commit}
reference_commit: ${reference_commit}
recovery_steps: ${RECOVERY_STEPS}
prerequisite_artifacts: readiness-audit.md
evidence_dir: ${out_dir}
redaction_reviewer: required-before-citation
post_action_safe_state_marker: safe_state: mining=disabled
hardware_control=disabled
work_submission=disabled
LEDGER

	if [[ -n "$blocker" ]]; then
		printf 'blocker: %s\n' "$blocker" >>"$enablement_ledger"
	fi
}

block_enablement() {
	local blocker="$1"
	local contract_tests="${2:-not_run}"
	local readiness_status="${3:-blocked}"

	write_ledger "blocked" "blocked" "$contract_tests" "$readiness_status" "$blocker" "" ""
	printf '[phase21-live-mining-package] blocked=%s ledger=%s\n' "$blocker" "$enablement_ledger" >&2
	exit 1
}

read_manifest_field() {
	local manifest="$1"
	local field="$2"

	python3 - "$manifest" "$field" <<'PY'
import json
import sys
from pathlib import Path

manifest = Path(sys.argv[1])
field = sys.argv[2]
try:
    value = json.loads(manifest.read_text(encoding="utf-8")).get(field, "")
except Exception:
    sys.exit(1)
if not isinstance(value, str) or not value.strip():
    sys.exit(1)
print(value)
PY
}

clean_generated_package_outputs() {
	rm -f \
		"${package_dir}/bitaxe-ultra205.elf" \
		"${package_dir}/esp-miner.bin" \
		"${package_dir}/www.bin" \
		"${package_dir}/otadata-initial.bin" \
		"${package_dir}/bitaxe-ultra205-factory.bin" \
		"${package_dir}/bitaxe-ultra205-package.json"
}

validate_readiness_audit() {
	if [[ ! -f "$readiness_audit" ]]; then
		block_enablement "missing readiness audit" "not_run" "blocked"
	fi

	if ! grep -Fq "firmware_live_mining_status: blocked_by_default" "$readiness_audit" ||
		! grep -Fq "controlled_enablement_required: true" "$readiness_audit"; then
		block_enablement "readiness audit missing controlled enablement markers" "not_run" "blocked"
	fi
}

validate_readiness_audit

if ! cargo test -p bitaxe-stratum --all-features controlled_runtime; then
	block_enablement "controlled runtime contract tests failed" "failed" "blocked_by_default"
fi

build_cmd=(
	bazel
	build
	"--action_env=BITAXE_MINING_EVIDENCE_MODE=${ENABLEMENT_MODE}"
	"--action_env=BITAXE_HARDWARE_EVIDENCE_ACK=${HARDWARE_EVIDENCE_ACK}"
	"$FIRMWARE_TARGET"
)

printf '[phase21-live-mining-package] build_command='
printf '%q ' "${build_cmd[@]}"
printf '\n'
if ! "${build_cmd[@]}"; then
	block_enablement "controlled firmware build failed" "passed" "blocked_by_default"
fi

bazel_bin="$(bazel info bazel-bin)"
if [[ -z "$bazel_bin" ]]; then
	block_enablement "bazel info bazel-bin returned empty path" "passed" "blocked_by_default"
fi

mkdir -p "$package_dir"
clean_generated_package_outputs
firmware_elf="${bazel_bin}/firmware/bitaxe/bitaxe-firmware.elf"
package_firmware_script="${PHASE21_PACKAGE_FIRMWARE_SCRIPT:-$DEFAULT_PACKAGE_FIRMWARE_SCRIPT}"
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

printf '[phase21-live-mining-package] package_command='
printf '%q ' "${package_cmd[@]}"
printf '\n'
if ! "${package_cmd[@]}"; then
	block_enablement "controlled firmware package failed" "passed" "blocked_by_default"
fi

if [[ ! -f "$package_manifest" ]]; then
	block_enablement "package manifest missing after packaging" "passed" "blocked_by_default"
fi

source_commit="$(read_manifest_field "$package_manifest" "source_commit" || true)"
reference_commit="$(read_manifest_field "$package_manifest" "reference_commit" || true)"
if [[ -z "$source_commit" || -z "$reference_commit" ]]; then
	block_enablement "package manifest missing source or reference commit" "passed" "blocked_by_default"
fi

write_ledger "ready" "ready" "passed" "blocked_by_default" "" "$source_commit" "$reference_commit"
printf '[phase21-live-mining-package] ledger=%s\n' "$enablement_ledger"
