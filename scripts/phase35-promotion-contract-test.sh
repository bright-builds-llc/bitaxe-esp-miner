#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir

if [[ -n "${TEST_SRCDIR:-}" && -n "${TEST_WORKSPACE:-}" ]]; then
	readonly workspace_root="${TEST_SRCDIR}/${TEST_WORKSPACE}"
	readonly parity_tests="${workspace_root}/tools/parity/tests"
else
	readonly workspace_root="${PHASE35_WORKSPACE_ROOT:-$(cd "${script_dir}/.." && pwd)}"
	readonly parity_tests="${workspace_root}/bazel-bin/tools/parity/tests"
fi

readonly checklist_path="${workspace_root}/docs/parity/checklist.md"
readonly evidence_root="${workspace_root}/docs/parity/evidence"
readonly types_path="${workspace_root}/tools/parity/src/phase35_promotion/types.rs"
readonly checklist_source="${workspace_root}/tools/parity/src/phase35_promotion/checklist.rs"
readonly evaluator_path="${workspace_root}/tools/parity/src/phase35_promotion/evaluator.rs"
readonly phase30_disposition="${workspace_root}/docs/parity/evidence/phase-30-live-share-outcome-and-verified-promotion/disposition.md"

fail() {
	printf 'phase35_promotion_contract_test_error: category=%s\n' "$1" >&2
	exit 1
}

for required_path in \
	"$parity_tests" \
	"$checklist_path" \
	"$types_path" \
	"$checklist_source" \
	"$evaluator_path" \
	"$phase30_disposition"; do
	[[ -f "$required_path" ]] || fail artifact-inventory
done

umask 077
tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase35-promotion-contract.XXXXXX")"
readonly tmp_root
chmod 700 "$tmp_root"

cleanup() {
	rm -rf "$tmp_root"
}
trap cleanup EXIT

snapshot_contract() {
	local suffix="$1"
	awk '/^\| / { print }' "$checklist_path" \
		>"${tmp_root}/checklist-${suffix}.txt"
	find "$evidence_root" -type f -print 2>/dev/null \
		| LC_ALL=C sort >"${tmp_root}/evidence-${suffix}.txt"
	chmod 600 \
		"${tmp_root}/checklist-${suffix}.txt" \
		"${tmp_root}/evidence-${suffix}.txt"
}

snapshot_contract before

"$parity_tests" phase35_promotion --test-threads=1 \
	>"${tmp_root}/promotion-tests.txt"
"$parity_tests" phase35_publication --test-threads=1 \
	>"${tmp_root}/publication-tests.txt"

snapshot_contract after
cmp -s "${tmp_root}/checklist-before.txt" "${tmp_root}/checklist-after.txt" \
	|| fail checklist-mutated
cmp -s "${tmp_root}/evidence-before.txt" "${tmp_root}/evidence-after.txt" \
	|| fail evidence-inventory-mutated

for test_name in \
	phase35_promotion_is_complete_and_uses_one_root_digest \
	phase35_promotion_preserves_every_non_allowlisted_row_byte_identically \
	phase35_promotion_rejects_administrative_artifact_sources \
	phase35_promotion_rechecks_every_live_gate; do
	rg -q -F "$test_name" "${tmp_root}/promotion-tests.txt" \
		|| fail promotion-test-coverage
done

for test_name in \
	phase35_publication_atomically_admits_one_complete_redacted_generation \
	phase35_publication_failures_preserve_previous_generation_and_checklist \
	phase35_publication_rejects_fingerprint_drift_before_exchange; do
	rg -q -F "$test_name" "${tmp_root}/publication-tests.txt" \
		|| fail publication-test-coverage
done

readonly allowlisted_rows=(
	V12-HOSTNAME-205
	V12-PACKAGE-IDENTITY-205
	V12-OPERATOR-SNAPSHOT-205
	V12-RUNTIME-HEALTH-205
)
for row_id in "${allowlisted_rows[@]}"; do
	row="$(rg -m 1 -F "| ${row_id} |" "$checklist_path")" \
		|| fail allowlisted-row-missing
	[[ "$row" == *"| implemented | unit,workflow |"* ]] \
		|| fail premature-promotion
	[[ "$row" == *"No eligible real root is admitted yet"* ]] \
		|| fail eligibility-boundary
done

for preserved_row in STR-09 CFG-07 ASIC-11; do
	row="$(rg -m 1 -F "| ${preserved_row} |" "$checklist_path")" \
		|| fail phase30-row-missing
	[[ "$row" == *"| implemented |"* ]] || fail phase30-row-promoted
	[[ "$row" == *"phase30_disposition: no_promotion_no_eligible_evidence"* ]] \
		|| fail phase30-row-breadcrumb
done
rg -q -F 'phase30_disposition: no_promotion_no_eligible_evidence' \
	"$phase30_disposition" || fail phase30-disposition
rg -q -F 'archived_lineage_verification: gaps_found' \
	"$phase30_disposition" || fail archived-lineage

while IFS= read -r reason; do
	rg -q -F "$reason" "$types_path" || fail exclusion-reason
done <<'REASONS'
ActiveControlExcluded
SelfTestEffectsExcluded
WatchdogInterventionExcluded
MiningStratumAsicExcluded
ArchivedPhase28_1_1Excluded
CredentialsExcluded
DirectUartOrPinsExcluded
OtaOrRecoveryExcluded
OtherBoardsExcluded
LifecycleTestOnlyProofExcluded
BroaderOrUnmappedRowExcluded
REASONS

promoted_rendering="$(
	sed -n '/cells\\[4\\] = "verified"/,/lines\\[row.line_index\\]/p' \
		"$checklist_source"
)"
if rg -qi \
	'(uart|pin|credential|mining|watchdog|self-test|ota|other-board|phase ?28)' \
	<<<"$promoted_rendering"; then
	fail excluded-category-in-promoted-row
fi

promote_mappings="$(
	sed -n '/fn decision_for_scope/,/fn reason_for_excluded_scope/p' \
		"$evaluator_path"
)"
for row_id in "${allowlisted_rows[@]}"; do
	rg -q -F "${row_id#V12-}" "$types_path" || fail allowlist-source
done
promote_count="$(
	rg -c 'Phase35PromotionDecision::Promote' <<<"$promote_mappings"
)"
[[ "$promote_count" -eq 1 ]] || fail open-promotion-constructor

printf 'phase35_promotion_contract_test: passed changed_non_allowlisted_rows=0 raw_canaries=0\n'
