#!/usr/bin/env bash
set -euo pipefail

[[ $# -eq 2 ]] || {
	printf 'usage: %s JUSTFILE FLASH_MAIN\n' "$0" >&2
	exit 2
}

justfile="$1"
flash_main="$2"

for contract in \
	'bazel run //tools/flash:flash -- detect' \
	'bazel run //tools/flash:flash -- flash ' \
	'bazel run //tools/flash:flash -- monitor ' \
	'bazel run //tools/flash:flash -- flash-monitor '; do
	grep -Fq "$contract" "$justfile" || {
		printf 'active USB entrypoint bypasses the shared flash supervisor: %s\n' "$contract" >&2
		exit 1
	}
done

grep -Fq '.run_espflash(' "$flash_main" || {
	printf 'espflash effects are not routed through UsbSession\n' >&2
	exit 1
}
grep -Fq '.run_espflash_probe(' "$flash_main" || {
	printf 'espflash prerequisite probes are not routed through UsbSession\n' >&2
	exit 1
}
grep -Fq '"bitaxe-receive-only"' "$flash_main" || {
	printf 'runtime monitoring is not routed through the receive-only adapter\n' >&2
	exit 1
}

production_source="$(awk '/^#\\[cfg\\(test\\)\\]/{exit} {print}' "$flash_main")"
if grep -Fq 'Command::new(self.espflash_bin.as_std_path())' <<<"$production_source"; then
	printf 'production code launches an unsupervised espflash child\n' >&2
	exit 1
fi
if grep -Fq 'CommandSpec::new("espflash", ["monitor"' <<<"$production_source"; then
	printf 'production monitor command renders reset-capable espflash monitor\n' >&2
	exit 1
fi

printf 'flash_supervisor_contract=ready\n'
