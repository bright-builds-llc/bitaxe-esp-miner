#!/usr/bin/env bash
# The wrapper must hand the command a private, socket-safe TMPDIR, remove it afterwards and keep
# the command's exit status.
set -euo pipefail

wrapper="$1"
# A short root, because this test itself runs under the wrapper.
work="$(mktemp -d /tmp/ttd.XXXXXX)"
trap 'rm -rf "$work"' EXIT

fail() { echo "FAIL: $*" >&2; exit 1; }

# Arrange
probe="$work/probe.sh"
cat > "$probe" <<'PROBE'
printf '%s\n' "$TMPDIR" > "$1"
stat -f '%Lp' "$TMPDIR" >> "$1"
touch "$TMPDIR/left-behind"
exit 7
PROBE

# Act
status=0
TMPDIR="$work" bash "$wrapper" bash "$probe" "$work/report" || status=$?

# Assert
private_tmpdir="$(sed -n 1p "$work/report")"
mode="$(sed -n 2p "$work/report")"
[[ "$status" -eq 7 ]] || fail "exit status $status, expected 7"
[[ "$private_tmpdir" == "$work"/bzt.* ]] || fail "TMPDIR $private_tmpdir is not a private child of the inherited TMPDIR"
[[ "$mode" == "700" ]] || fail "TMPDIR mode $mode, expected 700"
(( ${#private_tmpdir} + 20 <= 103 )) || fail "TMPDIR is too long for a Unix socket path below it"
[[ ! -e "$private_tmpdir" ]] || fail "TMPDIR was not removed"
echo "PASS"
