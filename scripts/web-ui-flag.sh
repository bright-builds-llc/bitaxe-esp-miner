#!/usr/bin/env bash
# Maps `just build|package [--web-ui current|solid]` to the Bazel build setting
# `--//firmware/bitaxe:web_ui=<variant>` (ADR-0034). Prints the flag; the
# default variant is `current`.
set -euo pipefail

variant=current
while (($# > 0)); do
  case "$1" in
    --web-ui)
      if (($# < 2)); then
        echo "web_ui_flag=failed reason=--web-ui needs current or solid" >&2
        exit 2
      fi
      variant="$2"
      shift 2
      ;;
    --web-ui=*)
      variant="${1#--web-ui=}"
      shift
      ;;
    *)
      echo "web_ui_flag=failed reason=unknown argument $1 (expected --web-ui current|solid)" >&2
      exit 2
      ;;
  esac
done

case "$variant" in
  current | solid) printf -- '--//firmware/bitaxe:web_ui=%s\n' "$variant" ;;
  *)
    echo "web_ui_flag=failed reason=unknown web UI variant $variant (expected current or solid)" >&2
    exit 2
    ;;
esac
