#!/usr/bin/env bash
set -euo pipefail

readonly contract_script="${1:?missing durability log contract}"
# shellcheck source=scripts/flash-durability-log-contract.sh
source "$contract_script"

readonly fixture_root="${TEST_TMPDIR:-/tmp}/flash-durability-log-contract-${RANDOM}${RANDOM}"
mkdir -p "$fixture_root"
trap 'rm -rf "$fixture_root"' EXIT

assert_ready() {
	local fixture="$1"
	durability_log_has_terminal_ready "$fixture" || {
		printf 'expected terminal ready marker: %s\n' "$(basename "$fixture")" >&2
		exit 1
	}
}

assert_not_ready() {
	local fixture="$1"
	if durability_log_has_terminal_ready "$fixture"; then
		printf 'unexpected terminal ready marker: %s\n' "$(basename "$fixture")" >&2
		exit 1
	fi
}

printf 'serial\nusb_session: ready\n' >"$fixture_root/well-framed.log"
assert_ready "$fixture_root/well-framed.log"

printf 'serialusb_session: ready\n' >"$fixture_root/concatenated.log"
assert_not_ready "$fixture_root/concatenated.log"

printf 'usb_session: ready\nserial\n' >"$fixture_root/earlier-only.log"
assert_not_ready "$fixture_root/earlier-only.log"

printf 'serial usb_session: ready suffix\n' >"$fixture_root/embedded.log"
assert_not_ready "$fixture_root/embedded.log"

: >"$fixture_root/empty.log"
assert_not_ready "$fixture_root/empty.log"

printf 'usb_session: ready\nserial\nusb_session: ready\n' >"$fixture_root/final-wins.log"
assert_ready "$fixture_root/final-wins.log"

printf 'flash_durability_log_contract=ready\n'
