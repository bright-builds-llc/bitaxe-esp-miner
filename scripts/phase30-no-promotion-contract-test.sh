#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly workspace_root="${PHASE30_WORKSPACE_ROOT:-$(cd "${script_dir}/.." && pwd)}"
readonly disposition_path="${workspace_root}/docs/parity/evidence/phase-30-live-share-outcome-and-verified-promotion/disposition.md"
readonly conclusion_path="${workspace_root}/docs/parity/evidence/phase-30-live-share-outcome-and-verified-promotion/conclusion.md"
readonly checklist_path="${workspace_root}/docs/parity/checklist.md"
readonly requirements_path="${workspace_root}/.planning/REQUIREMENTS.md"
readonly archived_verification_path="${workspace_root}/.planning/milestones/v1.1-phases/28.1.1-bm1366-nonce-production-wire-parity/28.1.1-VERIFICATION.md"
readonly validation_path="${workspace_root}/.planning/phases/28.1-live-mining-blocker-fix-h4-w13-orchestration-parity-discrimi/28.1-VALIDATION.md"
readonly phase29_summary_path="${workspace_root}/docs/parity/evidence/phase-29-evidence-workflow-automation-closure/summary.md"
readonly denylist="${script_dir}/phase28.1.1-promoted-evidence-denylist.sh"

fail() {
	printf 'phase30_no_promotion_contract_test_error: category=%s\n' "$1" >&2
	exit 1
}

require_file() {
	[[ -f "$1" ]] || fail artifact-inventory
}

require_literal() {
	local path="$1"
	local expected="$2"
	local category="$3"

	rg -q -F -- "$expected" "$path" || fail "$category"
}

require_row_status() {
	local requirement_id="$1"
	local row

	row="$(rg -m 1 -F "| ${requirement_id} |" "$checklist_path")" || fail checklist-row
	[[ "$row" == *"| implemented |"* ]] || fail checklist-status
	[[ "$row" == *'phase30_disposition: no_promotion_no_eligible_evidence'* ]] || fail checklist-breadcrumb
}

require_pending_traceability() {
	local requirement_id="$1"
	local row

	row="$(rg -m 1 -F "| ${requirement_id} | Phase" "$requirements_path")" || fail requirements-traceability
	[[ "$row" == *'| Pending (gap closure) |'* ]] || fail requirements-status
}

for required_path in \
	"$disposition_path" \
	"$conclusion_path" \
	"$checklist_path" \
	"$requirements_path" \
	"$archived_verification_path" \
	"$validation_path" \
	"$phase29_summary_path" \
	"$denylist"; do
	require_file "$required_path"
done

while IFS= read -r required_field; do
	require_literal "$disposition_path" "$required_field" disposition-field
done <<'FIELDS'
phase30_disposition: no_promotion_no_eligible_evidence
new_evidence_input: none
archived_lineage_verification: gaps_found
eligible_share_outcome: none
hardware_accessed: false
credentials_accessed: false
raw_artifacts_committed: no
FIELDS

while IFS= read -r required_field; do
	require_literal "$conclusion_path" "$required_field" conclusion-field
done <<'FIELDS'
phase30_disposition: no_promotion_no_eligible_evidence
new_evidence_input: none
archived_lineage_verification: gaps_found
eligible_share_outcome: none
hardware_accessed: false
credentials_accessed: false
raw_artifacts_committed: no
FIELDS

require_literal "$conclusion_path" \
	'Phase completion is not requirement verification and does not satisfy STR-09, CFG-07, or ASIC-11.' \
	conclusion-non-verification

while IFS= read -r required_non_claim; do
	require_literal "$disposition_path" "$required_non_claim" exact-non-claim
done <<'NON_CLAIMS'
full active voltage/fan/thermal/fault/self-test safety
OTAWWW/recovery destructive or fault-injection behavior
non-205 boards
other ASIC families
Stratum v2
runtime UI/display/input/BAP
unbounded stress mining
NON_CLAIMS

for requirement_id in STR-09 CFG-07 ASIC-11; do
	require_literal "$disposition_path" "| ${requirement_id} | pending |" disposition-matrix
	require_literal "$conclusion_path" "| ${requirement_id} | not_promoted_pending |" conclusion-matrix
	require_row_status "$requirement_id"
	require_pending_traceability "$requirement_id"
done

while IFS= read -r required_non_claim; do
	require_literal "$conclusion_path" "$required_non_claim" conclusion-non-claim
done <<'NON_CLAIMS'
full active voltage/fan/thermal/fault/self-test safety
OTAWWW/recovery destructive or fault-injection behavior
non-205 boards
other ASIC families
Stratum v2
runtime UI/display/input/BAP
unbounded stress mining
NON_CLAIMS

require_literal "$archived_verification_path" 'status: gaps_found' archived-verification
require_literal "$archived_verification_path" 'verification_result: gaps_found' archived-verification
require_literal "$validation_path" 'status: closed_wont_do_unresolved' validation-status
require_literal "$validation_path" 'nyquist_compliant: true' validation-nyquist
require_literal "$validation_path" 'wave_0_complete: false' validation-wave-zero
require_literal "$validation_path" 'verification_result: gaps_found' validation-verification

umask 077
tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase30-no-promotion-contract.XXXXXX")"
readonly tmp_root
chmod 700 "$tmp_root"

cleanup() {
	rm -rf "$tmp_root"
}
trap cleanup EXIT

readonly aggregate_path="${tmp_root}/aggregate.txt"
cat "$disposition_path" "$conclusion_path" "$validation_path" >"$aggregate_path"
for requirement_id in STR-09 CFG-07 ASIC-11; do
	rg -m 1 -F "| ${requirement_id} |" "$checklist_path" >>"$aggregate_path"
done
chmod 600 "$aggregate_path"

set +e
"$denylist" "$aggregate_path" \
	>"${tmp_root}/denylist.stdout" \
	2>"${tmp_root}/denylist.stderr"
denylist_status=$?
set -e
[[ "$denylist_status" -eq 0 ]] || fail forbidden-evidence

scan_explicit_category() {
	local category="$1"
	local pattern="$2"
	local search_status

	set +e
	rg -q -i -e "$pattern" -- "$aggregate_path"
	search_status=$?
	set -e
	case "$search_status" in
	0) fail "$category" ;;
	1) ;;
	*) fail scanner ;;
	esac
}

scan_explicit_category macos-user-path '/Users/'
scan_explicit_category linux-home-path '/home/'
scan_explicit_category windows-drive-path "[A-Za-z]:\\\\"
scan_explicit_category ipv4-address '[0-9]{1,3}(\.[0-9]{1,3}){3}'
scan_explicit_category ipv6-address '([[:xdigit:]]{1,4}:){2,7}[[:xdigit:]]{0,4}'
scan_explicit_category mac-address '[[:xdigit:]]{2}(:[[:xdigit:]]{2}){5}'
scan_explicit_category url 'https?://'
scan_explicit_category credential-value '(pool(URL|Port|User|Password)|wifi-credentials|pool-credentials|password|token)[=:][[:space:]]*[^[:space:]]+'
scan_explicit_category raw-value '(hex|raw_value|register_value|frame_value|frame_hex|uart_hex|nonce_hex|target_hex)[=:][[:space:]]*(0x)?[0-9a-f]{6,}'

printf 'phase30_no_promotion_contract_test: passed\n'
