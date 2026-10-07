#!/usr/bin/env bash
# Bazel on macOS hands every test the user's TMPDIR, so each os.tmpdir() or std::env::temp_dir()
# directory a test leaves behind piles up there. Give each test its own private TMPDIR and remove it
# afterwards. TEST_TMPDIR itself is too long: Unix socket paths, which some tests create under
# TMPDIR, are limited to 104 bytes on macOS.
set -euo pipefail

private_tmpdir="$(mktemp -d "${TMPDIR:-/tmp}/bzt.XXXXXX")"
trap 'rm -rf "$private_tmpdir"' EXIT
# Bazel signals the whole process group, so the test receives the signal too; exiting here runs the
# cleanup trap once it has stopped.
trap 'exit 143' TERM
trap 'exit 130' INT

export TMPDIR="$private_tmpdir"
status=0
"$@" || status=$?
exit "$status"
