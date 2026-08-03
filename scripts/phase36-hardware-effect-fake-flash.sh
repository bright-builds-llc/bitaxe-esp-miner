#!/usr/bin/env bash
set -euo pipefail

: "${PHASE36_EFFECT_RESULT_PATH:?}"
: "${PHASE36_EFFECT_OPERATION:?}"
: "${PHASE36_EFFECT_PACKAGE_IDENTITY_DIGEST:?}"
: "${PHASE36_EFFECT_FACTORY_IMAGE_DIGEST:?}"
[[ -z "${PHASE35_FLASH_STAGE_ROOT+x}" ]] || {
	printf 'legacy Phase 35 stage root crossed the Phase 36 boundary\n' >&2
	exit 2
}

readonly capture_dir="$(dirname "$PHASE36_EFFECT_RESULT_PATH")/fake-flash"
mkdir "$capture_dir"
chmod 700 "$capture_dir"
printf '%s\n' "$@" >"$capture_dir/flash.args"
chmod 600 "$capture_dir/flash.args"
jq -cn \
	--arg operation "$PHASE36_EFFECT_OPERATION" \
	--arg package_identity_digest "$PHASE36_EFFECT_PACKAGE_IDENTITY_DIGEST" \
	--arg factory_image_digest "$PHASE36_EFFECT_FACTORY_IMAGE_DIGEST" \
	'{schema_version:"phase36-effect-result-v1",operation:$operation,status:"completed",
	failure:null,package_identity_digest:$package_identity_digest,
	factory_image_digest:$factory_image_digest}' \
	>"$PHASE36_EFFECT_RESULT_PATH"
chmod 600 "$PHASE36_EFFECT_RESULT_PATH"
