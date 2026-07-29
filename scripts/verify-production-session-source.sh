#!/usr/bin/env bash
set -euo pipefail

verify_adapter_contract() {
	local adapter_owner_source="$1"
	local adapter_test_source="$2"
	local owner_contract
	local contract_present
	local adapter_count
	local -a owner_contracts=(
		"mod asic_worker;"
		"mod transport;"
		"const OWNER_STACK_BYTES: usize = 16 * 1024;"
		"const NOTIFICATION_CAPACITY: usize = 16;"
		"const AUTHORITATIVE_REREAD_INTERVAL: Duration = Duration::from_secs(1);"
		"enum OwnerInboxMessage"
		"sender.try_send(OwnerInboxMessage::Wake(wakeup))"
		"ProductionSessionNotificationOutcome::Coalesced"
		"receiver.recv_timeout(AUTHORITATIVE_REREAD_INTERVAL)"
		"adapter.event_from_inbox(message, now_ms)"
		"drive_session(&mut session, &mut adapter, event, now_ms)"
		"adapter.maybe_execute(effect, now_ms)"
	)

	for owner_contract in "${owner_contracts[@]}"; do
		if command -v rg >/dev/null 2>&1; then
			contract_present="$(rg -F "$owner_contract" "$adapter_owner_source" || true)"
		else
			contract_present="$(grep -F -- "$owner_contract" "$adapter_owner_source" || true)"
		fi
		if [[ -z "$contract_present" ]]; then
			printf 'production session source contract failed: owner contract missing: %s\n' "$owner_contract" >&2
			return 1
		fi
	done

	verify_source_excludes \
		"$adapter_owner_source" \
		'TcpStream|TcpListener|std::net|write_all' \
		"production owner contains raw socket I/O"
	verify_source_excludes \
		"$adapter_owner_source" \
		'EspNvs|EspDefaultNvsPartition|pool(URL|Port|User|Password)|pool_(url|port|user|password)' \
		"production owner contains raw pool secrets or NVS access"
	verify_source_excludes \
		"$adapter_owner_source" \
		'ProductionAsicExecutor|I2cDriver|EspI2c|PinDriver|apply_negotiated_version_mask|execute_production_command|try_read_production_result' \
		"production owner contains raw device primitives"

	if command -v rg >/dev/null 2>&1; then
		adapter_count="$(
			{
				rg -F 'struct OrdinaryEspProductionSessionAdapter' "$adapter_owner_source" || true
				rg -F 'struct DeterministicProductionSessionAdapter' \
					"$adapter_test_source" || true
			} | wc -l | tr -d ' '
		)"
	else
		adapter_count="$(
			{
				grep -F -- 'struct OrdinaryEspProductionSessionAdapter' "$adapter_owner_source" || true
				grep -F -- 'struct DeterministicProductionSessionAdapter' \
					"$adapter_test_source" || true
			} | wc -l | tr -d ' '
		)"
	fi
	if [[ "$adapter_count" != "2" ]]; then
		printf 'production session source contract failed: expected exactly two adapters, found %s\n' "$adapter_count" >&2
		return 1
	fi
}

verify_source_excludes() {
	local source="$1"
	local forbidden_pattern="$2"
	local failure_message="$3"
	local forbidden_matches

	if command -v rg >/dev/null 2>&1; then
		forbidden_matches="$(rg -n "$forbidden_pattern" "$source" || true)"
	else
		forbidden_matches="$(grep -n -E -- "$forbidden_pattern" "$source" || true)"
	fi
	if [[ -n "$forbidden_matches" ]]; then
		printf '%s\n' "$forbidden_matches"
		printf 'production session source contract failed: %s\n' "$failure_message" >&2
		return 1
	fi
}

verify_deep_engine_contract() {
	local source
	for source in "$@"; do
		verify_source_excludes \
			"$source" \
			'(^|[^[:alnum:]_])(Instant|SystemTime)([^[:alnum:]_]|$)' \
			"reusable engine owns a real clock"
		verify_source_excludes \
			"$source" \
			'TcpStream|TcpListener|std::net|EspNvs|EspDefaultNvsPartition|pool(URL|Port|User|Password)|pool_(url|port|user|password)' \
			"reusable engine owns raw transport or secret I/O"
		verify_source_excludes \
			"$source" \
			'ProductionAsicExecutor|I2cDriver|EspI2c|PinDriver' \
			"reusable engine owns device primitives"
	done
}

if [[ "${1:-}" == "--verify-adapter-contract" ]]; then
	if [[ "$#" != "3" ]]; then
		printf 'usage: %s --verify-adapter-contract OWNER_SOURCE DETERMINISTIC_ADAPTER_SOURCE\n' "$0" >&2
		exit 2
	fi

	verify_adapter_contract "$2" "$3"
	printf 'production_session_adapter_contract=passed\n'
	exit 0
fi

if [[ "${1:-}" == "--verify-engine-clock-contract" ]]; then
	if [[ "$#" -lt "2" ]]; then
		printf 'usage: %s --verify-engine-clock-contract ENGINE_SOURCE...\n' "$0" >&2
		exit 2
	fi

	verify_deep_engine_contract "${@:2}"
	printf 'production_session_engine_clock_contract=passed\n'
	exit 0
fi

if [[ "${1:-}" == "--verify-deep-engine-contract" ]]; then
	if [[ "$#" -lt "2" ]]; then
		printf 'usage: %s --verify-deep-engine-contract ENGINE_SOURCE...\n' "$0" >&2
		exit 2
	fi

	verify_deep_engine_contract "${@:2}"
	printf 'production_session_deep_engine_contract=passed\n'
	exit 0
fi

if [[ "$#" != "0" ]]; then
	printf 'usage: %s [--verify-adapter-contract OWNER_SOURCE DETERMINISTIC_ADAPTER_SOURCE | --verify-deep-engine-contract ENGINE_SOURCE...]\n' "$0" >&2
	exit 2
fi

readonly repo_root="${BUILD_WORKSPACE_DIRECTORY:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
cd "$repo_root"

readonly retired_pattern='BITAXE_MINING_EVIDENCE_MODE|phase[ _-]?21|phase[ _-]?25|phase[ _-]?27|phase28_evidence|controlled_runtime|controlled_mining_runtime|mining_evidence_mode|live_stratum_runtime'
readonly active_paths=(
	"Justfile"
	"crates/bitaxe-api/src/runtime_projection.rs"
	"crates/bitaxe-asic/src"
	"crates/bitaxe-safety/src"
	"crates/bitaxe-stratum/src"
	"firmware/bitaxe/build.rs"
	"firmware/bitaxe/src"
	"firmware/bitaxe/BUILD.bazel"
	"scripts/BUILD.bazel"
	"scripts/phase23-redacted-operator-evidence.sh"
	"tools/parity/src/mining_allow.rs"
	"tools/parity/src/safety_allow.rs"
	"tools/parity/src/operator_evidence.rs"
	"tools/parity/src/operator_evidence/profile.rs"
	"tools/parity/src/operator_evidence/generation.rs"
)

if command -v rg >/dev/null 2>&1; then
	retired_matches="$(rg -n -i "$retired_pattern" "${active_paths[@]}" || true)"
else
	retired_matches="$(grep -R -n -i -E -- "$retired_pattern" "${active_paths[@]}" || true)"
fi
if [[ -n "$retired_matches" ]]; then
	printf '%s\n' "$retired_matches"
	printf 'production session source contract failed: retired runtime reference remains\n' >&2
	exit 1
fi

readonly retired_files=(
	"crates/bitaxe-stratum/fixtures/v1/fake-pool-transcripts.json"
	"crates/bitaxe-stratum/src/v1/fake_pool.rs"
	"crates/bitaxe-stratum/src/v1/mining_loop.rs"
	"firmware/bitaxe/src/controlled_mining_runtime.rs"
	"firmware/bitaxe/src/live_stratum_runtime.rs"
	"firmware/bitaxe/src/mining_evidence_mode.rs"
	"firmware/bitaxe/src/safety_adapter/phase27_bring_up.rs"
	"scripts/phase21-live-mining-evidence.sh"
	"scripts/phase12-mining-smoke-preflight.sh"
	"scripts/phase36-substantive-evidence-test.sh"
	"scripts/phase25-live-stratum-evidence.sh"
	"scripts/phase27-live-hardware-bridge-evidence.sh"
	"scripts/phase28-evidence.sh"
)

for retired_file in "${retired_files[@]}"; do
	if [[ -e "$retired_file" ]]; then
		printf 'production session source contract failed: retired file remains: %s\n' "$retired_file" >&2
		exit 1
	fi
done

readonly owner_source="firmware/bitaxe/src/production_mining_session.rs"
readonly deterministic_adapter_source="crates/bitaxe-stratum/src/v1/production_session/tests.rs"
verify_adapter_contract "$owner_source" "$deterministic_adapter_source"

readonly engine_sources=(
	"crates/bitaxe-stratum/src/v1/bridge_orchestration.rs"
	"crates/bitaxe-stratum/src/v1/production_session.rs"
	"crates/bitaxe-stratum/src/v1/production_session/campaign.rs"
	"crates/bitaxe-stratum/src/v1/production_session/orchestration.rs"
	"crates/bitaxe-stratum/src/v1/production_session/runtime.rs"
	"crates/bitaxe-stratum/src/v1/production_session/runtime/transport.rs"
	"crates/bitaxe-stratum/src/v1/production_session/types.rs"
	"crates/bitaxe-stratum/src/v1/recovery_policy.rs"
)
readonly engine_contracts=(
	"pub enum ProductionSessionEvent"
	"pub enum ProductionSessionEffect"
	"pub struct ProductionSessionSnapshot"
	"pub struct MiningHardwareProfile"
	"pub struct MiningCampaignLease"
	"FirstSubmitResponse"
	"ActiveDuration"
	"HardwarePrepared"
	"HardwareSafeStopConfirmed"
	"PrepareHardware"
	"SafeStopHardware"
	"StratumLineFramer"
	"LiveStratumRuntime"
	"BridgeOrchestrator"
	"classify_submit_response"
)

for engine_contract in "${engine_contracts[@]}"; do
	if command -v rg >/dev/null 2>&1; then
		contract_present="$(rg -F "$engine_contract" "${engine_sources[@]}" || true)"
	else
		contract_present="$(grep -F -- "$engine_contract" "${engine_sources[@]}" || true)"
	fi
	if [[ -z "$contract_present" ]]; then
		printf 'production session source contract failed: engine contract missing: %s\n' "$engine_contract" >&2
		exit 1
	fi
done

verify_deep_engine_contract "${engine_sources[@]}"

readonly forbidden_active_pattern='pub mod (fake_pool|live_runtime|mining_loop)|ProductionSessionAction|production_asic_ready|mining_loop_status|FakePoolTranscript|run_live_runtime|phase36_substantive_evidence_test'
readonly forbidden_active_paths=(
	"crates/bitaxe-api/src"
	"crates/bitaxe-stratum/src"
	"firmware/bitaxe/src"
	"crates/bitaxe-stratum/BUILD.bazel"
	"firmware/bitaxe/BUILD.bazel"
	"scripts/BUILD.bazel"
)

if command -v rg >/dev/null 2>&1; then
	forbidden_active_matches="$(rg -n "$forbidden_active_pattern" "${forbidden_active_paths[@]}" || true)"
else
	forbidden_active_matches="$(grep -R -n -E -- "$forbidden_active_pattern" "${forbidden_active_paths[@]}" || true)"
fi
if [[ -n "$forbidden_active_matches" ]]; then
	printf '%s\n' "$forbidden_active_matches"
	printf 'production session source contract failed: superseded active surface remains\n' >&2
	exit 1
fi

printf 'production_session_source_contract=passed\n'
