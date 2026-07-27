#!/usr/bin/env bash
# Detector, capture, restoration, and cleanup helpers for the Phase 35 supervisor.
# shellcheck disable=SC2034,SC2154

resolve_phase35_effect_helper() {
	local helper_name="$1"
	local direct_dir
	direct_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

	local candidate_path
	for candidate_path in \
		"${direct_dir}/${helper_name}" \
		"${BASH_SOURCE[0]}.runfiles/_main/scripts/${helper_name}" \
		"${RUNFILES_DIR:-}/_main/scripts/${helper_name}"; do
		if [[ -f "$candidate_path" ]]; then
			printf '%s\n' "$candidate_path"
			return 0
		fi
	done

	printf 'failure_category=runfiles_incomplete\n' >&2
	return 1
}

phase35_admission_helper="$(resolve_phase35_effect_helper phase35-correlated-evidence-admission.sh)" || return 1
phase35_runtime_helper="$(resolve_phase35_effect_helper phase35-correlated-evidence-runtime.sh)" || return 1
phase35_finalization_helper="$(resolve_phase35_effect_helper phase35-correlated-evidence-finalization.sh)" || return 1
readonly phase35_admission_helper phase35_runtime_helper phase35_finalization_helper
# shellcheck source=scripts/phase35-correlated-evidence-admission.sh
source "$phase35_admission_helper"
# shellcheck source=scripts/phase35-correlated-evidence-runtime.sh
source "$phase35_runtime_helper"
# shellcheck source=scripts/phase35-correlated-evidence-finalization.sh
source "$phase35_finalization_helper"
