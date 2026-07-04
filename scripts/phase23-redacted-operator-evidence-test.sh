#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly wrapper="${PHASE23_EVIDENCE_SCRIPT:-${script_dir}/phase23-redacted-operator-evidence.sh}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase23-redacted-operator-evidence-test.XXXXXX")"
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

write_fake_parity() {
	local path="$1"

	cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

printf 'fake_operator_evidence_args: %s\n' "$*"
printf 'operator_evidence_status: passed\n'
SH
	chmod +x "$path"
}

write_failing_detector() {
	local path="$1"

	cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

if [[ "${PHASE23_DETECT_MUST_FAIL:-1}" == "1" ]]; then
	printf 'detector failed before board-info; package helpers must not have been called\n' >&2
	exit 42
fi

printf 'port=/dev/cu.usbmodem-sentinel\n'
SH
	chmod +x "$path"
}

assert_all_slots_exist() {
	local evidence_root="$1"
	local slot

	for slot in package detector board-info command log api websocket share-outcome safe-stop redaction-review conclusion; do
		if [[ ! -f "${evidence_root}/${slot}.md" ]]; then
			printf 'missing slot: %s\n' "$slot" >&2
			exit 1
		fi
	done
}

run_blocked_mode_test() {
	local fake_parity="${tmp_root}/fake-parity.sh"
	local evidence_root="${tmp_root}/blocked-root"
	write_fake_parity "$fake_parity"

	PHASE23_PARITY_COMMAND="$fake_parity" "$wrapper" \
		--evidence-root "$evidence_root" \
		--manifest "${tmp_root}/bitaxe-ultra205-package.json" \
		--mode blocked \
		--pool-credentials "${tmp_root}/sentinel-pool.invalid.json" \
		--wifi-credentials "${tmp_root}/SentinelWifi.json" \
		--device-url "http://192.0.2.55"

	assert_all_slots_exist "$evidence_root"
	assert_contains "${evidence_root}/command.md" "pool_config: local-owner-supplied"
	assert_contains "${evidence_root}/command.md" "raw_pool_values_committed: no"
	assert_contains "${evidence_root}/share-outcome.md" "accepted/rejected share outcomes remain non-claims"
	assert_contains "${evidence_root}/api.md" "stale DEVICE_URL"
	assert_contains "${evidence_root}/api.md" "network scan"
	assert_contains "${evidence_root}/websocket.md" "mDNS"

	local slot
	for slot in "${evidence_root}"/*.md; do
		assert_not_contains "$slot" "sentinel-pool.invalid"
		assert_not_contains "$slot" "sentinel-password"
		assert_not_contains "$slot" "sentinel-extra"
		assert_not_contains "$slot" "sentinel-share"
		assert_not_contains "$slot" "raw_bm1366_frame"
		assert_not_contains "$slot" "192.0.2.55"
	done
}

run_detector_failure_test() {
	local fake_parity="${tmp_root}/fake-parity-detector.sh"
	local fake_detector="${tmp_root}/fake-detect-ultra205.sh"
	local evidence_root="${tmp_root}/detector-root"
	write_fake_parity "$fake_parity"
	write_failing_detector "$fake_detector"

	set +e
	PHASE23_PARITY_COMMAND="$fake_parity" \
	PHASE23_DETECT_COMMAND="$fake_detector" \
	PHASE23_DETECT_MUST_FAIL=1 \
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

	assert_contains "${tmp_root}/detector.stderr" "phase23_detector_status=blocked"
	assert_contains "${evidence_root}/detector.md" "slot_status: blocked"
	assert_contains "${evidence_root}/board-info.md" "slot_status: blocked"
	assert_contains "${evidence_root}/detector.md" "just detect-ultra205"
	assert_contains "${evidence_root}/board-info.md" "Board-info blocked"
}

run_blocked_mode_test
run_detector_failure_test

printf 'phase23_redacted_operator_evidence_test=passed\n'
