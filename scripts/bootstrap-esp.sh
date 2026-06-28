#!/usr/bin/env bash
set -euo pipefail

dry_run="false"

usage() {
	printf 'usage: %s [--dry-run]\n' "$0" >&2
	printf 'Installs ESP Rust tooling through cargo/espup when explicitly requested.\n' >&2
}

while [[ "$#" -gt 0 ]]; do
	case "$1" in
	--dry-run)
		dry_run="true"
		shift
		;;
	-h | --help)
		usage
		exit 0
		;;
	*)
		printf 'error: unexpected argument: %s\n' "$1" >&2
		usage
		exit 2
		;;
	esac
done

if [[ -z "${HOME:-}" ]]; then
	HOME="$(cd ~ && pwd)"
	export HOME
fi

if [[ -d "${HOME}/.cargo/bin" ]]; then
	PATH="${HOME}/.cargo/bin:${PATH}"
	export PATH
fi

readonly esp_export="${HOME}/export-esp.sh"

require_tool() {
	local tool="$1"
	local fix="$2"

	if command -v "$tool" >/dev/null 2>&1; then
		return
	fi

	printf 'error: %s is required before ESP bootstrap\n' "$tool" >&2
	printf 'fix: %s\n' "$fix" >&2
	exit 1
}

run_step() {
	printf 'run:'
	printf ' %q' "$@"
	printf '\n'

	if [[ "$dry_run" == "true" ]]; then
		return
	fi

	"$@"
}

needs_esp_install() {
	if [[ ! -f "$esp_export" ]]; then
		return 0
	fi

	local toolchains
	if ! toolchains="$(rustup toolchain list 2>/dev/null)"; then
		return 0
	fi

	case "$toolchains" in
	*esp*) return 1 ;;
	*) return 0 ;;
	esac
}

printf 'ESP-IDF contributor bootstrap\n'
if [[ "$dry_run" == "true" ]]; then
	printf 'mode: dry-run\n'
fi

require_tool cargo "Install Rust/Cargo from https://rustup.rs"
require_tool rustup "Install Rust with rustup from https://rustup.rs"

if ! command -v espup >/dev/null 2>&1; then
	run_step cargo install espup --locked
else
	printf 'ok: espup found\n'
fi

if needs_esp_install; then
	run_step espup install --targets esp32s3 --std
else
	printf 'ok: ESP Rust toolchain and export script already present\n'
fi

if ! command -v espflash >/dev/null 2>&1; then
	run_step cargo install espflash --locked
else
	printf 'ok: espflash found\n'
fi

printf 'next: source "%s" or open a new shell, then run `just doctor`.\n' "$esp_export"
