#!/usr/bin/env bash
set -euo pipefail

readonly unavailable="unavailable"

fail() {
	printf 'build identity status error: %s\n' "$1" >&2
	exit 1
}

repo_root="$(git rev-parse --show-toplevel 2>/dev/null)" || fail "Git metadata unavailable"
readonly repo_root
cd "$repo_root"

readonly pathspec_file="scripts/build-identity-pathspecs.txt"
[[ -f "$pathspec_file" ]] || fail "missing pathspec contract"

pathspecs=()
while IFS= read -r pathspec || [[ -n "$pathspec" ]]; do
	[[ -z "$pathspec" || "$pathspec" == \#* ]] && continue
	pathspecs+=("$pathspec")
done <"$pathspec_file"
[[ "${#pathspecs[@]}" -gt 0 ]] || fail "empty pathspec contract"

source_commit="$(git rev-parse --verify 'HEAD^{commit}' 2>/dev/null)" || fail "source commit unavailable"
[[ "$source_commit" =~ ^[0-9a-f]{40}$ ]] || fail "source commit must be a full lowercase hash"
readonly source_commit

source_dirty="false"
if [[ -n "$(git status --porcelain=v1 --untracked-files=all -- "${pathspecs[@]}")" ]]; then
	source_dirty="true"
fi
readonly source_dirty

matching_tags=()
while IFS= read -r tag; do
	[[ -z "$tag" ]] && continue
	if [[ "$tag" =~ ^v[0-9]+\.[0-9]+(\.[0-9]+)?$ ]]; then
		matching_tags+=("$tag")
	fi
done < <(git tag --points-at HEAD)
[[ "${#matching_tags[@]}" -le 1 ]] || fail "multiple release tags at HEAD"
release_tag="$unavailable"
if [[ "${#matching_tags[@]}" -eq 1 ]]; then
	release_tag="${matching_tags[0]}"
fi
readonly release_tag

semantic_version="$({
	awk '
		$0 == "[package]" { in_package = 1; next }
		/^\[/ { in_package = 0 }
		in_package && $1 == "version" && $2 == "=" {
			value = $3
			gsub(/^"|"$/, "", value)
			print value
		}
	' firmware/bitaxe/Cargo.toml
} 2>/dev/null)" || fail "semantic version unavailable"
[[ "$semantic_version" =~ ^[0-9A-Za-z][0-9A-Za-z.+-]*\.[0-9A-Za-z.+-]+$ ]] || fail "invalid semantic version"
readonly semantic_version

reference_root="$(git -C reference/esp-miner rev-parse --show-toplevel 2>/dev/null)" || fail "reference Git metadata unavailable"
[[ "$reference_root" == "$repo_root/reference/esp-miner" ]] || fail "reference Git metadata unavailable"
readonly reference_root
reference_commit="$(git -C "$reference_root" rev-parse --verify 'HEAD^{commit}' 2>/dev/null)" || fail "reference commit unavailable"
[[ "$reference_commit" =~ ^[0-9a-f]{40}$ ]] || fail "reference commit must be a full lowercase hash"
readonly reference_commit

printf 'STABLE_BITAXE_SOURCE_COMMIT %s\n' "$source_commit"
printf 'STABLE_BITAXE_SOURCE_DIRTY %s\n' "$source_dirty"
printf 'STABLE_BITAXE_RELEASE_TAG %s\n' "$release_tag"
printf 'STABLE_BITAXE_SEMANTIC_VERSION %s\n' "$semantic_version"
printf 'STABLE_BITAXE_REFERENCE_COMMIT %s\n' "$reference_commit"
