#!/usr/bin/env bash
# Shared authority for the private Ultra 205 transport qualification contract.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly repo_root
readonly expected_firmware_head="e622253d2fc4aea4589e0dcf5524081b6b054aaf"
readonly -a contract_paths=(
	"scripts/diagnose-ultra205-late-attach.sh"
	"scripts/ultra205-late-attach-broker.sh"
	"scripts/ultra205-late-attach-worker.sh"
	"scripts/ultra205-late-attach-classifier.mjs"
	"scripts/phase13-monitor-capture.sh"
	"scripts/phase13-os-native-reader.pl"
)

die() {
	printf 'transport_qualification_error=%s\n' "$1" >&2
	exit 1
}

usage() {
	printf 'usage: %s contract-digest | validate PATH [EXPECTED_TOOL_HEAD]\n' "$(basename "$0")" >&2
}

mode_of_path() {
	if stat -f '%Lp' "$1" >/dev/null 2>&1; then
		stat -f '%Lp' "$1"
		return
	fi
	stat -c '%a' "$1"
}

owner_of_path() {
	if stat -f '%u' "$1" >/dev/null 2>&1; then
		stat -f '%u' "$1"
		return
	fi
	stat -c '%u' "$1"
}

contract_digest() {
	local relative_path
	for relative_path in "${contract_paths[@]}"; do
		[[ -f "$repo_root/$relative_path" ]] || die "contract_file_missing"
	done
	{
		for relative_path in "${contract_paths[@]}"; do
			printf '%s\0%s\0' \
				"$relative_path" \
				"$(shasum -a 256 "$repo_root/$relative_path" | awk '{print $1}')"
		done
	} | shasum -a 256 | awk '{print $1}'
}

validate_summary() {
	local summary_path="$1"
	local expected_tool_head="${2:-}"
	[[ -f "$summary_path" && ! -L "$summary_path" ]] || die "qualification_missing"
	[[ "$(mode_of_path "$summary_path")" == "600" ]] || die "qualification_mode_invalid"
	[[ "$(owner_of_path "$summary_path")" == "$(id -u)" ]] || die "qualification_owner_invalid"
	if [[ -z "$expected_tool_head" ]]; then
		expected_tool_head="$(git -C "$repo_root" rev-parse HEAD)"
	fi
	[[ "$expected_tool_head" =~ ^[0-9a-f]{40}$ ]] || die "qualification_tool_head_invalid"
	local expected_contract_digest
	expected_contract_digest="$(contract_digest)"
	jq -e \
		--arg expected_tool_head "$expected_tool_head" \
		--arg expected_firmware_head "$expected_firmware_head" \
		--arg expected_contract_digest "$expected_contract_digest" '
		  type == "object" and
		  (keys | sort) == ([
		    "schema_version",
		    "tool_head",
		    "expected_firmware_head",
		    "classification_category",
		    "preflight_espflash_heartbeat_count",
		    "preflight_os_native_heartbeat_count",
		    "cold_os_native_heartbeat_count",
		    "identity_stable",
		    "new_enumeration_epoch",
		    "soak_complete",
		    "cleanup_complete",
		    "diagnostic_contract_digest_sha256",
		    "trace_digest_sha256"
		  ] | sort) and
		  .schema_version == "ultra205-transport-qualification-v2" and
		  .tool_head == $expected_tool_head and
		  .expected_firmware_head == $expected_firmware_head and
		  .classification_category == "os_native_cold_delivers" and
		  (.preflight_espflash_heartbeat_count | type == "number" and floor == . and . >= 0) and
		  (.preflight_os_native_heartbeat_count | type == "number" and floor == . and . >= 1) and
		  (.cold_os_native_heartbeat_count | type == "number" and floor == . and . >= 3) and
		  .identity_stable == true and
		  .new_enumeration_epoch == true and
		  .soak_complete == true and
		  .cleanup_complete == true and
		  .diagnostic_contract_digest_sha256 == $expected_contract_digest and
		  (.trace_digest_sha256 | type == "string" and test("^[0-9a-f]{64}$"))
		' "$summary_path" >/dev/null || die "qualification_invalid"
}

command="${1:-}"
case "$command" in
contract-digest)
	[[ $# -eq 1 ]] || die "unknown_argument"
	contract_digest
	;;
validate)
	[[ $# -ge 2 && $# -le 3 ]] || die "unknown_argument"
	validate_summary "$2" "${3:-}"
	;;
-h | --help)
	usage
	;;
*)
	usage
	die "unknown_command"
	;;
esac
