#!/usr/bin/env bash
# Maps `just build|package [--board 205|601] [--web-ui current|solid]` to Bazel targets and the
# `--//firmware/bitaxe:web_ui=<variant>` build setting (ADR-0034, ADR-0041). Prints one
# argument per line. Defaults: board 205, web UI `current`.
set -euo pipefail

fail() {
  echo "firmware_build_args=failed reason=$1" >&2
  exit 2
}

(($# >= 1)) || fail "missing mode (build or package)"
mode="$1"
shift
board=205
variant=current
while (($# > 0)); do
  case "$1" in
    --board)
      (($# >= 2)) || fail "--board needs 205 or 601"
      board="$2"
      shift 2
      ;;
    --board=*)
      board="${1#--board=}"
      shift
      ;;
    --web-ui)
      (($# >= 2)) || fail "--web-ui needs current or solid"
      variant="$2"
      shift 2
      ;;
    --web-ui=*)
      variant="${1#--web-ui=}"
      shift
      ;;
    *) fail "unknown argument $1 (expected --board 205|601 or --web-ui current|solid)" ;;
  esac
done

case "$variant" in
  current | solid) ;;
  *) fail "unknown web UI variant $variant (expected current or solid)" ;;
esac
case "$mode:$board" in
  build:205) printf '%s\n' //firmware/bitaxe:firmware //firmware/bitaxe:web_ui_staged ;;
  build:601) printf '%s\n' //firmware/bitaxe:firmware_gamma601 //firmware/bitaxe:web_ui_staged ;;
  package:205) printf '%s\n' //firmware/bitaxe:firmware_image ;;
  package:601) printf '%s\n' //firmware/bitaxe:firmware_image_gamma601 ;;
  build:* | package:*) fail "unknown board $board (expected 205 or 601)" ;;
  *) fail "unknown mode $mode (expected build or package)" ;;
esac
printf -- '--//firmware/bitaxe:web_ui=%s\n' "$variant"
