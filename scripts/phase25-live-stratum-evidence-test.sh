#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly wrapper="${PHASE25_EVIDENCE_SCRIPT:-${script_dir}/phase25-live-stratum-evidence.sh}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase25-live-stratum-evidence-test.XXXXXX")"
readonly tmp_root

cleanup() {
	rm -rf "$tmp_root"
}
trap cleanup EXIT

assert_contains() {
	local path="$1"
	local needle="$2"

	if ! grep -Fq -- "$needle" "$path"; then
		printf 'Expected %s to contain: %s\n' "$path" "$needle" >&2
		exit 1
	fi
}

assert_not_contains() {
	local path="$1"
	local needle="$2"

	if grep -Fq -- "$needle" "$path"; then
		printf 'Expected %s not to contain: %s\n' "$path" "$needle" >&2
		exit 1
	fi
}

assert_file_exists() {
	local path="$1"

	if [[ ! -f "$path" ]]; then
		printf 'missing file: %s\n' "$path" >&2
		exit 1
	fi
}

write_fake_parity() {
	local path="$1"

	cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

printf 'fake_phase25_parity_args: %s\n' "$*"
printf 'mining_allow_status: passed\n'
SH
	chmod +x "$path"
}

write_fake_detector() {
	local path="$1"
	local exit_code="$2"

	cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

printf 'fake-detect-ultra205 invoked\n' >&2
exit "${PHASE25_FAKE_DETECT_EXIT:-0}"
SH
	chmod +x "$path"
	printf '%s\n' "$exit_code" >"${path}.exit"
}

assert_full_evidence_slots_exist() {
	local evidence_root="$1"
	local slot

	for slot in package detector board-info command log api websocket share-outcome safe-stop redaction-review summary; do
		assert_file_exists "${evidence_root}/${slot}.md"
	done
}

assert_detector_failure_slots_exist() {
	local evidence_root="$1"
	local slot

	for slot in detector board-info share-outcome safe-stop redaction-review summary; do
		assert_file_exists "${evidence_root}/${slot}.md"
	done

	for slot in package command log api websocket; do
		if [[ -e "${evidence_root}/${slot}.md" ]]; then
			printf 'detector failure must not create %s.md\n' "$slot" >&2
			exit 1
		fi
	done
}

assert_evidence_is_redacted() {
	local evidence_root="$1"
	local path

	for path in "${evidence_root}"/*.md; do
		assert_not_contains "$path" "sentinel-pool"
		assert_not_contains "$path" "sentinel-password"
		assert_not_contains "$path" "sentinel-token"
		assert_not_contains "$path" "sentinel-share"
		assert_not_contains "$path" "sentinel-extra"
		assert_not_contains "$path" "raw_bm1366_frame"
		assert_not_contains "$path" "192.0.2.55"
		assert_not_contains "$path" "bc1qsentinelowneraddress"
	done
}

run_blocked_mode_test() {
	local fake_parity="${tmp_root}/fake-parity.sh"
	local evidence_root="${tmp_root}/blocked-root"
	write_fake_parity "$fake_parity"

	PHASE25_PARITY_COMMAND="$fake_parity" \
	PHASE25_RAW_DIAGNOSTIC_SAMPLE="sentinel-pool sentinel-password sentinel-share sentinel-extra sentinel-token raw_bm1366_frame 192.0.2.55 bc1qsentinelowneraddress" \
	"$wrapper" \
		--evidence-root "$evidence_root" \
		--manifest "${tmp_root}/bitaxe-ultra205-package.json" \
		--mode blocked \
		--pool-credentials "${tmp_root}/sentinel-pool.invalid.json" \
		--wifi-credentials "${tmp_root}/SentinelWifi.json" \
		--device-url "http://192.0.2.55"

	assert_full_evidence_slots_exist "$evidence_root"
	assert_contains "${evidence_root}/command.md" "pool_config: local-owner-supplied"
	assert_contains "${evidence_root}/command.md" "wifi_config: local-owner-supplied"
	assert_contains "${evidence_root}/command.md" "raw_artifacts_committed: no"
	assert_contains "${evidence_root}/command.md" "raw_pool_values_committed: no"
	assert_contains "${evidence_root}/command.md" "network_scan: disabled"
	assert_contains "${evidence_root}/share-outcome.md" "share_outcome: blocked_safe_prerequisite"
	assert_contains "${evidence_root}/share-outcome.md" "accepted/rejected shares remain non-claims"
	assert_contains "${evidence_root}/safe-stop.md" "safe_stop_status: complete"
	assert_contains "${evidence_root}/safe-stop.md" "work_submission=disabled"
	assert_contains "${evidence_root}/redaction-review.md" "diagnostic_input_status: rejected_sensitive_raw_payload"
	assert_contains "${evidence_root}/summary.md" "raw_artifacts_committed: no"
	assert_contains "${evidence_root}/summary.md" "raw_pool_values_committed: no"
	assert_evidence_is_redacted "$evidence_root"
}

run_detector_failure_test() {
	local fake_parity="${tmp_root}/fake-parity-detector.sh"
	local fake_detector="${tmp_root}/fake-detect-ultra205.sh"
	local evidence_root="${tmp_root}/detector-root"
	write_fake_parity "$fake_parity"
	write_fake_detector "$fake_detector" 42

	set +e
	PHASE25_PARITY_COMMAND="$fake_parity" \
	PHASE25_DETECT_COMMAND="$fake_detector" \
	PHASE25_FAKE_DETECT_EXIT=42 \
	"$wrapper" \
		--evidence-root "$evidence_root" \
		--manifest "${tmp_root}/bitaxe-ultra205-package.json" \
		--mode hardware >"${tmp_root}/detector.stdout" 2>"${tmp_root}/detector.stderr"
	local status=$?
	set -e

	if [[ "$status" -eq 0 ]]; then
		printf 'hardware mode detector failure should exit non-zero\n' >&2
		exit 1
	fi

	assert_detector_failure_slots_exist "$evidence_root"
	assert_contains "${tmp_root}/detector.stderr" "phase25_detector_status=blocked"
	assert_contains "${evidence_root}/detector.md" "slot_status: blocked"
	assert_contains "${evidence_root}/board-info.md" "slot_status: blocked"
	assert_contains "${evidence_root}/share-outcome.md" "share_outcome: blocked_safe_prerequisite"
	assert_contains "${evidence_root}/safe-stop.md" "safe_stop_status: blocked"
	assert_contains "${evidence_root}/summary.md" "package_artifact_status: not-run"
	assert_evidence_is_redacted "$evidence_root"
}

run_blocked_mode_test
run_detector_failure_test

printf 'phase25_live_stratum_evidence_test=passed\n'
