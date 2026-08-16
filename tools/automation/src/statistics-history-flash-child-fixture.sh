#!/usr/bin/env bash
set -euo pipefail

umask 077

if [[ "${1:-}" != "flash-monitor" ]]; then
  exit 64
fi
shift

evidence_dir=""
while (( $# > 0 )); do
  if [[ "$1" == "--evidence-dir" ]]; then
    evidence_dir="${2:-}"
    shift 2
    continue
  fi
  shift
done

test -n "$evidence_dir"
test -n "${PHASE36_EFFECT_RESULT_PATH:-}"
test -n "${PHASE36_EFFECT_PACKAGE_IDENTITY_DIGEST:-}"
test -n "${PHASE36_EFFECT_FACTORY_IMAGE_DIGEST:-}"

# Scale the whole production operation to separate pre-monitor, capture, and
# post-monitor phases. Their 440 ms total intentionally exceeds the old 420 ms
# policy before the child writes evidence and returns its effect result.
sleep 0.06
sleep 0.36
sleep 0.02
mkdir -p "$evidence_dir"
printf '%s\n' \
  'safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled' \
  'runtime_origin session=private device_url=http://private-device.test redacted=true' \
  > "$evidence_dir/flash-monitor.classifier-input.log"
printf '{"schema_version":"phase36-effect-result-v1","operation":"exact_package_flash","status":"completed","failure":null,"package_identity_digest":"%s","factory_image_digest":"%s"}\n' \
  "$PHASE36_EFFECT_PACKAGE_IDENTITY_DIGEST" \
  "$PHASE36_EFFECT_FACTORY_IMAGE_DIGEST" \
  > "$PHASE36_EFFECT_RESULT_PATH"
chmod 600 \
  "$evidence_dir/flash-monitor.classifier-input.log" \
  "$PHASE36_EFFECT_RESULT_PATH"
