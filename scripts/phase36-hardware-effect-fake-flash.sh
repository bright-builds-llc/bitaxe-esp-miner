#!/usr/bin/env bash
set -euo pipefail

: "${PHASE35_FLASH_STAGE_ROOT:?}"

mkdir "$PHASE35_FLASH_STAGE_ROOT"
chmod 700 "$PHASE35_FLASH_STAGE_ROOT"
printf '%s\n' "$@" >"$PHASE35_FLASH_STAGE_ROOT/flash.args"
chmod 600 "$PHASE35_FLASH_STAGE_ROOT/flash.args"
jq -cn \
	'{schema_version:"phase35-flash-boundary-v1",stage:"factory",transfer_started:true,completed:true}' \
	>"$PHASE35_FLASH_STAGE_ROOT/factory.metrics.json"
chmod 600 "$PHASE35_FLASH_STAGE_ROOT/factory.metrics.json"
