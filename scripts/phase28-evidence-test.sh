#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly wrapper="${PHASE28_EVIDENCE_SCRIPT:-${script_dir}/phase28-evidence.sh}"

tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/phase28-evidence-test.XXXXXX")"
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

assert_nonzero_status() {
	local status="$1"
	local scenario="$2"

	if [[ "$status" -eq 0 ]]; then
		printf '%s should exit non-zero\n' "$scenario" >&2
		exit 1
	fi
}

assert_phase28_inventory() {
	local root="$1"
	local slot

	for slot in package detector board-info command log api websocket share-outcome safe-stop redaction-review conclusion; do
		if [[ ! -f "${root}/${slot}.md" ]]; then
			printf 'missing Phase 28 slot: %s\n' "$slot" >&2
			exit 1
		fi
	done
	for required in summary.md .phase28-evidence-manifest; do
		if [[ ! -f "${root}/${required}" ]]; then
			printf 'missing Phase 28 generated file: %s\n' "$required" >&2
			exit 1
		fi
	done
	if [[ "$(find "$root" -maxdepth 1 -type f | wc -l | tr -d ' ')" -ne 13 ]]; then
		printf 'Phase 28 destination inventory must contain exactly 13 managed files\n' >&2
		exit 1
	fi
}

tree_digest() {
	local root="$1"

	if [[ ! -d "$root" ]]; then
		printf 'missing'
		return
	fi

	(
		cd "$root"
		find . -type f -print0 | LC_ALL=C sort -z | xargs -0 shasum -a 256
	) | shasum -a 256 | awk '{print $1}'
}

write_source_fixture() {
	local root="$1"
	local outcome="$2"
	local asic_status="$3"
	local safe_stop_status="$4"
	local redaction_status="${5:-passed}"

	mkdir -p "$root"
	for slot in detector board-info command conclusion; do
		printf 'slot: %s\nslot_status: passed\nredaction_status: passed\n' "$slot" >"${root}/${slot}.md"
	done
	cat >"${root}/summary.md" <<EOF
share_outcome: ${outcome}
asic_bridge_status: ${asic_status}
safe_stop_status: ${safe_stop_status}
redaction_status: ${redaction_status}
raw_artifacts_committed: no
raw_pool_values_committed: no
EOF
	cat >"${root}/share-outcome.md" <<EOF
slot: share-outcome
share_outcome: ${outcome}
asic_bridge_status: ${asic_status}
safe_stop_status: ${safe_stop_status}
redaction_status: ${redaction_status}
EOF
	cat >"${root}/redaction-review.md" <<EOF
slot: redaction-review
redaction_status: ${redaction_status}
raw_artifacts_committed: no
raw_pool_values_committed: no
EOF
	printf 'source-only-forbidden-sentinel\n' >"${root}/source-only-sentinel.txt"
}

write_managed_destination() {
	local root="$1"
	local marker="$2"

	mkdir -p "$root"
	printf 'phase28-evidence-v1\n' >"${root}/.phase28-evidence-manifest"
	printf '%s\n' "$marker" >"${root}/conclusion.md"
}

write_fake_parity() {
	local path="$1"

	cat >"$path" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

command_name="${1:-missing}"
shift || true
phase27_root=""
evidence_root=""
profile="none"
while [[ $# -gt 0 ]]; do
	case "$1" in
	--phase27-root) phase27_root="${2:-}"; shift 2 ;;
	--evidence-root) evidence_root="${2:-}"; shift 2 ;;
	--profile) profile="${2:-missing}"; shift 2 ;;
	--require-redaction-passed) shift ;;
	*) exit 97 ;;
	esac
done

if [[ "$command_name" == "consolidate-phase28-evidence" ]]; then
	printf 'command=consolidate-phase28-evidence profile=none\n' >>"${PHASE28_FAKE_PARITY_TRACE:?}"
	[[ "${PHASE28_FAKE_CONSOLIDATE_EXIT:-0}" -eq 0 ]] || exit "${PHASE28_FAKE_CONSOLIDATE_EXIT}"
	case "$phase27_root:$evidence_root" in
	/*:* | *:/* | *../*:* | *:*../*) exit 31 ;;
	esac
	if [[ "$phase27_root" == "$evidence_root" || "$phase27_root" == "$evidence_root"/* || "$evidence_root" == "$phase27_root"/* ]]; then
		exit 32
	fi
	for required in summary.md share-outcome.md redaction-review.md detector.md board-info.md command.md conclusion.md; do
		[[ -f "${phase27_root}/${required}" ]] || exit 33
	done
	outcome="$(awk -F': ' '/^share_outcome:/{print $2; exit}' "${phase27_root}/summary.md")"
	asic_status="$(awk -F': ' '/^asic_bridge_status:/{print $2; exit}' "${phase27_root}/summary.md")"
	safe_stop_status="$(awk -F': ' '/^safe_stop_status:/{print $2; exit}' "${phase27_root}/summary.md")"
	redaction_status="$(awk -F': ' '/^redaction_status:/{print $2; exit}' "${phase27_root}/summary.md")"
	[[ "$redaction_status" == "passed" ]] || exit 34
	case "$outcome" in
	accepted | rejected)
		[[ "$asic_status" == "result_correlated" && "$safe_stop_status" == "complete" ]] || exit 35
		;;
	blocked_safe_prerequisite)
		[[ "$asic_status" == "blocked" && "$safe_stop_status" == "complete" ]] || exit 36
		;;
	*) exit 37 ;;
	esac
	if [[ -d "$evidence_root" ]]; then
		[[ -f "${evidence_root}/.phase28-evidence-manifest" ]] || exit 38
		while IFS= read -r existing; do
			case "$existing" in
			.phase28-evidence-manifest | package.md | detector.md | board-info.md | command.md | log.md | api.md | websocket.md | share-outcome.md | safe-stop.md | redaction-review.md | conclusion.md | summary.md) ;;
			*) exit 39 ;;
			esac
		done < <(find "$evidence_root" -maxdepth 1 -type f -exec basename {} \;)
	fi
	[[ "${PHASE28_FAKE_INTERNAL_VALIDATION_EXIT:-0}" -eq 0 ]] || exit "${PHASE28_FAKE_INTERNAL_VALIDATION_EXIT}"
	staging="${evidence_root}.staging"
	rm -rf "$staging"
	mkdir -p "$staging"
	for slot in package detector board-info command log api websocket share-outcome safe-stop redaction-review conclusion; do
		cat >"${staging}/${slot}.md" <<EOF
slot: ${slot}
evidence_profile: phase28
evidence_disposition: cross_linked
source_phase27_category: allowlisted
share_outcome: ${outcome}
redaction_status: passed
raw_artifacts_committed: no
EOF
	done
	printf 'share_outcome: %s\nredaction_status: passed\n' "$outcome" >"${staging}/summary.md"
	printf 'phase28-evidence-v1\n' >"${staging}/.phase28-evidence-manifest"
	rm -rf "$evidence_root"
	mv "$staging" "$evidence_root"
	exit 0
fi

if [[ "$command_name" == "operator-evidence" ]]; then
	printf 'command=operator-evidence profile=%s\n' "$profile" >>"${PHASE28_FAKE_PARITY_TRACE:?}"
	[[ "$profile" == "phase28" && -d "$evidence_root" ]] || exit 41
	exit "${PHASE28_FAKE_OPERATOR_EXIT:-0}"
fi

exit 98
SH
	chmod +x "$path"
}

run_wrapper() {
	local fake_parity="$1"
	local trace_path="$2"
	local source_root="$3"
	local destination_root="$4"
	shift 4
	local parity_command="$fake_parity"
	if [[ "$parity_command" != */* ]]; then
		parity_command="./${parity_command}"
	fi

	PARITY_COMMAND="$parity_command" \
	PHASE28_FAKE_PARITY_TRACE="$trace_path" \
		"$wrapper" --phase27-root "$source_root" --evidence-root "$destination_root" "$@"
}

assert_success_trace() {
	local trace_path="$1"
	local expected=$'command=consolidate-phase28-evidence profile=none\ncommand=operator-evidence profile=phase28'
	local actual
	actual="$(<"$trace_path")"

	if [[ "$actual" != "$expected" ]]; then
		printf 'unexpected Phase 28 parity trace\nexpected:\n%s\nactual:\n%s\n' "$expected" "$actual" >&2
		exit 1
	fi
}

run_outcome_and_determinism_tests() {
	local outcome
	for outcome in accepted rejected blocked_safe_prerequisite; do
		local case_root="${tmp_root}/${outcome}"
		local source_root="${case_root}/source"
		local destination_root="${case_root}/destination"
		local fake_parity="${case_root}/fake-parity.sh"
		local trace_path="${case_root}/trace"
		local asic_status="result_correlated"
		if [[ "$outcome" == "blocked_safe_prerequisite" ]]; then
			asic_status="blocked"
		fi
		mkdir -p "$case_root"
		write_source_fixture "$source_root" "$outcome" "$asic_status" "complete"
		write_fake_parity "$fake_parity"
		(
			cd "$case_root"
			run_wrapper "fake-parity.sh" "trace" "source" "destination" >"stdout" 2>"stderr"
		)
		assert_success_trace "$trace_path"
		assert_phase28_inventory "$destination_root"
		assert_contains "${destination_root}/summary.md" "share_outcome: ${outcome}"
		assert_not_contains "${destination_root}/summary.md" "source-only-forbidden-sentinel"
		if [[ -e "${destination_root}/source-only-sentinel.txt" ]]; then
			printf 'source-only sentinel must not be copied into Phase 28\n' >&2
			exit 1
		fi
		assert_not_contains "${case_root}/stdout" "source-only-forbidden-sentinel"
		assert_not_contains "${case_root}/stderr" "source-only-forbidden-sentinel"
		local first_digest
		first_digest="$(tree_digest "$destination_root")"
		: >"$trace_path"
		(
			cd "$case_root"
			run_wrapper "fake-parity.sh" "trace" "source" "destination" >"rerun.stdout" 2>"rerun.stderr"
		)
		if [[ "$first_digest" != "$(tree_digest "$destination_root")" ]]; then
			printf 'deterministic rerun changed destination for %s\n' "$outcome" >&2
			exit 1
		fi
		assert_success_trace "$trace_path"
	done
}

run_rejection_and_preservation_tests() {
	local scenario
	for scenario in missing contradictory equal nested unknown-destination consolidate-failure validation-failure operator-failure; do
		local case_root="${tmp_root}/failure-${scenario}"
		local source_root="${case_root}/source"
		local destination_root="${case_root}/destination"
		local fake_parity="${case_root}/fake-parity.sh"
		local trace_path="${case_root}/trace"
		mkdir -p "$case_root"
		write_source_fixture "$source_root" "accepted" "result_correlated" "complete"
		write_managed_destination "$destination_root" "previous-valid-generation"
		write_fake_parity "$fake_parity"
		case "$scenario" in
		missing) rm "${source_root}/summary.md" ;;
		contradictory) write_source_fixture "$source_root" "accepted" "blocked" "complete" ;;
		unknown-destination) printf 'operator-owned\n' >"${destination_root}/unknown.txt" ;;
		esac
		local before_digest
		before_digest="$(tree_digest "$destination_root")"
		set +e
		(
			cd "$case_root"
			case "$scenario" in
			equal) run_wrapper "fake-parity.sh" "trace" "source" "source" ;;
			nested) run_wrapper "fake-parity.sh" "trace" "source" "source/nested" ;;
			consolidate-failure) PHASE28_FAKE_CONSOLIDATE_EXIT=51 run_wrapper "fake-parity.sh" "trace" "source" "destination" ;;
			validation-failure) PHASE28_FAKE_INTERNAL_VALIDATION_EXIT=52 run_wrapper "fake-parity.sh" "trace" "source" "destination" ;;
			operator-failure) PHASE28_FAKE_OPERATOR_EXIT=53 run_wrapper "fake-parity.sh" "trace" "source" "destination" ;;
			*) run_wrapper "fake-parity.sh" "trace" "source" "destination" ;;
			esac
		) >"${case_root}/stdout" 2>"${case_root}/stderr"
		local status=$?
		set -e
		assert_nonzero_status "$status" "$scenario"
		if [[ "$scenario" != "operator-failure" && "$before_digest" != "$(tree_digest "$destination_root")" ]]; then
			printf '%s changed the previous destination\n' "$scenario" >&2
			exit 1
		fi
	done
}

run_argument_surface_test() {
	local fake_parity="${tmp_root}/argument-fake-parity.sh"
	write_fake_parity "$fake_parity"

	local unsupported_flag
	for unsupported_flag in --pool-credentials --port --device-url --raw-log; do
		local trace_path="${tmp_root}/argument-${unsupported_flag#--}.trace"
		set +e
		PARITY_COMMAND="$fake_parity" PHASE28_FAKE_PARITY_TRACE="$trace_path" \
			"$wrapper" --phase27-root source --evidence-root destination "$unsupported_flag" forbidden >"${tmp_root}/argument.stdout" 2>"${tmp_root}/argument.stderr"
		local status=$?
		set -e
		assert_nonzero_status "$status" "unsupported ${unsupported_flag} argument"
		if [[ -e "$trace_path" ]]; then
			printf 'argument validation must finish before parity invocation\n' >&2
			exit 1
		fi
	done
}

run_outcome_and_determinism_tests
run_rejection_and_preservation_tests
run_argument_surface_test

printf 'phase28_evidence_test=passed\n'
