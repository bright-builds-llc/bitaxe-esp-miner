#!/usr/bin/env bash
set -euo pipefail

usage() {
	printf 'usage: %s --phase27-root PATH --evidence-root PATH\n' "$(basename "$0")" >&2
}

phase27_root=""
evidence_root=""

while [[ $# -gt 0 ]]; do
	case "$1" in
	--phase27-root)
		phase27_root="${2:-}"
		shift 2
		;;
	--evidence-root)
		evidence_root="${2:-}"
		shift 2
		;;
	-h | --help)
		usage
		exit 0
		;;
	*)
		printf 'unknown argument category\n' >&2
		usage
		exit 2
		;;
	esac
done

if [[ -z "$phase27_root" || -z "$evidence_root" ]]; then
	usage
	exit 2
fi

validate_repo_relative_path() {
	local value="$1"

	case "$value" in
	/* | .. | ../* | */.. | */../*) return 1 ;;
	esac

	return 0
}

if ! validate_repo_relative_path "$phase27_root" || ! validate_repo_relative_path "$evidence_root"; then
	printf 'evidence roots must be normalized repo-relative paths\n' >&2
	exit 2
fi

readonly parity_command="${PARITY_COMMAND:-bazel run //tools/parity:report --}"

printf 'phase28_evidence_step=consolidate-and-strict-validate\n'
${parity_command} consolidate-phase28-evidence --phase27-root "$phase27_root" --evidence-root "$evidence_root" >/dev/null
printf 'phase28_evidence_status=passed redacted=true\n'
