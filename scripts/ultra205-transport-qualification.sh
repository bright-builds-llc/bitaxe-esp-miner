#!/usr/bin/env bash
source "${BASH_SOURCE[0]%/*}/phase28.1.1-terminal-closure-guard.sh"
# Shared authority for the private Ultra 205 transport qualification contract.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly repo_root
readonly expected_firmware_head="e622253d2fc4aea4589e0dcf5524081b6b054aaf"
readonly -a native_contract_paths=(
	"scripts/phase13-os-native-reader.pl"
	"scripts/phase13-monitor-capture.sh"
	"scripts/serial-session-trace.sh"
	"scripts/detect-ultra205.sh"
	"scripts/ultra205-late-attach-broker.sh"
	"scripts/ultra205-late-attach-worker.sh"
	"scripts/ultra205-late-attach-classifier.mjs"
	"scripts/phase28.1.1-accepted-state-diagnostic.sh"
	"scripts/phase28.1.1-exact-head-hardware-attempt.sh"
)
readonly -a uart_contract_paths=(
	"scripts/diagnose-ultra205-uart-capture.sh"
	"scripts/ultra205-uart-capture-broker.sh"
	"scripts/ultra205-uart-capture-worker.sh"
	"scripts/ultra205-uart-capture-classifier.mjs"
	"scripts/phase13-monitor-capture.sh"
	"scripts/phase13-uart-native-reader.py"
	"scripts/serial-session-trace.sh"
	"scripts/detect-ultra205.sh"
	"scripts/phase28.1.1-accepted-state-diagnostic.sh"
	"scripts/phase28.1.1-exact-head-hardware-attempt.sh"
)

die() {
	printf 'transport_qualification_error=%s\n' "$1" >&2
	exit 1
}

usage() {
	printf 'usage: %s contract-digest | native-contract-digest | validate PATH [EXPECTED_TOOL_HEAD] | validate-native PATH [EXPECTED_TOOL_HEAD]\n' "$(basename "$0")" >&2
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

digest_contract_paths() {
	local relative_path
	for relative_path in "$@"; do
		[[ -f "$repo_root/$relative_path" ]] || die "contract_file_missing"
	done
	{
		for relative_path in "$@"; do
			printf '%s\0%s\0' \
				"$relative_path" \
				"$(shasum -a 256 "$repo_root/$relative_path" | awk '{print $1}')"
		done
	} | shasum -a 256 | awk '{print $1}'
}

contract_digest() {
	digest_contract_paths "${uart_contract_paths[@]}"
}

native_contract_digest() {
	digest_contract_paths "${native_contract_paths[@]}"
}

validate_uart_summary() {
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
		    "native_preflight_heartbeat_count",
		    "uart_preflight_heartbeat_count",
		    "cold_uart_heartbeat_count",
		    "native_physical_identity_stable",
		    "native_new_enumeration_epoch",
		    "uart_physical_identity_stable",
		    "uart_enumeration_identity_stable",
		    "quiet_boundary_complete",
		    "original_boot_present",
		    "original_listener_present",
		    "boot_evidence_complete",
		    "accepted_state_stages_complete",
		    "heartbeat_monotonic",
		    "listener_ready",
		    "soak_complete",
		    "cleanup_complete",
		    "adapter_binding_sha256",
		    "diagnostic_contract_digest_sha256",
		    "trace_digest_sha256"
		  ] | sort) and
		  .schema_version == "ultra205-transport-qualification-v3" and
		  .tool_head == $expected_tool_head and
		  .expected_firmware_head == $expected_firmware_head and
		  .classification_category == "uart_cold_delivers" and
		  (.native_preflight_heartbeat_count | type == "number" and floor == . and . >= 1) and
		  (.uart_preflight_heartbeat_count | type == "number" and floor == . and . >= 1) and
		  (.cold_uart_heartbeat_count | type == "number" and floor == . and . >= 3) and
		  .native_physical_identity_stable == true and
		  .native_new_enumeration_epoch == true and
		  .uart_physical_identity_stable == true and
		  .uart_enumeration_identity_stable == true and
		  .quiet_boundary_complete == true and
		  .original_boot_present == true and
		  .original_listener_present == true and
		  .boot_evidence_complete == true and
		  .accepted_state_stages_complete == true and
		  .heartbeat_monotonic == true and
		  .listener_ready == true and
		  .soak_complete == true and
		  .cleanup_complete == true and
		  (.adapter_binding_sha256 | type == "string" and test("^[0-9a-f]{64}$")) and
		  .diagnostic_contract_digest_sha256 == $expected_contract_digest and
		  (.trace_digest_sha256 | type == "string" and test("^[0-9a-f]{64}$"))
		' "$summary_path" >/dev/null || die "qualification_invalid"
}

validate_native_summary() {
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
	expected_contract_digest="$(native_contract_digest)"
	jq -e \
		--arg expected_tool_head "$expected_tool_head" \
		--arg expected_firmware_head "$expected_firmware_head" \
		--arg expected_contract_digest "$expected_contract_digest" '
		  type == "object" and
		  (keys | sort) == ([
		    "schema_version",
		    "tool_head",
		    "expected_firmware_head",
		    "attempt_id",
		    "owner_fingerprint_sha256",
		    "owner_process_count",
		    "classification_category",
		    "capture_seconds",
		    "preflight_native_heartbeat_count",
		    "cold_native_heartbeat_count",
		    "application_byte_count",
		    "physical_identity_sha256",
		    "preflight_enumeration_identity_sha256",
		    "cold_enumeration_identity_sha256",
		    "preflight_session_sha256",
		    "cold_session_sha256",
		    "physical_identity_stable",
		    "new_enumeration_epoch",
		    "distinct_cold_session",
		    "heartbeat_monotonic",
		    "listener_ready",
		    "boot_evidence_replay_complete",
		    "accepted_state_replay_complete",
		    "soak_complete",
		    "cleanup_complete",
		    "owner_cleanup_complete",
		    "holder_cleanup_complete",
		    "socket_cleanup_complete",
		    "live_process_count",
		    "serial_holder_count",
		    "live_socket_count",
		    "diagnostic_contract_digest_sha256",
		    "trace_digest_sha256"
		  ] | sort) and
		  .schema_version == "ultra205-transport-qualification-v2" and
		  .tool_head == $expected_tool_head and
		  .expected_firmware_head == $expected_firmware_head and
		  (.attempt_id | type == "string" and test("^[0-9a-f]{32}$")) and
		  (.owner_fingerprint_sha256 | type == "string" and test("^[0-9a-f]{64}$")) and
		  .owner_process_count == 1 and
		  .classification_category == "native_cold_delivers" and
		  (.capture_seconds | type == "number" and floor == . and . >= 360) and
		  (.preflight_native_heartbeat_count | type == "number" and floor == . and . >= 1) and
		  (.cold_native_heartbeat_count | type == "number" and floor == . and . >= 3) and
		  (.application_byte_count | type == "number" and floor == . and . >= 1) and
		  (.physical_identity_sha256 | type == "string" and test("^[0-9a-f]{64}$")) and
		  (.preflight_enumeration_identity_sha256 | type == "string" and test("^[0-9a-f]{64}$")) and
		  (.cold_enumeration_identity_sha256 | type == "string" and test("^[0-9a-f]{64}$")) and
		  .preflight_enumeration_identity_sha256 != .cold_enumeration_identity_sha256 and
		  (.preflight_session_sha256 | type == "string" and test("^[0-9a-f]{64}$")) and
		  (.cold_session_sha256 | type == "string" and test("^[0-9a-f]{64}$")) and
		  .preflight_session_sha256 != .cold_session_sha256 and
		  .physical_identity_stable == true and
		  .new_enumeration_epoch == true and
		  .distinct_cold_session == true and
		  .heartbeat_monotonic == true and
		  .listener_ready == true and
		  .boot_evidence_replay_complete == true and
		  .accepted_state_replay_complete == true and
		  .soak_complete == true and
		  .cleanup_complete == true and
		  .owner_cleanup_complete == true and
		  .holder_cleanup_complete == true and
		  .socket_cleanup_complete == true and
		  .live_process_count == 0 and
		  .serial_holder_count == 0 and
		  .live_socket_count == 0 and
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
native-contract-digest)
	[[ $# -eq 1 ]] || die "unknown_argument"
	native_contract_digest
	;;
validate)
	[[ $# -ge 2 && $# -le 3 ]] || die "unknown_argument"
	validate_uart_summary "$2" "${3:-}"
	;;
validate-native)
	[[ $# -ge 2 && $# -le 3 ]] || die "unknown_argument"
	validate_native_summary "$2" "${3:-}"
	;;
-h | --help)
	usage
	;;
*)
	usage
	die "unknown_command"
	;;
esac
