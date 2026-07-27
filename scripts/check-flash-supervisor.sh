#!/usr/bin/env bash
set -euo pipefail

[[ $# -eq 4 ]] || {
	printf 'usage: %s JUSTFILE FLASH_ENVIRONMENT FLASH_MONITOR FLASH_COMMANDS\n' "$0" >&2
	exit 2
}

justfile="$1"
flash_environment="$2"
flash_monitor="$3"
flash_commands="$4"

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

grep -Fq '.run_espflash(' "$flash_environment" || {
	printf 'espflash effects are not routed through UsbSession\n' >&2
	exit 1
}
grep -Fq '.run_espflash_probe(' "$flash_environment" || {
	printf 'espflash prerequisite probes are not routed through UsbSession\n' >&2
	exit 1
}
grep -Fq '"bitaxe-receive-only"' "$flash_monitor" || {
	printf 'runtime monitoring is not routed through the receive-only adapter\n' >&2
	exit 1
}
grep -Fq 'write_receive_only_console(&bytes)?' "$flash_commands" || {
	printf 'receive-only output bypasses the framing helper\n' >&2
	exit 1
}

if grep -Fq 'Command::new(self.espflash_bin.as_std_path())' "$flash_environment"; then
	printf 'production code launches an unsupervised espflash child\n' >&2
	exit 1
fi
if grep -Fq '.write_all(&bytes)' "$flash_commands"; then
	printf 'production code writes receive-only bytes without framing\n' >&2
	exit 1
fi
if grep -Fq 'CommandSpec::new("espflash", ["monitor"' "$flash_monitor"; then
	printf 'production monitor command renders reset-capable espflash monitor\n' >&2
	exit 1
fi

printf 'flash_supervisor_contract=ready\n'
