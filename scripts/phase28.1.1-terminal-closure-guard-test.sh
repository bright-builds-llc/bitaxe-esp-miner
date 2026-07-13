#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
expected="Phase 28.1.1 and descendants are closed — Won't Do (unresolved); Phase 30 is the only allowed continuation."
scripts=(
	phase28.1.1-accepted-state-diagnostic.sh
	phase28.1.1-exact-head-hardware-attempt.sh
	phase28.1.1-upstream-wire-capture.sh
	phase28.1.1-wire-parity-capture.sh
	phase28.1.1.1-below-job-byte-diagnostic.sh
	phase28.1.1.1-source-work-aligned-capture.sh
	phase28.1.1.1-upstream-golden-capture.sh
	phase28.1.1.2-result-path-diagnostic.sh
	phase28.1.1.3-rx-acquisition-diagnostic.sh
	phase28.1.1.4-init-sequencing-diagnostic.sh
	phase28.1.1.5-chip-enumerate-diagnostic.sh
	phase28.1.1.6-version-rolling-diagnostic.sh
	phase28.1.1.7-asic-mask-reload-diagnostic.sh
	diagnose-ultra205-late-attach.sh
	diagnose-ultra205-uart-capture.sh
	ultra205-late-attach-broker.sh
	ultra205-late-attach-worker.sh
	ultra205-uart-capture-broker.sh
	ultra205-uart-capture-worker.sh
	ultra205-transport-qualification.sh
)

tmp_dir=$(mktemp -d)
trap 'rm -rf "$tmp_dir"' EXIT
marker="$tmp_dir/forbidden-side-effect"
stub_dir="$tmp_dir/stubs"
mkdir -p "$stub_dir"

for command_name in bazel cargo curl espflash git node perl python3 reset; do
	stub="$stub_dir/$command_name"
	printf '#!/usr/bin/env bash\nprintf invoked >%q\nexit 99\n' "$marker" >"$stub"
	chmod +x "$stub"
done

active_before=""
if [[ -d $repo_root/.planning/phases ]]; then
	active_before=$(find "$repo_root/.planning/phases" -maxdepth 1 -type d -name '28.1.1*' -print | sort)
fi

assert_closed() {
	local label=$1
	shift
	local output
	local status

	set +e
	output=$(PATH="$stub_dir:/usr/bin:/bin" "$@" 2>&1)
	status=$?
	set -e

	if [[ $status -ne 64 ]]; then
		printf 'expected exit 64 from %s, got %s\n%s\n' "$label" "$status" "$output" >&2
		exit 1
	fi
	if [[ $output != *"$expected"* ]]; then
		printf 'unexpected closure output from %s\n%s\n' "$label" "$output" >&2
		exit 1
	fi
	if [[ -e $marker ]]; then
		printf 'forbidden side effect invoked by %s\n' "$label" >&2
		exit 1
	fi
}

for script in "${scripts[@]}"; do
	assert_closed "$script --help" /bin/bash "$repo_root/scripts/$script" --help
	assert_closed "$script --dry-run" /bin/bash "$repo_root/scripts/$script" --dry-run
done

just_bin=$(command -v just)
assert_closed "just diagnose-ultra205-late-attach" "$just_bin" --justfile "$repo_root/Justfile" --working-directory "$repo_root" diagnose-ultra205-late-attach --help
assert_closed "just diagnose-ultra205-uart-capture" "$just_bin" --justfile "$repo_root/Justfile" --working-directory "$repo_root" diagnose-ultra205-uart-capture --help

active_after=""
if [[ -d $repo_root/.planning/phases ]]; then
	active_after=$(find "$repo_root/.planning/phases" -maxdepth 1 -type d -name '28.1.1*' -print | sort)
fi
if [[ $active_after != "$active_before" ]]; then
	printf 'active Phase 28.1.1 directory state changed\n' >&2
	exit 1
fi

if [[ -e $marker ]]; then
	printf 'guarded entrypoint invoked a forbidden command\n' >&2
	exit 1
fi

printf 'Phase 28.1.1 terminal closure guard tests passed\n'
