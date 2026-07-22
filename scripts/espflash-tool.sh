#!/usr/bin/env bash
# Shared exact espflash executable resolver.

readonly ESPFLASH_EXPECTED_VERSION="4.5.0"

espflash_resolve_bin() {
	local requested="${ESPFLASH_BIN:-espflash}"
	local resolved
	resolved="$(command -v -- "$requested" 2>/dev/null)" || return 1
	local link_count=0
	while [[ -L "$resolved" ]]; do
		((link_count += 1))
		((link_count <= 40)) || return 1
		local link
		link="$(readlink "$resolved")" || return 1
		if [[ "$link" == /* ]]; then
			resolved="$link"
		else
			resolved="${resolved%/*}/$link"
		fi
	done
	local directory
	directory="$(cd "${resolved%/*}" && pwd -P)" || return 1
	resolved="${directory}/${resolved##*/}"
	[[ -f "$resolved" && -x "$resolved" ]] || return 1
	printf '%s\n' "$resolved"
}

espflash_version() {
	local executable="$1"
	local version
	version="$("$executable" --version 2>/dev/null)" || return 1
	version="${version%%$'\n'*}"
	[[ "$version" == "espflash ${ESPFLASH_EXPECTED_VERSION}" ]] || return 1
	printf '%s\n' "$version"
}

espflash_executable_digest() {
	local output
	output="$(shasum -a 256 "$1")" || return 1
	printf '%s\n' "${output%% *}"
}
