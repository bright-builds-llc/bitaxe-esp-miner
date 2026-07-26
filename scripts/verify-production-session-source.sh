#!/usr/bin/env bash
set -euo pipefail

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
	"firmware/bitaxe/src/controlled_mining_runtime.rs"
	"firmware/bitaxe/src/live_stratum_runtime.rs"
	"firmware/bitaxe/src/mining_evidence_mode.rs"
	"firmware/bitaxe/src/safety_adapter/phase27_bring_up.rs"
	"scripts/phase21-live-mining-evidence.sh"
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
readonly owner_contracts=(
	"const OWNER_STACK_BYTES: usize = 16 * 1024;"
	"const NOTIFICATION_CAPACITY: usize = 8;"
	"const AUTHORITATIVE_REREAD_INTERVAL: Duration = Duration::from_secs(1);"
	"sender.try_send(wakeup)"
	"ProductionSessionNotificationOutcome::Coalesced"
	"receiver.recv_timeout(AUTHORITATIVE_REREAD_INTERVAL)"
)

for owner_contract in "${owner_contracts[@]}"; do
	if command -v rg >/dev/null 2>&1; then
		contract_present="$(rg -F "$owner_contract" "$owner_source" || true)"
	else
		contract_present="$(grep -F -- "$owner_contract" "$owner_source" || true)"
	fi
	if [[ -z "$contract_present" ]]; then
		printf 'production session source contract failed: owner contract missing: %s\n' "$owner_contract" >&2
		exit 1
	fi
done

printf 'production_session_source_contract=passed\n'
