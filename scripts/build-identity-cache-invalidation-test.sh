#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly rule_file="${BUILD_IDENTITY_RULE_FILE:-${script_dir}/build_identity.bzl}"
readonly firmware_build="${FIRMWARE_BUILD_FILE:-${script_dir}/../firmware/bitaxe/BUILD.bazel}"

fail() {
	printf 'FAIL: %s\n' "$1" >&2
	exit 1
}

assert_contains() {
	local file="$1"
	local pattern="$2"
	grep -Fq "$pattern" "$file" || fail "${file} missing ${pattern}"
}

assert_contains "$rule_file" 'inputs = [ctx.info_file] + ctx.files.srcs'
assert_contains "$rule_file" 'executable = ctx.executable._materializer'
assert_contains "$firmware_build" 'name = "build_provenance_inputs"'
assert_contains "$firmware_build" '"//crates/bitaxe-api:bitaxe_api"'
assert_contains "$firmware_build" '"//crates/bitaxe-asic:bitaxe_asic"'
assert_contains "$firmware_build" '"//crates/bitaxe-config:bitaxe_config"'
assert_contains "$firmware_build" '"//crates/bitaxe-core:bitaxe_core"'
assert_contains "$firmware_build" '"//crates/bitaxe-safety:bitaxe_safety"'
assert_contains "$firmware_build" '"//crates/bitaxe-stratum:bitaxe_stratum"'
assert_contains "$firmware_build" '"//:firmware_root_build_inputs"'

printf 'build identity cache invalidation source guard passed\n'
