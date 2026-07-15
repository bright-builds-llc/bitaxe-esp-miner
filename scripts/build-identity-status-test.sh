#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly script_dir
readonly status_script="${BUILD_IDENTITY_STATUS_SCRIPT:-${script_dir}/build-identity-status.sh}"
readonly pathspec_contract="${BUILD_IDENTITY_PATHSPECS:-${script_dir}/build-identity-pathspecs.txt}"
tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/build-identity-status-test.XXXXXX")"
readonly tmp_root

cleanup() {
	rm -rf "$tmp_root"
}
trap cleanup EXIT

fail() {
	printf 'FAIL: %s\n' "$1" >&2
	exit 1
}

assert_contains() {
	local haystack="$1"
	local needle="$2"
	[[ "$haystack" == *"$needle"* ]] || fail "expected output to contain ${needle}"
}

assert_status_value() {
	local output="$1"
	local key="$2"
	local expected="$3"
	local actual
	actual="$(awk -v key="$key" '$1 == key { print $2 }' <<<"$output")"
	[[ "$actual" == "$expected" ]] || fail "${key}: expected ${expected}, found ${actual:-missing}"
}

create_repo() {
	local name="$1"
	local repo="${tmp_root}/${name}"
	mkdir -p "$repo/firmware/bitaxe/src" "$repo/crates/example/src" "$repo/tools/xtask/src" \
		"$repo/scripts" "$repo/reference/esp-miner" "$repo/.planning" "$repo/docs" "$repo/.cargo"
	cp "$status_script" "$repo/scripts/build-identity-status.sh"
	cp "$pathspec_contract" "$repo/scripts/build-identity-pathspecs.txt"
	chmod +x "$repo/scripts/build-identity-status.sh"
	printf '[package]\nname = "bitaxe-firmware"\nversion = "0.1.0"\n' >"$repo/firmware/bitaxe/Cargo.toml"
	printf 'fn main() {}\n' >"$repo/firmware/bitaxe/src/main.rs"
	printf 'pub fn example() {}\n' >"$repo/crates/example/src/lib.rs"
	printf 'fn main() {}\n' >"$repo/tools/xtask/src/main.rs"
	printf '[workspace]\n' >"$repo/Cargo.toml"
	printf '# lock\n' >"$repo/Cargo.lock"
	printf 'ignored.tmp\n' >"$repo/.gitignore"
	printf '7.0.0\n' >"$repo/.bazelversion"
	printf 'module(name = "test")\n' >"$repo/MODULE.bazel"
	printf 'package(default_visibility = ["//visibility:public"])\n' >"$repo/BUILD.bazel"
	printf 'build:\n' >"$repo/Justfile"
	printf '# scripts\n' >"$repo/scripts/BUILD.bazel"
	printf '#!/usr/bin/env bash\n' >"$repo/scripts/build-firmware.sh"
	printf '#!/usr/bin/env bash\n' >"$repo/scripts/package-firmware.sh"

	git -C "$repo/reference/esp-miner" init -q
	git -C "$repo/reference/esp-miner" config user.name "Build Identity Test"
	git -C "$repo/reference/esp-miner" config user.email "build-identity@example.invalid"
	printf 'reference\n' >"$repo/reference/esp-miner/README.md"
	git -C "$repo/reference/esp-miner" add README.md
	git -C "$repo/reference/esp-miner" commit -q -m reference

	git -C "$repo" init -q
	git -C "$repo" config user.name "Build Identity Test"
	git -C "$repo" config user.email "build-identity@example.invalid"
	git -C "$repo" config advice.addEmbeddedRepo false
	git -C "$repo" add . ':!reference/esp-miner'
	git -C "$repo" commit -q -m initial
	printf '%s\n' "$repo"
}

run_status() {
	local repo="$1"
	(cd "$repo" && scripts/build-identity-status.sh)
}

assert_dirty_case() {
	local name="$1"
	local operation="$2"
	local repo
	repo="$(create_repo "$name")"
	case "$operation" in
	staged)
		printf 'staged\n' >>"$repo/firmware/bitaxe/src/main.rs"
		git -C "$repo" add firmware/bitaxe/src/main.rs
		;;
	unstaged) printf 'unstaged\n' >>"$repo/crates/example/src/lib.rs" ;;
	untracked) printf 'new\n' >"$repo/tools/xtask/src/new.rs" ;;
	deleted) rm "$repo/scripts/package-firmware.sh" ;;
	renamed) git -C "$repo" mv crates/example/src/lib.rs crates/example/src/renamed.rs ;;
	*) fail "unknown dirty operation ${operation}" ;;
	esac
	assert_status_value "$(run_status "$repo")" STABLE_BITAXE_SOURCE_DIRTY true
}

test_clean_status_has_closed_key_set() {
	local repo output source_commit reference_commit key_count
	repo="$(create_repo clean)"
	output="$(run_status "$repo")"
	source_commit="$(git -C "$repo" rev-parse HEAD)"
	reference_commit="$(git -C "$repo/reference/esp-miner" rev-parse HEAD)"
	key_count="$(wc -l <<<"$output" | tr -d ' ')"
	[[ "$key_count" == "5" ]] || fail "status output did not contain exactly five keys"
	assert_status_value "$output" STABLE_BITAXE_SOURCE_COMMIT "$source_commit"
	assert_status_value "$output" STABLE_BITAXE_SOURCE_DIRTY false
	assert_status_value "$output" STABLE_BITAXE_RELEASE_TAG unavailable
	assert_status_value "$output" STABLE_BITAXE_SEMANTIC_VERSION 0.1.0
	assert_status_value "$output" STABLE_BITAXE_REFERENCE_COMMIT "$reference_commit"
	if awk '$1 !~ /^STABLE_BITAXE_/ { exit 1 }' <<<"$output"; then
		return
	fi
	fail "status output exposed a non-Bitaxe key"
}

test_excluded_and_ignored_changes_remain_clean() {
	local repo output
	repo="$(create_repo excluded)"
	printf 'plan\n' >"$repo/.planning/PLAN.md"
	printf 'docs\n' >"$repo/docs/notes.md"
	printf 'ignored\n' >"$repo/ignored.tmp"
	output="$(run_status "$repo")"
	assert_status_value "$output" STABLE_BITAXE_SOURCE_DIRTY false
}

test_release_tag_rules_and_detached_head() {
	local repo output
	repo="$(create_repo tags)"
	git -C "$repo" tag unrelated
	git -C "$repo" tag v1.2
	git -C "$repo" checkout -q --detach HEAD
	output="$(run_status "$repo")"
	assert_status_value "$output" STABLE_BITAXE_RELEASE_TAG v1.2
	git -C "$repo" tag v1.2.3
	if run_status "$repo" >"${tmp_root}/multiple.out" 2>&1; then
		fail "multiple allowed tags unexpectedly passed"
	fi
	assert_contains "$(<"${tmp_root}/multiple.out")" "multiple release tags"
}

test_missing_git_and_reference_fail_closed() {
	local repo
	mkdir -p "${tmp_root}/not-a-repo"
	if (cd "${tmp_root}/not-a-repo" && "$status_script") >"${tmp_root}/missing-git.out" 2>&1; then
		fail "missing Git metadata unexpectedly passed"
	fi
	repo="$(create_repo missing-reference)"
	rm -rf "$repo/reference/esp-miner/.git"
	if run_status "$repo" >"${tmp_root}/missing-reference.out" 2>&1; then
		fail "missing reference metadata unexpectedly passed"
	fi
}

test_clean_status_has_closed_key_set
assert_dirty_case staged staged
assert_dirty_case unstaged unstaged
assert_dirty_case untracked untracked
assert_dirty_case deleted deleted
assert_dirty_case renamed renamed
test_excluded_and_ignored_changes_remain_clean
test_release_tag_rules_and_detached_head
test_missing_git_and_reference_fail_closed

printf 'build identity status tests passed\n'
