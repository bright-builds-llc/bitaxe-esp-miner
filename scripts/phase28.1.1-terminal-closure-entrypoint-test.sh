#!/usr/bin/env bash
set -euo pipefail

script=${1:?guarded script path required}
expected="Phase 28.1.1 and descendants are closed — Won't Do (unresolved); Phase 30 is the only allowed continuation."

for mode in --help --dry-run; do
	set +e
	output=$(/bin/bash "$script" "$mode" 2>&1)
	status=$?
	set -e

	[[ $status -eq 64 ]] || {
		printf 'expected exit 64 from %s %s, got %s\n%s\n' "$script" "$mode" "$status" "$output" >&2
		exit 1
	}
	[[ $output == "$expected" ]] || {
		printf 'unexpected closure message from %s %s\n%s\n' "$script" "$mode" "$output" >&2
		exit 1
	}
done

printf 'terminal closure assertion passed: %s\n' "$(basename "$script")"
