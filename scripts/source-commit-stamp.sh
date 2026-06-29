#!/usr/bin/env bash
set -euo pipefail

usage() {
  printf 'usage: %s <output-file>\n' "$0" >&2
}

if [[ "$#" -ne 1 ]]; then
  usage
  exit 2
fi

readonly output_file="$1"

source_commit="$(git rev-parse --short=12 HEAD)"
readonly source_commit

mkdir -p "$(dirname "$output_file")"
printf '%s\n' "$source_commit" >"$output_file"
