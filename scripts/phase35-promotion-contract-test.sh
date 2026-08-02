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
readonly phase35_generation="${workspace_root}/docs/parity/evidence/phase-35-detector-gated-correlated-evidence-and-exact-parity-promotion"
readonly phase35_manifest="${phase35_generation}/.phase35-generation-manifest.json"
readonly phase35_verdict="${phase35_generation}/admitted.json"
readonly phase35_matrix="${phase35_generation}/decision-matrix.json"
readonly phase35_projection="${phase35_generation}/projection.json"
readonly phase36_generation="${workspace_root}/docs/parity/evidence/phase-36-substantive-evidence-admission-and-exact-re-promotion"
readonly phase36_manifest="${phase36_generation}/manifest.json"
readonly phase36_verdict="${phase36_generation}/verdict.json"
readonly phase36_matrix="${phase36_generation}/decision-matrix.json"
readonly phase36_projection="${phase36_generation}/typed-fact-projection.json"
readonly phase36_checklist="${phase36_generation}/checklist.md"
readonly predecessor_revision_spec="${workspace_root}/docs/parity/checklist-revisions/2026-07-26-source-pointer-refresh.json"
readonly predecessor_revision_root="${workspace_root}/docs/parity/evidence/checklist-revisions/2026-07-26-source-pointer-refresh"
readonly predecessor_revision_manifest="${predecessor_revision_root}/manifest.json"
readonly predecessor_revision_snapshot="${predecessor_revision_root}/checklist.md"
readonly checklist_revision_spec="${workspace_root}/docs/parity/checklist-revisions/2026-07-27-module-ownership-refactor.json"
readonly checklist_revision_root="${workspace_root}/docs/parity/evidence/checklist-revisions/2026-07-27-module-ownership-refactor"
readonly checklist_revision_manifest="${checklist_revision_root}/manifest.json"
readonly checklist_revision_snapshot="${checklist_revision_root}/checklist.md"
readonly current_revision_spec="${workspace_root}/docs/parity/checklist-revisions/2026-07-28-runtime-display-documentation.json"
readonly current_revision_root="${workspace_root}/docs/parity/evidence/checklist-revisions/2026-07-28-runtime-display-documentation"
readonly current_revision_manifest="${current_revision_root}/manifest.json"
readonly current_revision_snapshot="${current_revision_root}/checklist.md"
readonly comprehensive_revision_spec="${workspace_root}/docs/parity/checklist-revisions/2026-08-02-comprehensive-reference-inventory.json"
readonly reference_surface_inventory="${workspace_root}/docs/parity/reference-surface-inventory.json"

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
	"$phase30_disposition" \
	"$phase35_manifest" \
	"$phase35_verdict" \
	"$phase35_matrix" \
	"$phase35_projection" \
	"$phase36_manifest" \
	"$phase36_verdict" \
	"$phase36_matrix" \
	"$phase36_projection" \
	"$phase36_checklist" \
	"$predecessor_revision_spec" \
	"$predecessor_revision_manifest" \
	"$predecessor_revision_snapshot" \
	"$checklist_revision_spec" \
	"$checklist_revision_manifest" \
	"$checklist_revision_snapshot" \
	"$current_revision_spec" \
	"$current_revision_manifest" \
	"$current_revision_snapshot" \
	"$comprehensive_revision_spec" \
	"$reference_surface_inventory"; do
	[[ -f "$required_path" ]] || fail artifact-inventory
done

sha256_file() {
	shasum -a 256 "$1" | awk '{print $1}'
}

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
	find "$evidence_root" -type f -print 2>/dev/null |
		LC_ALL=C sort >"${tmp_root}/evidence-${suffix}.txt"
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
cmp -s "${tmp_root}/checklist-before.txt" "${tmp_root}/checklist-after.txt" ||
	fail checklist-mutated
cmp -s "${tmp_root}/evidence-before.txt" "${tmp_root}/evidence-after.txt" ||
	fail evidence-inventory-mutated

for test_name in \
	phase35_promotion_is_complete_and_uses_one_root_digest \
	phase35_promotion_preserves_every_non_allowlisted_row_byte_identically \
	phase35_promotion_rejects_administrative_artifact_sources \
	phase35_promotion_rechecks_every_live_gate; do
	rg -q -F "$test_name" "${tmp_root}/promotion-tests.txt" ||
		fail promotion-test-coverage
done

for test_name in \
	phase35_publication_atomically_admits_one_complete_redacted_generation \
	phase35_publication_failures_preserve_previous_generation_and_checklist \
	phase35_publication_rejects_fingerprint_drift_before_exchange; do
	rg -q -F "$test_name" "${tmp_root}/publication-tests.txt" ||
		fail publication-test-coverage
done

readonly allowlisted_rows=(
	V12-HOSTNAME-205
	V12-PACKAGE-IDENTITY-205
	V12-OPERATOR-SNAPSHOT-205
	V12-RUNTIME-HEALTH-205
)
hostname_row="$(rg -m 1 -F '| V12-HOSTNAME-205 |' "$checklist_path")" ||
	fail allowlisted-row-missing
[[ "$hostname_row" == *"| verified | hardware-smoke |"* ]] ||
	fail preserved-hostname-row
for row_id in \
	V12-PACKAGE-IDENTITY-205 \
	V12-OPERATOR-SNAPSHOT-205 \
	V12-RUNTIME-HEALTH-205; do
	row="$(rg -m 1 -F "| ${row_id} |" "$checklist_path")" ||
		fail allowlisted-row-missing
	[[ "$row" == *"| implemented | workflow |"* ]] ||
		fail successor-correction-missing
done

generation_files="$(
	find "$phase35_generation" -maxdepth 1 \( -type f -o -type l \) -exec basename {} \; |
		LC_ALL=C sort | tr '\n' ' '
)"
readonly expected_generation_files='.phase35-generation-manifest.json admitted.json decision-matrix.json projection.json '
[[ "$generation_files" == "$expected_generation_files" ]] ||
	fail admitted-generation-inventory

root_digest="$(jq -er '.root_digest | select(test("^[0-9a-f]{64}$"))' "$phase35_projection")" ||
	fail projection-root-digest
[[ "$(jq -er '.schema' "$phase35_projection")" == "phase35-evidence-v1" ]] ||
	fail projection-schema
[[ "$(jq -er '.admitted' "$phase35_verdict")" == "true" ]] ||
	fail admission-verdict
[[ "$(jq -er '.evidence_root_digest' "$phase35_verdict")" == "$root_digest" ]] ||
	fail verdict-root-digest
[[ "$(jq -er '.evidence_root_digest' "$phase35_matrix")" == "$root_digest" ]] ||
	fail matrix-root-digest
[[ "$(jq -er '.root_digest' "$phase35_manifest")" == "$root_digest" ]] ||
	fail manifest-root-digest
[[ "$(jq -er '.schema' "$phase35_manifest")" == "phase35-generation-v1" ]] ||
	fail manifest-schema
[[ "$(sha256_file "$phase35_projection")" == "$(jq -er '.projection_sha256' "$phase35_manifest")" ]] ||
	fail projection-digest
[[ "$(sha256_file "$phase35_matrix")" == "$(jq -er '.matrix_sha256' "$phase35_manifest")" ]] ||
	fail matrix-digest
[[ "$(jq '[.scope_decisions[][1] | select(.decision == "promote")] | length' "$phase35_matrix")" -eq 4 ]] ||
	fail promotion-count
[[ "$(jq '[.scope_decisions[][1] | select(.decision == "do_not_promote")] | length' "$phase35_matrix")" -eq 11 ]] ||
	fail non-promotion-count
[[ "$(jq '.preserved_row_fingerprints | length' "$phase35_matrix")" -eq 87 ]] ||
	fail preserved-row-count
for row_id in "${allowlisted_rows[@]}"; do
	jq -e --arg row_id "$row_id" \
		'.scope_decisions | any(.[1].decision == "promote" and .[1].row_id == $row_id)' \
		"$phase35_matrix" >/dev/null || fail matrix-allowlist
done

phase35_generation_digest="$(sha256_file "$phase35_manifest")"
[[ "$(jq -er '.schema_version' "$phase36_manifest")" == "phase36-generation-v1" ]] ||
	fail successor-manifest-schema
[[ "$(jq -er '.phase35_root_digest' "$phase36_manifest")" == "$root_digest" ]] ||
	fail successor-root-digest
[[ "$(jq -er '.superseded_phase35_generation_digest' "$phase36_manifest")" == "$phase35_generation_digest" ]] ||
	fail successor-generation-link
[[ "$(jq -er '.checklist_fingerprint_before' "$phase36_manifest")" == "$(jq -er '.checklist_sha256' "$phase35_manifest")" ]] ||
	fail successor-checklist-link
[[ "$(jq -er '.checklist_fingerprint_after' "$phase36_manifest")" == "$(sha256_file "$phase36_checklist")" ]] ||
	fail successor-current-checklist-digest
[[ "$(jq -er '.checklist_sha256' "$phase36_manifest")" == "$(sha256_file "$phase36_checklist")" ]] ||
	fail successor-authoritative-checklist-digest
[[ "$(sha256_file "$phase36_projection")" == "$(jq -er '.projection_sha256' "$phase36_manifest")" ]] ||
	fail successor-projection-digest
[[ "$(sha256_file "$phase36_matrix")" == "$(jq -er '.matrix_sha256' "$phase36_manifest")" ]] ||
	fail successor-matrix-digest
[[ "$(sha256_file "$phase36_verdict")" == "$(jq -er '.verdict_sha256' "$phase36_manifest")" ]] ||
	fail successor-verdict-digest
jq -e \
	'.supported_rows == ["V12-HOSTNAME-205"] and .complete_matrix == true' \
	"$phase36_verdict" >/dev/null || fail successor-verdict
jq -e \
	'.scope_decisions | any(.decision == "preserve" and .row_id == "V12-HOSTNAME-205")' \
	"$phase36_matrix" >/dev/null || fail successor-hostname-decision
for correction in \
	'V12-PACKAGE-IDENTITY-205:runtime_identity_observation_insufficient' \
	'V12-OPERATOR-SNAPSHOT-205:snapshot_substance_insufficient' \
	'V12-RUNTIME-HEALTH-205:runtime_health_insufficient'; do
	row_id="${correction%%:*}"
	reason="${correction#*:}"
	jq -e --arg row_id "$row_id" --arg reason "$reason" \
		'.scope_decisions | any(.decision == "demote" and .row_id == $row_id and .reason == $reason)' \
		"$phase36_matrix" >/dev/null || fail successor-exact-correction
done

[[ "$(jq -er '.schema_version' "$predecessor_revision_spec")" == "parity-checklist-documentation-revision-v1" ]] ||
	fail predecessor-documentation-revision-spec-schema
[[ "$(jq -er '.schema_version' "$predecessor_revision_manifest")" == "parity-checklist-documentation-manifest-v1" ]] ||
	fail predecessor-documentation-revision-manifest-schema
[[ "$(jq -er '.revision_id' "$predecessor_revision_manifest")" == "$(jq -er '.revision_id' "$predecessor_revision_spec")" ]] ||
	fail predecessor-documentation-revision-id
[[ "$(jq -er '.predecessor_sha256' "$predecessor_revision_manifest")" == "$(sha256_file "$phase36_checklist")" ]] ||
	fail predecessor-documentation-revision-predecessor
[[ "$(jq -er '.predecessor_sha256' "$predecessor_revision_spec")" == "$(sha256_file "$phase36_checklist")" ]] ||
	fail predecessor-documentation-revision-spec-predecessor
[[ "$(jq -er '.change_spec_sha256' "$predecessor_revision_manifest")" == "$(sha256_file "$predecessor_revision_spec")" ]] ||
	fail predecessor-documentation-revision-spec-digest
[[ "$(jq -er '.checklist_sha256' "$predecessor_revision_manifest")" == "$(sha256_file "$predecessor_revision_snapshot")" ]] ||
	fail predecessor-documentation-revision-checklist-digest
[[ "$(jq -er '.affected_rows | length' "$predecessor_revision_manifest")" -eq 17 ]] ||
	fail predecessor-documentation-revision-row-count

[[ "$(jq -er '.schema_version' "$checklist_revision_spec")" == "parity-checklist-documentation-revision-v1" ]] ||
	fail documentation-revision-spec-schema
[[ "$(jq -er '.schema_version' "$checklist_revision_manifest")" == "parity-checklist-documentation-manifest-v1" ]] ||
	fail documentation-revision-manifest-schema
[[ "$(jq -er '.revision_id' "$checklist_revision_manifest")" == "$(jq -er '.revision_id' "$checklist_revision_spec")" ]] ||
	fail documentation-revision-id
[[ "$(jq -er '.predecessor_sha256' "$checklist_revision_manifest")" == "$(sha256_file "$predecessor_revision_snapshot")" ]] ||
	fail documentation-revision-predecessor
[[ "$(jq -er '.predecessor_sha256' "$checklist_revision_spec")" == "$(sha256_file "$predecessor_revision_snapshot")" ]] ||
	fail documentation-revision-spec-predecessor
[[ "$(jq -er '.change_spec_sha256' "$checklist_revision_manifest")" == "$(sha256_file "$checklist_revision_spec")" ]] ||
	fail documentation-revision-spec-digest
[[ "$(jq -er '.checklist_sha256' "$checklist_revision_manifest")" == "$(sha256_file "$checklist_revision_snapshot")" ]] ||
	fail documentation-revision-checklist-digest
[[ "$(jq -er '.affected_rows | length' "$checklist_revision_manifest")" -eq 5 ]] ||
	fail documentation-revision-row-count
jq -e \
	'.affected_rows == ["CFG-004", "OTA-001", "OTA-002", "REL-003", "EVD-08"]' \
	"$checklist_revision_manifest" >/dev/null ||
	fail documentation-revision-row-set
for target in \
	'crates/bitaxe-config/src/nvs/schema.rs' \
	'firmware/bitaxe/src/http_api/updates.rs' \
	'tools/flash/src/commands/flash.rs' \
	'tools/flash/src/package.rs' \
	'tools/parity/src/report/validation/phase26.rs'; do
	rg -q -F "$target" "$checklist_revision_snapshot" ||
		fail documentation-revision-target
done

[[ "$(jq -er '.schema_version' "$current_revision_spec")" == "parity-checklist-documentation-revision-v1" ]] ||
	fail current-documentation-revision-spec-schema
[[ "$(jq -er '.schema_version' "$current_revision_manifest")" == "parity-checklist-documentation-manifest-v1" ]] ||
	fail current-documentation-revision-manifest-schema
[[ "$(jq -er '.revision_id' "$current_revision_manifest")" == "$(jq -er '.revision_id' "$current_revision_spec")" ]] ||
	fail current-documentation-revision-id
[[ "$(jq -er '.predecessor_sha256' "$current_revision_manifest")" == "$(sha256_file "$checklist_revision_snapshot")" ]] ||
	fail current-documentation-revision-predecessor
[[ "$(jq -er '.predecessor_sha256' "$current_revision_spec")" == "$(sha256_file "$checklist_revision_snapshot")" ]] ||
	fail current-documentation-revision-spec-predecessor
[[ "$(jq -er '.change_spec_sha256' "$current_revision_manifest")" == "$(sha256_file "$current_revision_spec")" ]] ||
	fail current-documentation-revision-spec-digest
[[ "$(jq -er '.checklist_sha256' "$current_revision_manifest")" == "$(sha256_file "$current_revision_snapshot")" ]] ||
	fail current-documentation-revision-checklist-digest
jq -e \
	'.affected_rows == ["IO-001", "UI-001", "UI-002", "UI-003"]' \
	"$current_revision_manifest" >/dev/null ||
	fail current-documentation-revision-row-set
for marker in \
	'display_input_status=runtime_gap reason=full_parity_pending display_runtime=alternating_debug input_runtime=unavailable' \
	"uptime milliseconds \`0..=7_999\`" \
	'complete 512-byte SSD1306 framebuffer' \
	'does not promote the full upstream display carousel'; do
	rg -q -F "$marker" "$current_revision_snapshot" ||
		fail current-documentation-revision-marker
done
for row_id in IO-001 UI-001 UI-002 UI-003; do
	row="$(rg -m 1 -F "| ${row_id} |" "$current_revision_snapshot")" ||
		fail current-documentation-revision-row-missing
	[[ "$row" == *"| in-progress |"* ]] ||
		fail current-documentation-revision-status-promoted
done
[[ "$(jq -er '.schema_version' "$comprehensive_revision_spec")" == "parity-checklist-comprehensive-revision-v1" ]] ||
	fail comprehensive-revision-spec-schema
[[ "$(jq -er '.revision_id' "$comprehensive_revision_spec")" == "2026-08-02-comprehensive-reference-inventory" ]] ||
	fail comprehensive-revision-id
[[ "$(jq -er '.predecessor_sha256' "$comprehensive_revision_spec")" == "$(sha256_file "$current_revision_snapshot")" ]] ||
	fail comprehensive-revision-predecessor
[[ "$(jq -er '.checklist_sha256' "$comprehensive_revision_spec")" == "$(sha256_file "$checklist_path")" ]] ||
	fail comprehensive-revision-root-drift
[[ "$(jq -er '.inventory_sha256' "$comprehensive_revision_spec")" == "$(sha256_file "$reference_surface_inventory")" ]] ||
	fail comprehensive-revision-inventory-drift

for preserved_row in STR-09 CFG-07 ASIC-11; do
	row="$(rg -m 1 -F "| ${preserved_row} |" "$checklist_path")" ||
		fail phase30-row-missing
	[[ "$row" == *"| implemented |"* ]] || fail phase30-row-promoted
	[[ "$row" == *"phase30_disposition: no_promotion_no_eligible_evidence"* ]] ||
		fail phase30-row-breadcrumb
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
